use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateStatusDefinition, StatusDefinition, UpdateStatusDefinition};

#[derive(Debug, serde::Deserialize)]
pub struct StatusQuery {
    pub scope: Option<String>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
    Query(query): Query<StatusQuery>,
) -> Result<Json<Vec<StatusDefinition>>, AppError> {
    let rows = if let Some(scope) = &query.scope {
        sqlx::query_as::<_, StatusDefinition>(
            "SELECT * FROM status_definitions WHERE scope = ? ORDER BY sort_order, id",
        )
        .bind(scope)
        .fetch_all(&pool)
        .await?
    } else {
        sqlx::query_as::<_, StatusDefinition>(
            "SELECT * FROM status_definitions ORDER BY scope, sort_order, id",
        )
        .fetch_all(&pool)
        .await?
    };
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateStatusDefinition>,
) -> Result<Json<StatusDefinition>, AppError> {
    if body.value.trim().is_empty() {
        return Err(AppError::validation("状态值不能为空"));
    }
    if body.label.trim().is_empty() {
        return Err(AppError::validation("状态标签不能为空"));
    }
    let row = sqlx::query_as::<_, StatusDefinition>(
        "INSERT INTO status_definitions (scope, value, label, color, icon, sort_order) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.scope)
    .bind(&body.value)
    .bind(&body.label)
    .bind(&body.color)
    .bind(&body.icon)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateStatusDefinition>,
) -> Result<Json<StatusDefinition>, AppError> {
    let existing = sqlx::query_as::<_, StatusDefinition>("SELECT * FROM status_definitions WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("状态定义", id))?;

    let scope = body.scope.unwrap_or(existing.scope);
    let value = body.value.unwrap_or(existing.value);
    let label = body.label.unwrap_or(existing.label);
    let color = body.color.unwrap_or(existing.color);
    let icon = body.icon.unwrap_or(existing.icon);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    if value.trim().is_empty() {
        return Err(AppError::validation("状态值不能为空"));
    }
    if label.trim().is_empty() {
        return Err(AppError::validation("状态标签不能为空"));
    }

    let row = sqlx::query_as::<_, StatusDefinition>(
        "UPDATE status_definitions SET scope = ?, value = ?, label = ?, color = ?, icon = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&scope)
    .bind(&value)
    .bind(&label)
    .bind(&color)
    .bind(&icon)
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
    let result = sqlx::query("DELETE FROM status_definitions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("状态定义", id));
    }
    Ok(())
}
