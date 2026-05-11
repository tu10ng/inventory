use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{Category, CreateCategory};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Category>>, AppError> {
    let rows = sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order, id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateCategory>,
) -> Result<Json<Category>, AppError> {
    body.validate()?;
    let row = sqlx::query_as::<_, Category>(
        "INSERT INTO categories (name, icon, sort_order) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.icon)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateCategory>,
) -> Result<Json<Category>, AppError> {
    body.validate()?;
    let row = sqlx::query_as::<_, Category>(
        "UPDATE categories SET name = ?, icon = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.icon)
    .bind(body.sort_order)
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("分类", id))?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM categories WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("分类", id));
    }
    Ok(())
}
