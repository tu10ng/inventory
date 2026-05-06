use serde::Deserialize;
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
    sync_categories(&pool).await;
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
            item_status TEXT NOT NULL DEFAULT '' CHECK(item_status IN ('', 'need_buy', 'need_find', 'need_charge', 'need_fetch', 'need_give')),
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

#[derive(Deserialize)]
struct CategoryConfig {
    id: i64,
    name: String,
    icon: String,
    sort_order: i64,
}

async fn sync_categories(pool: &SqlitePool) {
    let json = include_str!("../config/categories.json");
    let configs: Vec<CategoryConfig> = serde_json::from_str(json).expect("invalid categories.json");
    for c in &configs {
        sqlx::query(
            "INSERT INTO categories (id, name, icon, sort_order) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET name = excluded.name, icon = excluded.icon, sort_order = excluded.sort_order",
        )
        .bind(c.id)
        .bind(&c.name)
        .bind(&c.icon)
        .bind(c.sort_order)
        .execute(pool)
        .await
        .expect("failed to sync category");
    }
    tracing::info!("Synced {} categories from config", configs.len());
}
