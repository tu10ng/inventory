use axum::extract::{Path, Query, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateStatusDefinition, StatusDefinition};

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
    Json(body): Json<CreateStatusDefinition>,
) -> Result<Json<StatusDefinition>, AppError> {
    let row = sqlx::query_as::<_, StatusDefinition>(
        "UPDATE status_definitions SET scope = ?, value = ?, label = ?, color = ?, icon = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.scope)
    .bind(&body.value)
    .bind(&body.label)
    .bind(&body.color)
    .bind(&body.icon)
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
    sqlx::query("DELETE FROM status_definitions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
