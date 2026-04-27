use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreatePerson, Person};

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
    Json(body): Json<CreatePerson>,
) -> Result<Json<Person>, AppError> {
    let row = sqlx::query_as::<_, Person>(
        "UPDATE people SET name = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM people WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
