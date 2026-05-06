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
