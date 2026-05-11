use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreatePerson, Person, UpdatePerson};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Person>>, AppError> {
    let rows = sqlx::query_as::<_, Person>("SELECT * FROM people ORDER BY id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreatePerson>,
) -> Result<Json<Person>, AppError> {
    body.validate()?;
    let row = sqlx::query_as::<_, Person>(
        "INSERT INTO people (name) VALUES (?) RETURNING *",
    )
    .bind(&body.name)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdatePerson>,
) -> Result<Json<Person>, AppError> {
    let existing = sqlx::query_as::<_, Person>("SELECT * FROM people WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("人员", id))?;

    let name = body.name.unwrap_or(existing.name);

    if name.trim().is_empty() {
        return Err(AppError::validation("人员名称不能为空"));
    }

    let row = sqlx::query_as::<_, Person>(
        "UPDATE people SET name = ? WHERE id = ? RETURNING *",
    )
    .bind(&name)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM people WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("人员", id));
    }
    Ok(())
}
