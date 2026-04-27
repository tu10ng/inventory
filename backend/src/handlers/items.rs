use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateItem, Item};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Item>>, AppError> {
    let rows = sqlx::query_as::<_, Item>("SELECT * FROM items ORDER BY category_id, id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Item>, AppError> {
    let row = sqlx::query_as::<_, Item>("SELECT * FROM items WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateItem>,
) -> Result<Json<Item>, AppError> {
    let row = sqlx::query_as::<_, Item>(
        "INSERT INTO items (name, brand, model, category_id, default_qty, notes) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.brand)
    .bind(&body.model)
    .bind(body.category_id)
    .bind(body.default_qty)
    .bind(&body.notes)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateItem>,
) -> Result<Json<Item>, AppError> {
    let row = sqlx::query_as::<_, Item>(
        "UPDATE items SET name = ?, brand = ?, model = ?, category_id = ?, default_qty = ?, notes = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.brand)
    .bind(&body.model)
    .bind(body.category_id)
    .bind(body.default_qty)
    .bind(&body.notes)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
