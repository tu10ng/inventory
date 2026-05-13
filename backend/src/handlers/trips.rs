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
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("行程", id))?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateTrip>,
) -> Result<Json<Trip>, AppError> {
    body.validate()?;
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
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("行程", id))?;

    let name = body.name.unwrap_or(existing.name);
    if name.trim().is_empty() {
        return Err(AppError::validation("行程名称不能为空"));
    }
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
    let result = sqlx::query("DELETE FROM trips WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("行程", id));
    }
    Ok(())
}

/// Recursively collect all slots from an activity and its included activities.
/// Returns flattened, deduplicated slots (by slot_name + category_id: first wins).
/// Has cycle detection to prevent infinite loops.
async fn collect_activity_slots(
    pool: &SqlitePool,
    activity_id: i64,
) -> Result<Vec<ActivitySlot>, AppError> {
    // BFS/DFS with visited set for cycle detection
    let mut visited = std::collections::HashSet::new();
    let mut all_slots: Vec<(i64, ActivitySlot)> = Vec::new(); // (source_activity_id, slot)
    let mut queue: Vec<i64> = vec![activity_id];

    while let Some(aid) = queue.pop() {
        if !visited.insert(aid) {
            continue; // Already visited (cycle protection)
        }

        // Collect slots from this activity
        let slots = sqlx::query_as::<_, ActivitySlot>(
            "SELECT * FROM activity_slots WHERE activity_id = ? ORDER BY sort_order, id",
        )
        .bind(aid)
        .fetch_all(pool)
        .await?;

        for slot in slots {
            all_slots.push((aid, slot));
        }

        // Collect included activities
        let includes: Vec<i64> = sqlx::query_scalar(
            "SELECT included_activity_id FROM activity_includes WHERE activity_id = ? ORDER BY sort_order, id",
        )
        .bind(aid)
        .fetch_all(pool)
        .await?;

        // Push in reverse order so they're processed in sort_order
        for inc_id in includes.into_iter().rev() {
            if !visited.contains(&inc_id) {
                queue.push(inc_id);
            }
        }
    }

    // Deduplicate: same slot_name + category_id → first one wins (from the activity higher in graph)
    let mut seen_slot_keys = std::collections::HashSet::new();
    let mut deduped = Vec::new();
    for (_src, slot) in all_slots {
        let key = (slot.slot_name.clone(), slot.category_id);
        if seen_slot_keys.insert(key) {
            deduped.push(slot);
        }
    }

    Ok(deduped)
}

pub async fn populate(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Vec<TripItem>>, AppError> {
    let trip = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("行程", trip_id))?;

    let activity_id = trip
        .activity_id
        .ok_or_else(|| AppError::bad_request("行程未关联活动模板"))?;

    let slots = collect_activity_slots(&pool, activity_id).await?;

    let mut tx = pool.begin().await?;

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
        .execute(&mut *tx)
        .await?;
    }

    let items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(items))
}

/// Detail about a single removal in resync diff.
struct RemovalDetail {
    trip_item_id: i64,
    slot_id: Option<i64>,
    custom_name: Option<String>,
    item_name: String,
    reason: String,
}

/// Diff result for resync: which trip_items to remove, which slots to add.
struct ResyncDiff {
    /// IDs of trip_items to delete
    ids_to_remove: Vec<i64>,
    /// Info about each removal (for preview)
    removals: Vec<RemovalDetail>,
    /// Slots to insert (not yet in trip)
    slots_to_add: Vec<ActivitySlot>,
}

