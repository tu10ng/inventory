use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateTag, Tag};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Tag>>, AppError> {
    let rows = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY category_id, sort_order, id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateTag>,
) -> Result<Json<Tag>, AppError> {
    let row = sqlx::query_as::<_, Tag>(
        "INSERT INTO tags (name, category_id, sort_order) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(body.category_id)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateTag>,
) -> Result<Json<Tag>, AppError> {
    let row = sqlx::query_as::<_, Tag>(
        "UPDATE tags SET name = ?, category_id = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(body.category_id)
    .bind(body.sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tags WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
