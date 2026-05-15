use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use sqlx::SqlitePool;
use std::str::FromStr;

pub async fn init_pool() -> SqlitePool {
    let opts = SqliteConnectOptions::from_str("sqlite:inventory.db")
        .expect("invalid db url")
        .create_if_missing(true)
        .journal_mode(sqlx::sqlite::SqliteJournalMode::Wal);

    let pool = SqlitePoolOptions::new()
        .max_connections(5)
        .connect_with(opts)
        .await
        .expect("failed to connect to database");

    run_all_setup(&pool).await;
    pool
}

async fn run_all_setup(pool: &SqlitePool) {
    run_migrations(pool).await;
    rebuild_trip_items_fk(pool).await;
    rebuild_trips_table(pool).await;
    migrate_attrs(pool).await;
    rebuild_items_table(pool).await;
    clean_out_of_scope_attrs(pool).await;
}

/// Create an in-memory SQLite pool for testing, with all migrations applied.
/// Each test should wrap work in a transaction and rollback to keep tests isolated.
#[cfg(test)]
pub async fn init_test_pool() -> SqlitePool {
    let opts = SqliteConnectOptions::from_str("sqlite::memory:")
        .expect("invalid db url")
        .create_if_missing(true);

    let pool = SqlitePoolOptions::new()
        .max_connections(2)
        .connect_with(opts)
        .await
        .expect("failed to create test db");

    run_all_setup(&pool).await;
    pool
}

async fn run_migrations(pool: &SqlitePool) {
    // Create migration tracking table
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS _migrations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL UNIQUE,
            applied_at TEXT NOT NULL DEFAULT (datetime('now'))
        )",
    )
    .execute(pool)
    .await
    .expect("failed to create _migrations table");

    // Scan migration files
    let migrations_dir = std::path::Path::new("migrations");
    if !migrations_dir.exists() {
        tracing::warn!("migrations/ directory not found, skipping");
        return;
    }

    let mut files: Vec<String> = std::fs::read_dir(migrations_dir)
        .expect("failed to read migrations/ directory")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            let name = entry.file_name().to_string_lossy().to_string();
            if name.ends_with(".sql") {
                Some(name)
            } else {
                None
            }
        })
        .collect();
    files.sort();

    // Get already-applied migrations
    let applied: Vec<(String,)> =
        sqlx::query_as("SELECT filename FROM _migrations ORDER BY filename")
            .fetch_all(pool)
            .await
            .unwrap_or_default();
    let applied_set: std::collections::HashSet<&str> =
        applied.iter().map(|(f,)| f.as_str()).collect();

    for filename in &files {
        if applied_set.contains(filename.as_str()) {
            tracing::debug!("Skipping already-applied migration: {}", filename);
            continue;
        }

        let path = migrations_dir.join(filename);
        let sql = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));

        tracing::info!("Applying migration: {}", filename);
        execute_sql_file(pool, &sql).await;

        sqlx::query("INSERT INTO _migrations (filename) VALUES (?)")
            .bind(filename)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("failed to record migration {}: {e}", filename));
    }

    tracing::info!("Database migrations complete");
}

async fn execute_sql_file(pool: &SqlitePool, sql: &str) {
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            match sqlx::query(trimmed).execute(pool).await {
                Ok(_) => {}
                Err(e) => {
                    let msg = e.to_string();
                    // ALTER TABLE ADD COLUMN fails if column already exists — safe to ignore
                    if msg.contains("duplicate column name") {
                        tracing::warn!("Skipping migration (column exists): {}", trimmed);
                    } else {
                        panic!("migration failed: {e}\nStatement: {trimmed}");
                    }
                }
            }
        }
    }
}

/// Rebuild trip_items table to add ON DELETE SET NULL on item_id and person_id.
/// Needs PRAGMA foreign_keys = OFF during the rebuild, which can't be done in
/// the split-by-semicolon migration file.
/// Idempotent: checks if the FK already has ON DELETE SET NULL before rebuilding.
async fn rebuild_trip_items_fk(pool: &SqlitePool) {
    // Check current FK definition — if already has ON DELETE SET NULL, skip
    let fk_info: Vec<(String,)> = sqlx::query_as(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='trip_items'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if let Some((ref sql,)) = fk_info.first() {
        if sql.contains("ON DELETE SET NULL") {
            tracing::info!("trip_items FK already has ON DELETE SET NULL, skipping rebuild");
            return;
        }
    } else {
        return; // table doesn't exist yet (fresh DB will create it correctly)
    }

    tracing::info!("Rebuilding trip_items table to add ON DELETE SET NULL...");

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await
        .expect("failed to disable foreign keys");

    let stmts = [
        "CREATE TABLE trip_items_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            trip_id INTEGER NOT NULL REFERENCES trips(id) ON DELETE CASCADE,
            item_id INTEGER REFERENCES items(id) ON DELETE SET NULL,
            custom_name TEXT NOT NULL DEFAULT '',
            person_id INTEGER REFERENCES people(id) ON DELETE SET NULL,
            qty INTEGER NOT NULL DEFAULT 1,
            checked INTEGER NOT NULL DEFAULT 0,
            item_status TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            is_essential INTEGER NOT NULL DEFAULT 0,
            slot_id INTEGER REFERENCES activity_slots(id)
        )",
        "INSERT INTO trip_items_new SELECT * FROM trip_items",
        "DROP TABLE trip_items",
        "ALTER TABLE trip_items_new RENAME TO trip_items",
    ];

    for stmt in &stmts {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("trip_items FK rebuild failed: {e}\nStatement: {stmt}"));
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .expect("failed to re-enable foreign keys");

    tracing::info!("trip_items FK rebuild complete");
}

