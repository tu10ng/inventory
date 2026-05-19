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
    // migrate_remove_categories must run BEFORE other rebuilds because it
    // rebuilds items/types/activity_slots/display_rules tables without FK
    // to categories, and drops the categories table.
    migrate_remove_categories(pool).await;
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

    // Get all root type IDs (these replace category IDs for scope checking)
    let root_ids: Vec<(i64,)> =
        sqlx::query_as("SELECT id FROM types WHERE parent_id IS NULL")
            .fetch_all(pool)
            .await
            .unwrap_or_default();

    for (root_id,) in &root_ids {
        remove_map.entry(*root_id).or_default();
    }

    for (key, scope) in &attr_scopes {
        let allowed: HashSet<i64> = scope
            .split(',')
            .filter_map(|s| s.trim().parse::<i64>().ok())
            .collect();

        if allowed.is_empty() {
            continue;
        }

        for (root_id,) in &root_ids {
            if !allowed.contains(root_id) {
                remove_map
                    .entry(*root_id)
                    .or_default()
                    .push(key.clone());
            }
        }
    }

    // Items no longer have category_id - so scope-based cleaning is no longer
    // applicable at the item level. Mark as done and skip.
    tracing::info!("clean_out_of_scope_attrs: category_id removed, skipping item-level scope cleaning");

    // Record cleanup as done (custom marker, not a migration file)
    if let Err(e) = sqlx::query("INSERT INTO _migrations (filename) VALUES ('011_clean_attrs_done')")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to record clean_out_of_scope_attrs completion: {}", e);
    }

    tracing::info!("clean_out_of_scope_attrs complete (no-op after category removal)");
}

