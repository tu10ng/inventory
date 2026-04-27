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
    pool
}

async fn run_migrations(pool: &SqlitePool) {
    let sql = include_str!("../migrations/001_initial.sql");
    for statement in sql.split(';') {
        let trimmed = statement.trim();
        if !trimmed.is_empty() {
            sqlx::query(trimmed)
                .execute(pool)
                .await
                .expect("migration failed");
        }
    }
    tracing::info!("Database migrations complete");
}
