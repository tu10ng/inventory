use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateItem, Item, ItemUsageCount, ItemUsageStats, TripRef};

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
        "INSERT INTO items (name, brand, model, category_id, default_qty, notes, tag_id) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.brand)
    .bind(&body.model)
    .bind(body.category_id)
    .bind(body.default_qty)
    .bind(&body.notes)
    .bind(body.tag_id)
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
        "UPDATE items SET name = ?, brand = ?, model = ?, category_id = ?, default_qty = ?, notes = ?, tag_id = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.brand)
    .bind(&body.model)
    .bind(body.category_id)
    .bind(body.default_qty)
    .bind(&body.notes)
    .bind(body.tag_id)
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

pub async fn usage_stats(State(pool): State<SqlitePool>) -> Result<Json<Vec<ItemUsageCount>>, AppError> {
    let rows = sqlx::query_as::<_, ItemUsageCount>(
        "SELECT item_id, COUNT(DISTINCT trip_id) as trip_count FROM trip_items WHERE item_id IS NOT NULL GROUP BY item_id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn usage_detail(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ItemUsageStats>, AppError> {
    let trips = sqlx::query_as::<_, TripRef>(
        "SELECT DISTINCT t.id, t.name, t.status FROM trips t JOIN trip_items ti ON ti.trip_id = t.id WHERE ti.item_id = ? ORDER BY t.id DESC",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(ItemUsageStats { item_id: id, trips }))
}
