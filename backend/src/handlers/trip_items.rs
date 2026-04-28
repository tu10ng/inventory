use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;
use std::collections::HashMap;

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

pub async fn list_enriched(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Vec<TripItemEnriched>>, AppError> {
    // 1. Fetch trip items
    let trip_items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&pool)
    .await?;

    // 2. Collect unique slot_ids
    let slot_ids: Vec<i64> = trip_items
        .iter()
        .filter_map(|ti| ti.slot_id)
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    // 3. Batch fetch slots
    let mut slot_map: HashMap<i64, ActivitySlot> = HashMap::new();
    if !slot_ids.is_empty() {
        let placeholders = slot_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!("SELECT * FROM activity_slots WHERE id IN ({})", placeholders);
        let mut query = sqlx::query_as::<_, ActivitySlot>(&q);
        for id in &slot_ids {
            query = query.bind(id);
        }
        let slots = query.fetch_all(&pool).await?;
        for s in slots {
            slot_map.insert(s.id, s);
        }
    }

    // 4. Batch fetch slot_tags → tag_ids per slot
    let mut slot_tag_ids: HashMap<i64, Vec<i64>> = HashMap::new();
    if !slot_ids.is_empty() {
        let placeholders = slot_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!(
            "SELECT slot_id, tag_id FROM activity_slot_tags WHERE slot_id IN ({})",
            placeholders
        );
        let mut query = sqlx::query_as::<_, SlotTagIdRow>(&q);
        for id in &slot_ids {
            query = query.bind(id);
        }
        let rows = query.fetch_all(&pool).await?;
        for row in rows {
            slot_tag_ids.entry(row.slot_id).or_default().push(row.tag_id);
        }
    }

    // 5. Collect all unique tag_ids and batch fetch candidate items
    let all_tag_ids: Vec<i64> = slot_tag_ids
        .values()
        .flatten()
        .copied()
        .collect::<std::collections::HashSet<_>>()
        .into_iter()
        .collect();

    let mut items_by_tag: HashMap<i64, Vec<Item>> = HashMap::new();
    if !all_tag_ids.is_empty() {
        let placeholders = all_tag_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let q = format!(
            "SELECT * FROM items WHERE tag_id IN ({}) ORDER BY name",
            placeholders
        );
        let mut query = sqlx::query_as::<_, Item>(&q);
        for id in &all_tag_ids {
            query = query.bind(id);
        }
        let items = query.fetch_all(&pool).await?;
        for item in items {
            if let Some(tag_id) = item.tag_id {
                items_by_tag.entry(tag_id).or_default().push(item);
            }
        }
    }

    // 6. Assemble enriched items
    let mut result = Vec::with_capacity(trip_items.len());
    for ti in trip_items {
        let (slot, candidates) = if let Some(sid) = ti.slot_id {
            let slot_info = slot_map.get(&sid).map(|s| SlotInfo {
                id: s.id,
                slot_name: s.slot_name.clone(),
                category_id: s.category_id,
                is_essential: s.is_essential,
                default_item_id: s.default_item_id,
            });

            // Gather candidates from all tags of this slot
            let mut candidates: Vec<Item> = Vec::new();
            let mut seen_ids = std::collections::HashSet::new();
            if let Some(tag_ids) = slot_tag_ids.get(&sid) {
                for tag_id in tag_ids {
                    if let Some(items) = items_by_tag.get(tag_id) {
                        for item in items {
                            if seen_ids.insert(item.id) {
                                candidates.push(Item {
                                    id: item.id,
                                    name: item.name.clone(),
                                    brand: item.brand.clone(),
                                    model: item.model.clone(),
                                    category_id: item.category_id,
                                    default_qty: item.default_qty,
                                    notes: item.notes.clone(),
                                    tag_id: item.tag_id,
                                });
                            }
                        }
                    }
                }
            }

            (slot_info, candidates)
        } else {
            (None, Vec::new())
        };

        result.push(TripItemEnriched {
            id: ti.id,
            trip_id: ti.trip_id,
            item_id: ti.item_id,
            custom_name: ti.custom_name,
            person_id: ti.person_id,
            qty: ti.qty,
            checked: ti.checked,
            item_status: ti.item_status,
            notes: ti.notes,
            sort_order: ti.sort_order,
            is_essential: ti.is_essential,
            slot_id: ti.slot_id,
            slot,
            candidates,
        });
    }

    Ok(Json(result))
}

#[derive(Debug, sqlx::FromRow)]
struct SlotTagIdRow {
    slot_id: i64,
    tag_id: i64,
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
    Json(body): Json<CreateTripItem>,
) -> Result<Json<TripItem>, AppError> {
    let row = sqlx::query_as::<_, TripItem>(
        "INSERT INTO trip_items (trip_id, item_id, custom_name, person_id, qty, checked, item_status, notes, sort_order, is_essential, slot_id) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
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
    .bind(body.slot_id)
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

    let item_id = body.item_id.or(existing.item_id);
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
        "UPDATE trip_items SET item_id = ?, custom_name = ?, person_id = ?, qty = ?, checked = ?, item_status = ?, notes = ?, sort_order = ?, is_essential = ? WHERE id = ? RETURNING *",
    )
    .bind(item_id)
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