async fn compute_resync_diff(pool: &SqlitePool, trip_id: i64) -> Result<ResyncDiff, AppError> {
    let trip = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::not_found("行程", trip_id))?;

    let activity_id = trip
        .activity_id
        .ok_or_else(|| AppError::bad_request("行程未关联活动模板"))?;

    let slots = collect_activity_slots(pool, activity_id).await?;

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

    // Batch query all item names referenced by trip_items (avoids N+1 in loop)
    let item_ids: Vec<i64> = trip_items
        .iter()
        .filter_map(|ti| ti.item_id)
        .collect();
    let item_names: std::collections::HashMap<i64, String> = if !item_ids.is_empty() {
        let placeholders = item_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("SELECT id, json_extract(attrs, '$.name') FROM items WHERE id IN ({})", placeholders);
        let mut query = sqlx::query_as::<_, (i64, String)>(&sql);
        for id in &item_ids {
            query = query.bind(id);
        }
        query.fetch_all(pool).await?.into_iter().collect()
    } else {
        std::collections::HashMap::new()
    };

    for ti in &trip_items {
        if let Some(slot_id) = ti.slot_id {
            if !template_slot_ids.contains(&slot_id) {
                // Slot removed from template
                ids_to_remove.push(ti.id);
                let item_name = ti
                    .item_id
                    .and_then(|id| item_names.get(&id))
                    .cloned()
                    .unwrap_or_default();
                removals.push(RemovalDetail {
                    trip_item_id: ti.id,
                    slot_id: ti.slot_id,
                    custom_name: if ti.custom_name.is_empty() {
                        None
                    } else {
                        Some(ti.custom_name.clone())
                    },
                    item_name,
                    reason: "模板已移除此槽位".to_string(),
                });
            } else if !seen_slot_ids.insert(slot_id) {
                // Duplicate slot_id — remove later ones
                ids_to_remove.push(ti.id);
                let item_name = ti
                    .item_id
                    .and_then(|id| item_names.get(&id))
                    .cloned()
                    .unwrap_or_default();
                removals.push(RemovalDetail {
                    trip_item_id: ti.id,
                    slot_id: ti.slot_id,
                    custom_name: if ti.custom_name.is_empty() {
                        None
                    } else {
                        Some(ti.custom_name.clone())
                    },
                    item_name,
                    reason: "与模板重复".to_string(),
                });
            }
        } else {
            // Manual item (slot_id IS NULL) — all removed on resync
            ids_to_remove.push(ti.id);
            let item_name = ti
                .item_id
                .and_then(|id| item_names.get(&id))
                .cloned()
                .unwrap_or_default();
            let custom = if ti.custom_name.is_empty() {
                None
            } else {
                Some(ti.custom_name.clone())
            };
            removals.push(RemovalDetail {
                trip_item_id: ti.id,
                slot_id: None,
                custom_name: custom,
                item_name,
                reason: "手动添加的物品".to_string(),
            });
        }
    }

    // Find slots to add: template slots not present in trip (excluding items being removed)
    let remove_set: std::collections::HashSet<i64> = ids_to_remove.iter().copied().collect();
    let existing_slot_ids: std::collections::HashSet<i64> = trip_items
        .iter()
        .filter(|ti| !remove_set.contains(&ti.id))
        .filter_map(|ti| ti.slot_id)
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

    let items_to_remove: Vec<ResyncPreviewItem> = diff.removals.iter().map(|r| {
        ResyncPreviewItem {
            trip_item_id: Some(r.trip_item_id),
            slot_name: None, // will be filled below
            item_name: if r.item_name.is_empty() { None } else { Some(r.item_name.clone()) },
            custom_name: r.custom_name.clone(),
            reason: r.reason.clone(),
        }
    }).collect();

    // Enrich slot names for removals that had a slot_id
    let mut items_to_remove = items_to_remove;
    for (i, r) in diff.removals.iter().enumerate() {
        if let Some(sid) = r.slot_id {
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

    let mut tx = pool.begin().await?;

    // Delete all items marked for removal in a single query
    if !diff.ids_to_remove.is_empty() {
        let placeholders = diff.ids_to_remove.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!("DELETE FROM trip_items WHERE id IN ({})", placeholders);
        let mut query = sqlx::query(&sql);
        for id in &diff.ids_to_remove {
            query = query.bind(id);
        }
        query.execute(&mut *tx).await?;
    }

    // Insert new slots
    let max_sort: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(sort_order), 0) FROM trip_items WHERE trip_id = ?",
    )
    .bind(trip_id)
    .fetch_one(&mut *tx)
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
        .execute(&mut *tx)
        .await?;
        sort += 1;
    }

    let items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(items))
}

pub async fn clone(
    State(pool): State<SqlitePool>,
    Path(trip_id): Path<i64>,
) -> Result<Json<Trip>, AppError> {
    let original = sqlx::query_as::<_, Trip>("SELECT * FROM trips WHERE id = ?")
        .bind(trip_id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("行程", trip_id))?;

    let new_name = format!("{} (副本)", original.name);

    let mut tx = pool.begin().await?;

    let new_trip = sqlx::query_as::<_, Trip>(
        "INSERT INTO trips (name, activity_id, start_date, end_date, notes, status) VALUES (?, ?, ?, ?, ?, 'planning') RETURNING *",
    )
    .bind(&new_name)
    .bind(original.activity_id)
    .bind(&original.start_date)
    .bind(&original.end_date)
    .bind(&original.notes)
    .fetch_one(&mut *tx)
    .await?;

    let original_items = sqlx::query_as::<_, TripItem>(
        "SELECT * FROM trip_items WHERE trip_id = ? ORDER BY sort_order, id",
    )
    .bind(trip_id)
    .fetch_all(&mut *tx)
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
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    Ok(Json(new_trip))
}
