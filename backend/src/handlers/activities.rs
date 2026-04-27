use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::*;

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Activity>>, AppError> {
    let rows = sqlx::query_as::<_, Activity>("SELECT * FROM activities ORDER BY id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateActivity>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>(
        "INSERT INTO activities (name, description, icon) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.icon)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateActivity>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>(
        "UPDATE activities SET name = ?, description = ?, icon = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.icon)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

// ── Activity Items ──

pub async fn list_items(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<ActivityItem>>, AppError> {
    let rows = sqlx::query_as::<_, ActivityItem>(
        "SELECT * FROM activity_items WHERE activity_id = ? ORDER BY id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn add_item(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateActivityItem>,
) -> Result<Json<ActivityItem>, AppError> {
    let row = sqlx::query_as::<_, ActivityItem>(
        "INSERT INTO activity_items (activity_id, item_id, is_essential, default_qty, notes) VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(body.item_id)
    .bind(body.is_essential)
    .bind(body.default_qty)
    .bind(&body.notes)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn remove_item(
    State(pool): State<SqlitePool>,
    Path((activity_id, item_id)): Path<(i64, i64)>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM activity_items WHERE activity_id = ? AND item_id = ?")
        .bind(activity_id)
        .bind(item_id)
        .execute(&pool)
        .await?;
    Ok(())
}

// ── Tips ──

pub async fn list_tips(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<Tip>>, AppError> {
    let rows = sqlx::query_as::<_, Tip>(
        "SELECT * FROM tips WHERE activity_id = ? ORDER BY sort_order, id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_tip(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateTip>,
) -> Result<Json<Tip>, AppError> {
    let row = sqlx::query_as::<_, Tip>(
        "INSERT INTO tips (activity_id, content, sort_order) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(&body.content)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update_tip(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTip>,
) -> Result<Json<Tip>, AppError> {
    let existing = sqlx::query_as::<_, Tip>("SELECT * FROM tips WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let content = body.content.unwrap_or(existing.content);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    let row = sqlx::query_as::<_, Tip>(
        "UPDATE tips SET content = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&content)
    .bind(sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_tip(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tips WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
