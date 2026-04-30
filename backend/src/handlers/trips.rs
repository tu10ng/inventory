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

    // Use activity_slots instead of activity_items
    let slots = sqlx::query_as::<_, ActivitySlot>(
        "SELECT * FROM activity_slots WHERE activity_id = ? ORDER BY sort_order, id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;

    for (i, slot) in slots.iter().enumerate() {
        sqlx::query(
            "INSERT OR IGNORE INTO trip_items (trip_id, item_id, qty, notes, sort_order, is_essential, slot_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(trip_id)
        .bind(None::<i64>)
        .bind(slot.default_qty)
        .bind(&slot.notes)
        .bind(i as i64)
        .bind(slot.is_essential)
        .bind(slot.id)
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

/// Diff result for resync: which trip_items to remove, which slots to add.
struct ResyncDiff {
    /// IDs of trip_items to delete
    ids_to_remove: Vec<i64>,
    /// Info about each removal (for preview)
    removals: Vec<(i64, Option<i64>, Option<String>, String, String)>, // (trip_item_id, slot_id, custom_name, item_name, reason)
    /// Slots to insert (not yet in trip)
    slots_to_add: Vec<ActivitySlot>,
}

async fn compute_resync_diff(pool: &SqlitePool, trip_id: i64) -> Result<ResyncDiff, AppError> {
    let trip = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_one(pool)
        .await?;

    let activity_id = trip
        .activity_id
        .ok_or_else(|| anyhow::anyhow!("Trip has no activity"))?;

    let slots = sqlx::query_as::<_, ActivitySlot>(
        "SELECT * FROM activity_slots WHERE activity_id = ? ORDER BY sort_order, id",
    )
    .bind(activity_id)
    .fetch_all(pool)
    .await?;

    let trip_items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY id",
    )
    .bind(trip_id)
    .fetch_all(pool)
    .await?;

    let template_slot_ids: std::collections::HashSet<i64> = slots.iter().map(|s| s.id).collect();

    let mut ids_to_remove = Vec::new();
    let mut removals = Vec::new();

    // Track first occurrence of each slot_id for dedup
    let mut seen_slot_ids = std::collections::HashSet::new();

    for ti in &trip_items {
        if let Some(slot_id) = ti.slot_id {
            if !template_slot_ids.contains(&slot_id) {
                // Slot removed from template
                ids_to_remove.push(ti.id);
                let item_name = if let Some(item_id) = ti.item_id {
                    sqlx::query_scalar::<_, String>("SELECT name FROM items WHERE id = ?")
                        .bind(item_id)
                        .fetch_optional(pool)
                        .await?
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                removals.push((ti.id, ti.slot_id, ti.custom_name.clone().into(), item_name, "模板已移除此槽位".to_string()));
            } else if !seen_slot_ids.insert(slot_id) {
                // Duplicate slot_id — remove later ones
                ids_to_remove.push(ti.id);
                let item_name = if let Some(item_id) = ti.item_id {
                    sqlx::query_scalar::<_, String>("SELECT name FROM items WHERE id = ?")
                        .bind(item_id)
                        .fetch_optional(pool)
                        .await?
                        .unwrap_or_default()
                } else {
                    String::new()
                };
                removals.push((ti.id, ti.slot_id, ti.custom_name.clone().into(), item_name, "与模板重复".to_string()));
            }
        } else {
            // Manual item (slot_id IS NULL) — all removed on resync
            ids_to_remove.push(ti.id);
            let item_name = if let Some(item_id) = ti.item_id {
                sqlx::query_scalar::<_, String>("SELECT name FROM items WHERE id = ?")
                    .bind(item_id)
                    .fetch_optional(pool)
                    .await?
                    .unwrap_or_default()
            } else {
                String::new()
            };
            let custom = if ti.custom_name.is_empty() { None } else { Some(ti.custom_name.clone()) };
            removals.push((ti.id, None, custom, item_name, "手动添加的物品".to_string()));
        }
    }

    // Find slots to add: template slots not present in trip
    let existing_slot_ids: std::collections::HashSet<i64> = trip_items
        .iter()
        .filter_map(|ti| ti.slot_id)
        .filter(|sid| !ids_to_remove.iter().any(|rid| {
            trip_items.iter().any(|ti| ti.id == *rid && ti.slot_id == Some(*sid))
        }))
        .collect();

    let slots_to_add: Vec<ActivitySlot> = slots
        .into_iter()
        .filter(|s| !existing_slot_ids.contains(&s.id))
        .collect();

    Ok(ResyncDiff {
        ids_to_remove,
        removals,
        slots_to_add,
    })
}

pub async fn resync_preview(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<ResyncPreview>, AppError> {
    let diff = compute_resync_diff(&pool, trip_id).await?;

    let items_to_remove: Vec<ResyncPreviewItem> = diff.removals.iter().map(|(id, _slot_id, custom_name, item_name, reason)| {
        // Try to get slot_name if it had a slot_id
        let slot_name_str = None; // will be filled below
        ResyncPreviewItem {
            trip_item_id: Some(*id),
            slot_name: slot_name_str,
            item_name: if item_name.is_empty() { None } else { Some(item_name.clone()) },
            custom_name: custom_name.clone(),
            reason: reason.clone(),
        }
    }).collect();

    // Enrich slot names for removals that had a slot_id
    let mut items_to_remove = items_to_remove;
    for (i, (_id, slot_id, _custom, _item_name, _reason)) in diff.removals.iter().enumerate() {
        if let Some(sid) = slot_id {
            let slot_name = sqlx::query_scalar::<_, String>("SELECT slot_name FROM activity_slots WHERE id = ?")
                .bind(sid)
                .fetch_optional(&pool)
                .await?;
            items_to_remove[i].slot_name = slot_name;
        }
    }

    let mut items_to_add = Vec::new();
    for slot in &diff.slots_to_add {
        let item_name: Option<String> = None;
        items_to_add.push(ResyncPreviewItem {
            trip_item_id: None,
            slot_name: Some(slot.slot_name.clone()),
            item_name,
            custom_name: None,
            reason: "新增槽位".to_string(),
        });
    }

    Ok(Json(ResyncPreview {
        items_to_remove,
        items_to_add,
    }))
}

pub async fn resync(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Vec<TripItem>>, AppError> {
    let diff = compute_resync_diff(&pool, trip_id).await?;

    // Delete all items marked for removal in a single query
    if !diff.ids_to_remove.is_empty() {
        let placeholders = diff.ids_to_remove.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM trip_items WHERE id IN ({})", placeholders);
        let mut query = sqlx::query(&sql);
        for id in &diff.ids_to_remove {
            query = query.bind(id);
        }
        query.execute(&pool).await?;
    }

    // Insert new slots
    let max_sort: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), 0) FROM trip_items WHERE trip_id = ?",
    )
    .bind(trip_id)
    .fetch_one(&pool)
    .await?;

    let mut sort = max_sort + 1;
    for slot in &diff.slots_to_add {
        sqlx::query(
            "INSERT INTO trip_items (trip_id, item_id, qty, notes, sort_order, is_essential, slot_id) VALUES (?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(trip_id)
        .bind(None::<i64>)
        .bind(slot.default_qty)
        .bind(&slot.notes)
        .bind(sort)
        .bind(slot.is_essential)
        .bind(slot.id)
        .execute(&pool)
        .await?;
        sort += 1;
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
            "INSERT INTO trip_items (trip_id, item_id, custom_name, person_id, qty, checked, item_status, notes, sort_order, is_essential, slot_id) VALUES (?, ?, ?, ?, ?, 0, ?, ?, ?, ?, ?)",
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
        .bind(ti.slot_id)
        .execute(&pool)
        .await?;
    }

    Ok(Json(new_trip))
}