/// Rebuild trips table without CHECK(status IN (...)) constraint.
/// SQLite doesn't support ALTER TABLE DROP CHECK, so we must recreate the table.
/// Idempotent: checks if the table definition contains a CHECK constraint before rebuilding.
async fn rebuild_trips_table(pool: &SqlitePool) {
    let table_info: Vec<(String,)> = sqlx::query_as(
        "SELECT sql FROM sqlite_master WHERE type='table' AND name='trips'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if table_info.is_empty() {
        return; // table doesn't exist yet (fresh DB will create it without CHECK)
    }

    if let Some((ref sql,)) = table_info.first() {
        if !sql.contains("CHECK") {
            tracing::info!("trips table already has no CHECK constraint, skipping rebuild");
            return;
        }
    }

    tracing::info!("Rebuilding trips table to remove CHECK constraint...");

    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(pool)
        .await
        .expect("failed to disable foreign keys");

    let stmts = [
        "CREATE TABLE trips_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            activity_id INTEGER REFERENCES activities(id),
            start_date TEXT NOT NULL DEFAULT '',
            end_date TEXT NOT NULL DEFAULT '',
            notes TEXT NOT NULL DEFAULT '',
            status TEXT NOT NULL DEFAULT 'planning'
        )",
        "INSERT INTO trips_new SELECT * FROM trips",
        "DROP TABLE trips",
        "ALTER TABLE trips_new RENAME TO trips",
    ];

    for stmt in &stmts {
        sqlx::query(stmt)
            .execute(pool)
            .await
            .unwrap_or_else(|e| panic!("trips rebuild failed: {e}\nStatement: {stmt}"));
    }

    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(pool)
        .await
        .expect("failed to re-enable foreign keys");

    tracing::info!("trips table rebuild complete");
}

/// Pack old physical columns (name, brand, model, default_qty, notes) into attrs JSON.
/// Does NOT drop the old columns — they stay in the table but are no longer used by code.
/// Idempotent: checks if old column 'name' exists and attrs JSON doesn't already have name.
async fn rebuild_items_table(pool: &SqlitePool) {
    // Check if old column 'name' exists on items table
    let col_info: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM pragma_table_info('items') WHERE name = 'name'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if col_info.is_empty() {
        tracing::info!("items table already uses only attrs JSON, skipping rebuild");
        return;
    }

    // Check if items already have name in attrs (meaning migration already ran)
    let already_migrated: Vec<(i64,)> = sqlx::query_as(
        "SELECT id FROM items WHERE json_extract(attrs, '$.name') IS NOT NULL AND json_extract(attrs, '$.name') != '' LIMIT 1",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !already_migrated.is_empty() {
        tracing::info!("items already have name in attrs, skipping rebuild");
        return;
    }

    tracing::info!("Packing old item columns into attrs JSON...");

    // Pack name/brand/model/default_qty/notes into attrs, merging with existing attrs
    sqlx::query(
        "UPDATE items SET attrs = json_patch(
            json_object(
                'name', name,
                'brand', brand,
                'model', model,
                'default_qty', default_qty,
                'notes', notes
            ),
            CASE WHEN attrs IS NOT NULL AND attrs != '' THEN attrs ELSE '{}' END
        )",
    )
    .execute(pool)
    .await
    .expect("failed to pack item columns into attrs");

    // Create expression indexes for name search/sort performance
    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_items_category_name ON items(category_id, json_extract(attrs, '$.name'))",
    )
    .execute(pool)
    .await
    .expect("failed to create idx_items_category_name");

    sqlx::query(
        "CREATE INDEX IF NOT EXISTS idx_items_name ON items(json_extract(attrs, '$.name'))",
    )
    .execute(pool)
    .await
    .expect("failed to create idx_items_name");

    tracing::info!("attrs JSON packing complete");
}

