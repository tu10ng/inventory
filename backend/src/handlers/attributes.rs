use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{AttributeDefinition, CreateAttributeDefinition};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<AttributeDefinition>>, AppError> {
    let rows = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateAttributeDefinition>,
) -> Result<Json<AttributeDefinition>, AppError> {
    let row = sqlx::query_as::<_, AttributeDefinition>(
        "INSERT INTO attribute_definitions (key, label, attr_type, config, category_scope, sort_order) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.key)
    .bind(&body.label)
    .bind(&body.attr_type)
    .bind(&body.config)
    .bind(&body.category_scope)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateAttributeDefinition>,
) -> Result<Json<AttributeDefinition>, AppError> {
    let row = sqlx::query_as::<_, AttributeDefinition>(
        "UPDATE attribute_definitions SET key = ?, label = ?, attr_type = ?, config = ?, category_scope = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.key)
    .bind(&body.label)
    .bind(&body.attr_type)
    .bind(&body.config)
    .bind(&body.category_scope)
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
    sqlx::query("DELETE FROM attribute_definitions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
