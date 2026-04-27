use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::*;

pub async fn list(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Vec<TripItem>>, AppError> {
    let rows = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
    Json(body): Json<CreateTripItem>,
) -> Result<Json<TripItem>, AppError> {
    let row = sqlx::query_as::<_, TripItem>(
        "INSERT INTO trip_items (trip_id, item_id, custom_name, person_id, qty, checked, item_status, notes, sort_order, is_essential) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(trip_id)
    .bind(body.item_id)
    .bind(&body.custom_name)
    .bind(body.person_id)
    .bind(body.qty)
    .bind(body.checked)
    .bind(&body.item_status)
    .bind(&body.notes)
    .bind(body.sort_order)
    .bind(body.is_essential)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTripItem>,
) -> Result<Json<TripItem>, AppError> {
    let existing = sqlx::query_as::<_, TripItem>("SELECT * FROM trip_items WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let custom_name = body.custom_name.unwrap_or(existing.custom_name);
    let person_id = match body.person_id {
        Some(v) => v,
        None => existing.person_id,
    };
    let qty = body.qty.unwrap_or(existing.qty);
    let checked = body.checked.unwrap_or(existing.checked);
    let item_status = body.item_status.unwrap_or(existing.item_status);
    let notes = body.notes.unwrap_or(existing.notes);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);
    let is_essential = body.is_essential.unwrap_or(existing.is_essential);

    let row = sqlx::query_as::<_, TripItem>(
        "UPDATE trip_items SET custom_name = ?, person_id = ?, qty = ?, checked = ?, item_status = ?, notes = ?, sort_order = ?, is_essential = ? WHERE id = ? RETURNING *",
    )
    .bind(&custom_name)
    .bind(person_id)
    .bind(qty)
    .bind(checked)
    .bind(&item_status)
    .bind(&notes)
    .bind(sort_order)
    .bind(is_essential)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM trip_items WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

pub async fn bulk_update(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
    Json(body): Json<BulkUpdateTripItems>,
) -> Result<Json<Vec<TripItem>>, AppError> {
    for id in &body.ids {
        let existing = sqlx::query_as::<_, TripItem>("SELECT * FROM trip_items WHERE id = ? AND trip_id = ?")
            .bind(id)
            .bind(trip_id)
            .fetch_one(&pool)
            .await?;

        let checked = body.checked.unwrap_or(existing.checked);
        let person_id = match &body.person_id {
            Some(v) => *v,
            None => existing.person_id,
        };
        let item_status = body.item_status.clone().unwrap_or(existing.item_status);

        sqlx::query("UPDATE trip_items SET checked = ?, person_id = ?, item_status = ? WHERE id = ?")
            .bind(checked)
            .bind(person_id)
            .bind(&item_status)
            .bind(id)
            .execute(&pool)
            .await?;
    }

    let rows = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn check(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CheckBody>,
) -> Result<Json<TripItem>, AppError> {
    let row = sqlx::query_as::<_, TripItem>(
        "UPDATE trip_items SET checked = ? WHERE id = ? RETURNING *",
    )
    .bind(body.checked)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}