/// Rebuild tables to remove FK references to categories, then drop categories.
/// Creates root types from category names (parent_id IS NULL).
/// Must run on a SINGLE connection because PRAGMA foreign_keys = OFF must
/// persist across all statements in the rebuild.
async fn migrate_remove_categories(pool: &SqlitePool) {
    // Check idempotency
    let already_ran: Vec<(String,)> = sqlx::query_as(
        "SELECT filename FROM _migrations WHERE filename = '014_remove_categories_done'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if !already_ran.is_empty() {
        tracing::info!("migrate_remove_categories already executed, skipping");
        return;
    }

    // Check prerequisite: migration 014 marker must be applied
    let has_014: Vec<(String,)> = sqlx::query_as(
        "SELECT filename FROM _migrations WHERE filename = '014_remove_categories.sql'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if has_014.is_empty() {
        // Migration hasn't run yet (fresh DB), skip — will run after migration
        return;
    }

    // Check if categories table still exists (it may have already been dropped)
    let cat_exists: Vec<(String,)> = sqlx::query_as(
        "SELECT name FROM sqlite_master WHERE type='table' AND name='categories'",
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    if cat_exists.is_empty() {
        tracing::info!("categories table already dropped, recording marker and skipping");
        let _ = sqlx::query("INSERT INTO _migrations (filename) VALUES ('014_remove_categories_done')")
            .execute(pool)
            .await;
        return;
    }

    tracing::info!("Rebuilding tables to remove categories FK...");

    // Acquire a SINGLE connection so PRAGMA foreign_keys = OFF persists
    let mut conn = pool.acquire().await.expect("failed to acquire connection for migration");

    // Disable FK enforcement for the entire rebuild
    sqlx::query("PRAGMA foreign_keys = OFF")
        .execute(&mut *conn)
        .await
        .expect("failed to disable foreign keys");

    // ── 1. Rebuild types table (remove FK to categories, add root types) ──
    sqlx::query(
        "CREATE TABLE types_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            category_id INTEGER NOT NULL DEFAULT 0,
            sort_order INTEGER NOT NULL DEFAULT 0,
            parent_id INTEGER REFERENCES types(id)
        )",
    )
    .execute(&mut *conn)
    .await
    .expect("failed to create types_new");

    sqlx::query("INSERT INTO types_new SELECT id, name, category_id, sort_order, parent_id FROM types")
        .execute(&mut *conn)
        .await
        .expect("failed to copy types to types_new");

    // Insert root types from categories
    sqlx::query(
        "INSERT INTO types_new (name, category_id, sort_order, parent_id)
         SELECT name, 0, sort_order, NULL FROM categories ORDER BY id",
    )
    .execute(&mut *conn)
    .await
    .expect("failed to insert root types");

    // Remember the old→new root type mapping for scope remapping
    let cat_to_root: Vec<(i64, String)> = sqlx::query_as(
        "SELECT c.id, c.name FROM categories c ORDER BY c.id",
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap_or_default();

    let root_types: Vec<(i64, String)> = sqlx::query_as(
        "SELECT id, name FROM types_new WHERE parent_id IS NULL ORDER BY id",
    )
    .fetch_all(&mut *conn)
    .await
    .unwrap_or_default();

    sqlx::query("DROP TABLE types")
        .execute(&mut *conn)
        .await
        .expect("failed to drop old types table");

    sqlx::query("ALTER TABLE types_new RENAME TO types")
        .execute(&mut *conn)
        .await
        .expect("failed to rename types_new to types");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_types_parent_id ON types(parent_id)")
        .execute(&mut *conn)
        .await
        .ok();

    // ── 2. Rebuild items table (remove FK to categories) ──
    sqlx::query(
        "CREATE TABLE items_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL DEFAULT '',
            brand TEXT NOT NULL DEFAULT '',
            model TEXT NOT NULL DEFAULT '',
            category_id INTEGER NOT NULL DEFAULT 0,
            default_qty INTEGER NOT NULL DEFAULT 1,
            notes TEXT NOT NULL DEFAULT '',
            type_id INTEGER REFERENCES types(id),
            attrs TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(&mut *conn)
    .await
    .expect("failed to create items_new");

    sqlx::query("INSERT INTO items_new SELECT id, name, brand, model, category_id, default_qty, notes, type_id, COALESCE(attrs,'{}') FROM items")
        .execute(&mut *conn)
        .await
        .expect("failed to copy items to items_new");

    sqlx::query("DROP TABLE items")
        .execute(&mut *conn)
        .await
        .expect("failed to drop old items table");

    sqlx::query("ALTER TABLE items_new RENAME TO items")
        .execute(&mut *conn)
        .await
        .expect("failed to rename items_new to items");

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_type ON items(type_id)")
        .execute(&mut *conn)
        .await
        .ok();

    sqlx::query("CREATE INDEX IF NOT EXISTS idx_items_name ON items(json_extract(attrs, '$.name'))")
        .execute(&mut *conn)
        .await
        .ok();

    // ── 3. Rebuild activity_slots table (remove FK to categories) ──
    sqlx::query(
        "CREATE TABLE activity_slots_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            activity_id INTEGER NOT NULL REFERENCES activities(id),
            slot_name TEXT NOT NULL,
            category_id INTEGER NOT NULL DEFAULT 0,
            is_essential INTEGER NOT NULL DEFAULT 1,
            default_qty INTEGER NOT NULL DEFAULT 1,
            notes TEXT NOT NULL DEFAULT '',
            sort_order INTEGER NOT NULL DEFAULT 0,
            default_item_id INTEGER REFERENCES items(id)
        )",
    )
    .execute(&mut *conn)
    .await
    .expect("failed to create activity_slots_new");

    sqlx::query("INSERT INTO activity_slots_new SELECT id, activity_id, slot_name, category_id, is_essential, default_qty, notes, sort_order, default_item_id FROM activity_slots")
        .execute(&mut *conn)
        .await
        .expect("failed to copy activity_slots");

    sqlx::query("DROP TABLE activity_slots")
        .execute(&mut *conn)
        .await
        .expect("failed to drop old activity_slots table");

    sqlx::query("ALTER TABLE activity_slots_new RENAME TO activity_slots")
        .execute(&mut *conn)
        .await
        .expect("failed to rename activity_slots_new to activity_slots");

    // ── 4. Rebuild display_rules table (remove FK to categories) ──
    sqlx::query(
        "CREATE TABLE display_rules_new (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            category_id INTEGER DEFAULT 0,
            group_by_key TEXT NOT NULL DEFAULT '',
            sort_by_key TEXT NOT NULL DEFAULT '',
            sort_dir TEXT NOT NULL DEFAULT 'asc',
            visible_columns TEXT NOT NULL DEFAULT '[]',
            sort_order INTEGER NOT NULL DEFAULT 0,
            config TEXT NOT NULL DEFAULT '{}'
        )",
    )
    .execute(&mut *conn)
    .await
    .expect("failed to create display_rules_new");

    sqlx::query("INSERT INTO display_rules_new SELECT id, name, category_id, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config FROM display_rules")
        .execute(&mut *conn)
        .await
        .expect("failed to copy display_rules");

    sqlx::query("DROP TABLE display_rules")
        .execute(&mut *conn)
        .await
        .expect("failed to drop old display_rules table");

    sqlx::query("ALTER TABLE display_rules_new RENAME TO display_rules")
        .execute(&mut *conn)
        .await
        .expect("failed to rename display_rules_new to display_rules");

    // ── 5. Remap parent_id and scope BEFORE zeroing category_id ──
    // (uses the cat_to_root mapping captured before the rebuild)
    for (old_cat_id, cat_name) in &cat_to_root {
        if let Some((new_root_id, _)) = root_types.iter().find(|(_, name)| name == cat_name) {
            // Update types.parent_id: top-level types under old category → point to new root
            sqlx::query(
                "UPDATE types SET parent_id = ? WHERE category_id = ? AND parent_id IS NULL AND id != ?",
            )
            .bind(new_root_id)
            .bind(old_cat_id)
            .bind(new_root_id)
            .execute(&mut *conn)
            .await
            .ok();

            // Replace old category ID with new root type ID in scope strings
            let old_id_str = old_cat_id.to_string();
            let new_id_str = new_root_id.to_string();
            sqlx::query(
                "UPDATE attribute_definitions SET category_scope = REPLACE(category_scope, ?1, ?2) WHERE category_scope LIKE ?3",
            )
            .bind(&old_id_str)
            .bind(&new_id_str)
            .bind(format!("%{}%", old_id_str))
            .execute(&mut *conn)
            .await
            .ok();
        }
    }

    // ── 6. Set legacy category_id = 0 ──
    sqlx::query("UPDATE items SET category_id = 0")
        .execute(&mut *conn).await.ok();
    sqlx::query("UPDATE types SET category_id = 0")
        .execute(&mut *conn).await.ok();
    sqlx::query("UPDATE activity_slots SET category_id = 0")
        .execute(&mut *conn).await.ok();
    sqlx::query("UPDATE display_rules SET category_id = 0")
        .execute(&mut *conn).await.ok();

    // ── 7. Now safe to drop categories (no FKs reference it anymore) ──
    sqlx::query("DROP TABLE categories")
        .execute(&mut *conn)
        .await
        .expect("failed to drop categories table");

    // ── 8. Re-enable FK enforcement ──
    sqlx::query("PRAGMA foreign_keys = ON")
        .execute(&mut *conn)
        .await
        .expect("failed to re-enable foreign keys");

    // Release the connection back to the pool
    drop(conn);

    // ── 9. Record completion ──
    if let Err(e) = sqlx::query("INSERT INTO _migrations (filename) VALUES ('014_remove_categories_done')")
        .execute(pool)
        .await
    {
        tracing::warn!("Failed to record migrate_remove_categories completion: {}", e);
    }

    tracing::info!("migrate_remove_categories complete: tables rebuilt without categories FK");
}

