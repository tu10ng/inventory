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

    run_migrations(&pool).await;
    rebuild_trip_items_fk(&pool).await;
    migrate_attrs(&pool).await;
    pool
}

async fn run_migrations(pool: &SqlitePool) {
    let sql = include_str!("../migrations/001_initial.sql");
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
    tracing::info!("Database migrations complete");
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