/// Migrate old hardcoded item columns into the attrs JSON column.
/// Idempotent: only runs on items where attrs is still '{}' and old columns have data.
async fn migrate_attrs(pool: &SqlitePool) {
    // Check if old columns exist (they might not on a fresh DB after a future cleanup)
    let table_info: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM pragma_table_info('items') WHERE name = 'warmth_rating'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if table_info.is_empty() {
        return; // Old columns don't exist, nothing to migrate
    }

    // Pack old column values into attrs JSON for items that still have attrs = '{}'
    let migrated = sqlx::query(
        "UPDATE items SET attrs = json_object(
            'warmth_rating', warmth_rating,
            'material', material,
            'encumbrance', encumbrance,
            'waterproof', waterproof,
            'weight_grams', weight_grams,
            'season', season,
            'body_parts', body_parts,
            'env_protection', env_protection,
            'durability', durability,
            'storage_ml', storage_ml,
            'breathable', breathable
        ) WHERE attrs = '{}' AND (
            warmth_rating != 0 OR material != '' OR encumbrance != 0 OR
            waterproof != 0 OR weight_grams != 0 OR season != '' OR
            body_parts != '' OR env_protection != 0 OR durability != 0 OR
            storage_ml != 0 OR breathable != 0
        )",
    )
    .execute(pool)
    .await;

    match migrated {
        Ok(result) => {
            if result.rows_affected() > 0 {
                tracing::info!(
                    "Migrated {} items from old columns to attrs JSON",
                    result.rows_affected()
                );
            }
        }
        Err(e) => {
            tracing::warn!("attrs migration failed (may be expected on fresh DB): {}", e);
        }
    }
}

/// Remove out-of-scope attribute keys from items' attrs JSON.
/// For each attribute definition with a non-empty category_scope, if an item's
/// category_id is not in that scope, the key is removed from the item's attrs.
/// Core fields (name/brand/model/notes/default_qty) have empty scope and are kept.
/// Ad-hoc keys (not in attribute_definitions) are also kept.
/// Requires migration 010 (scope fix) to have run first.
/// Idempotent: checks _migrations for 011_clean_attrs.sql before running.
async fn clean_out_of_scope_attrs(pool: &SqlitePool) {
    // Check prerequisite: migration 010 must be applied
    let has_010: Vec<(String,)> = sqlx::query_as(
        "SELECT filename FROM _migrations WHERE filename = '010_fix_category_scope.sql'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if has_010.is_empty() {
        // 010 hasn't run yet (fresh DB), skip — migrations will handle scope naturally
        return;
    }

    // Check idempotency: has cleanup already run?
    let already_ran: Vec<(String,)> = sqlx::query_as(
        "SELECT filename FROM _migrations WHERE filename = '011_clean_attrs_done'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !already_ran.is_empty() {
        tracing::info!("clean_out_of_scope_attrs already executed, skipping");
        return;
    }

    tracing::info!("Cleaning out-of-scope attrs from items...");

    // Load attribute_definitions scope map: key → set of allowed category IDs
    let attr_scopes: Vec<(String, String)> = sqlx::query_as(
        "SELECT key, category_scope FROM attribute_definitions WHERE category_scope != ''",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    // Collect keys to remove per category_id
    use std::collections::{HashMap, HashSet};
    let mut remove_map: HashMap<i64, Vec<String>> = HashMap::new();

    // Get all category IDs
    let cat_ids: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM categories")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    for (cat_id,) in &cat_ids {
        remove_map.entry(*cat_id).or_default();
    }

    for (key, scope) in &attr_scopes {
        let allowed: HashSet<i64> = scope
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();

        if allowed.is_empty() {
            continue;
        }

        for (cat_id,) in &cat_ids {
            if !allowed.contains(cat_id) {
                remove_map
                    .entry(*cat_id)
                    .or_default()
                    .push(key.clone());
            }
        }
    }

    // Load all items and clean their attrs
    let items: Vec<(i64, i64, String)> = sqlx::query_as(
        "SELECT id, category_id, COALESCE(attrs, '{}') FROM items",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    let mut cleaned_count = 0;
    for (item_id, cat_id, attrs_json) in &items {
        let keys_to_remove = match remove_map.get(cat_id) {
            Some(keys) if !keys.is_empty() => keys,
            _ => continue,
        };

        let mut attrs: serde_json::Value =
            serde_json::from_str(attrs_json).unwrap_or(serde_json::Value::Object(Default::default()));

        let mut removed_any = false;
        if let Some(obj) = attrs.as_object_mut() {
            for key in keys_to_remove {
                if obj.remove(key).is_some() {
                    removed_any = true;
                }
            }
        }

        if removed_any {
            let new_attrs = serde_json::to_string(&attrs).unwrap();
            if let Err(e) = sqlx::query("UPDATE items SET attrs = ? WHERE id = ?")
                .bind(&new_attrs)
                .bind(item_id)
                .execute(pool)
                .await
            {
                tracing::warn!("Failed to update attrs for item {}: {}", *item_id, e);
            }
            cleaned_count += 1;
        }
    }

    // Record cleanup as done (custom marker, not a migration file)
    if let Err(e) = sqlx::query("INSERT INTO _migrations (filename) VALUES ('011_clean_attrs_done')")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to record clean_out_of_scope_attrs completion: {}", e);
    }

    tracing::info!(
        "clean_out_of_scope_attrs complete: cleaned {} items",
        cleaned_count
    );
}

