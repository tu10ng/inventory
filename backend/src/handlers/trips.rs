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
            "INSERT OR IGNORE INTO trip_items (trip_id, item_id, qty, notes, sort_order, is_essential) VALUES (?, ?, ?, ?, ?, ?)",
        )
        .bind(trip_id)
        .bind(ai.item_id)
        .bind(ai.default_qty)
        .bind(&ai.notes)
        .bind(i as i64)
        .bind(ai.is_essential)
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

pub async fn resync(
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

    // Get existing item_ids in this trip
    let existing: Vec<i64> = sqlx::query_scalar(
        "SELECT item_id FROM trip_items WHERE trip_id = ? AND item_id IS NOT NULL",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;

    let max_sort: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), 0) FROM trip_items WHERE trip_id = ?",
    )
    .bind(trip_id)
    .fetch_one(&pool)
    .await?;

    let mut sort = max_sort + 1;
    for ai in &activity_items {
        if !existing.contains(&ai.item_id) {
            sqlx::query(
                "INSERT INTO trip_items (trip_id, item_id, qty, notes, sort_order, is_essential) VALUES (?, ?, ?, ?, ?, ?)",
            )
            .bind(trip_id)
            .bind(ai.item_id)
            .bind(ai.default_qty)
            .bind(&ai.notes)
            .bind(sort)
            .bind(ai.is_essential)
            .execute(&pool)
            .await?;
            sort += 1;
        }
    }

    let items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(items))
}

pub async fn clone(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Trip>, AppError> {
    let original = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_one(&pool)
        .await?;

    let new_name = format!("{} (副本)", original.name);
    let new_trip = sqlx::query_as::<_, Trip>(
        "INSERT INTO trips (name, activity_id, start_date, end_date, notes, status) VALUES (?, ?, ?, ?, ?, 'planning') RETURNING *",
    )
    .bind(&new_name)
    .bind(original.activity_id)
    .bind(&original.start_date)
    .bind(&original.end_date)
    .bind(&original.notes)
    .fetch_one(&pool)
    .await?;

    let original_items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;

    for ti in &original_items {
        sqlx::query(
            "INSERT INTO trip_items (trip_id, item_id, custom_name, person_id, qty, checked, item_status, notes, sort_order, is_essential) VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?, ?)",
        )
        .bind(new_trip.id)
        .bind(ti.item_id)
        .bind(&ti.custom_name)
        .bind(ti.person_id)
        .bind(ti.qty)
        .bind(&ti.item_status)
        .bind(&ti.notes)
        .bind(ti.sort_order)
        .bind(ti.is_essential)
        .execute(&pool)
        .await?;
    }

    Ok(Json(new_trip))
}
