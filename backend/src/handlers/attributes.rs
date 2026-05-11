use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{AttributeDefinition, CreateAttributeDefinition, UpdateAttributeDefinition};

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
    if body.key.trim().is_empty() {
        return Err(AppError::validation("属性键不能为空"));
    }
    if body.label.trim().is_empty() {
        return Err(AppError::validation("属性标签不能为空"));
    }
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
    Json(body): Json<UpdateAttributeDefinition>,
) -> Result<Json<AttributeDefinition>, AppError> {
    let existing = sqlx::query_as::<_, AttributeDefinition>("SELECT * FROM attribute_definitions WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("属性定义", id))?;

    let key = body.key.unwrap_or(existing.key);
    let label = body.label.unwrap_or(existing.label);
    let attr_type = body.attr_type.unwrap_or(existing.attr_type);
    let config = body.config.unwrap_or(existing.config);
    let category_scope = body.category_scope.unwrap_or(existing.category_scope);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    if key.trim().is_empty() {
        return Err(AppError::validation("属性键不能为空"));
    }
    if label.trim().is_empty() {
        return Err(AppError::validation("属性标签不能为空"));
    }

    let row = sqlx::query_as::<_, AttributeDefinition>(
        "UPDATE attribute_definitions SET key = ?, label = ?, attr_type = ?, config = ?, category_scope = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&key)
    .bind(&label)
    .bind(&attr_type)
    .bind(&config)
    .bind(&category_scope)
    .bind(sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM attribute_definitions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("属性定义", id));
    }
    Ok(())
}
