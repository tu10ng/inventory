use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::*;

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Trip>>, AppError> {
    let rows = sqlx::query_as::<_, Trip>("SELECT * FROM trips ORDER BY id DESC")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Trip>, AppError> {
    let row = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateTrip>,
) -> Result<Json<Trip>, AppError> {
    let row = sqlx::query_as::<_, Trip>(
        "INSERT INTO trips (name, activity_id, start_date, end_date, notes, status) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(body.activity_id)
    .bind(&body.start_date)
    .bind(&body.end_date)
    .bind(&body.notes)
    .bind(&body.status)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTrip>,
) -> Result<Json<Trip>, AppError> {
    let existing = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let name = body.name.unwrap_or(existing.name);
    let activity_id = body.activity_id.or(existing.activity_id);
    let start_date = body.start_date.unwrap_or(existing.start_date);
    let end_date = body.end_date.unwrap_or(existing.end_date);
    let notes = body.notes.unwrap_or(existing.notes);
    let status = body.status.unwrap_or(existing.status);

    let row = sqlx::query_as::<_, Trip>(
        "UPDATE trips SET name = ?, activity_id = ?, start_date = ?, end_date = ?, notes = ?, status = ? WHERE id = ? RETURNING *",
    )
    .bind(&name)
    .bind(activity_id)
    .bind(&start_date)
    .bind(&end_date)
    .bind(&notes)
    .bind(&status)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM trips WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn populate(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Vec<TripItem>>, AppError> {
    let trip = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_one(&pool)
        .await?;

    let activity_id = trip
        .activity_id
        .ok_or_else(|| anyhow::anyhow!("Trip has no activity"))?;

    let activity_items = sqlx::query_as::<_, ActivityItem>(
        "SELECT * FROM activity_items WHERE activity_id = ? ORDER BY id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;

    for (i, ai) in activity_items.iter().enumerate() {
        sqlx::query(
            "INSERT OR IGNORE INTO trip_items (trip_id, item_id, qty, notes, sort_order) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(trip_id)
        .bind(ai.item_id)
        .bind(ai.default_qty)
        .bind(&ai.notes)
        .bind(i as i64)
        .execute(&pool)
        .await?;
    }

    let items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(items))
}
