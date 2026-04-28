use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::*;

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Activity>>, AppError> {
    let rows = sqlx::query_as::<_, Activity>("SELECT * FROM activities ORDER BY id")
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateActivity>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>(
        "INSERT INTO activities (name, description, icon) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.icon)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<CreateActivity>,
) -> Result<Json<Activity>, AppError> {
    let row = sqlx::query_as::<_, Activity>(
        "UPDATE activities SET name = ?, description = ?, icon = ? WHERE id = ? RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.description)
    .bind(&body.icon)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

// ── Activity Items (legacy, kept for backward compat) ──

pub async fn list_items(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<ActivityItem>>, AppError> {
    let rows = sqlx::query_as::<_, ActivityItem>(
        "SELECT * FROM activity_items WHERE activity_id = ? ORDER BY id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn add_item(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateActivityItem>,
) -> Result<Json<ActivityItem>, AppError> {
    let row = sqlx::query_as::<_, ActivityItem>(
        "INSERT INTO activity_items (activity_id, item_id, is_essential, default_qty, notes) VALUES (?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(body.item_id)
    .bind(body.is_essential)
    .bind(body.default_qty)
    .bind(&body.notes)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn remove_item(
    State(pool): State<SqlitePool>,
    Path((activity_id, item_id)): Path<(i64, i64)>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM activity_items WHERE activity_id = ? AND item_id = ?")
        .bind(activity_id)
        .bind(item_id)
        .execute(&pool)
        .await?;
    Ok(())
}

// ── Activity Slots ──

pub async fn list_slots(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<ActivitySlotWithTags>>, AppError> {
    let slots = sqlx::query_as::<_, ActivitySlot>(
        "SELECT * FROM activity_slots WHERE activity_id = ? ORDER BY sort_order, id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;

    let slot_ids: Vec<i64> = slots.iter().map(|s| s.id).collect();
    let mut result = Vec::with_capacity(slots.len());

    if slot_ids.is_empty() {
        return Ok(Json(result));
    }

    // Batch query all slot_tags + tags
    let placeholders = slot_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
    let query_str = format!(
        "SELECT ast.slot_id, t.id, t.name, t.category_id, t.sort_order FROM activity_slot_tags ast JOIN tags t ON t.id = ast.tag_id WHERE ast.slot_id IN ({}) ORDER BY t.sort_order, t.id",
        placeholders
    );
    let mut query = sqlx::query_as::<_, SlotTagRow>(&query_str);
    for id in &slot_ids {
        query = query.bind(id);
    }
    let tag_rows = query.fetch_all(&pool).await?;

    // Group tags by slot_id
    let mut tag_map: std::collections::HashMap<i64, Vec<Tag>> = std::collections::HashMap::new();
    for row in tag_rows {
        tag_map.entry(row.slot_id).or_default().push(Tag {
            id: row.id,
            name: row.name,
            category_id: row.category_id,
            sort_order: row.sort_order,
        });
    }

    for slot in slots {
        let tags = tag_map.remove(&slot.id).unwrap_or_default();
        result.push(ActivitySlotWithTags {
            id: slot.id,
            activity_id: slot.activity_id,
            slot_name: slot.slot_name,
            category_id: slot.category_id,
            is_essential: slot.is_essential,
            default_qty: slot.default_qty,
            default_item_id: slot.default_item_id,
            notes: slot.notes,
            sort_order: slot.sort_order,
            tags,
        });
    }

    Ok(Json(result))
}

#[derive(Debug, sqlx::FromRow)]
struct SlotTagRow {
    slot_id: i64,
    id: i64,
    name: String,
    category_id: i64,
    sort_order: i64,
}

pub async fn create_slot(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateActivitySlot>,
) -> Result<Json<ActivitySlotWithTags>, AppError> {
    let slot = sqlx::query_as::<_, ActivitySlot>(
        "INSERT INTO activity_slots (activity_id, slot_name, category_id, is_essential, default_qty, default_item_id, notes, sort_order) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(&body.slot_name)
    .bind(body.category_id)
    .bind(body.is_essential)
    .bind(body.default_qty)
    .bind(body.default_item_id)
    .bind(&body.notes)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;

    let mut tags = Vec::new();
    for tag_id in &body.tag_ids {
        sqlx::query("INSERT INTO activity_slot_tags (slot_id, tag_id) VALUES (?, ?)")
            .bind(slot.id)
            .bind(tag_id)
            .execute(&pool)
            .await?;
        let tag = sqlx::query_as::<_, Tag>("SELECT * FROM tags WHERE id = ?")
            .bind(tag_id)
            .fetch_one(&pool)
            .await?;
        tags.push(tag);
    }

    Ok(Json(ActivitySlotWithTags {
        id: slot.id,
        activity_id: slot.activity_id,
        slot_name: slot.slot_name,
        category_id: slot.category_id,
        is_essential: slot.is_essential,
        default_qty: slot.default_qty,
        default_item_id: slot.default_item_id,
        notes: slot.notes,
        sort_order: slot.sort_order,
        tags,
    }))
}

pub async fn update_slot(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateActivitySlot>,
) -> Result<Json<ActivitySlotWithTags>, AppError> {
    let existing = sqlx::query_as::<_, ActivitySlot>("SELECT * FROM activity_slots WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let slot_name = body.slot_name.unwrap_or(existing.slot_name);
    let category_id = body.category_id.unwrap_or(existing.category_id);
    let is_essential = body.is_essential.unwrap_or(existing.is_essential);
    let default_qty = body.default_qty.unwrap_or(existing.default_qty);
    let default_item_id = match body.default_item_id {
        Some(v) => v,
        None => existing.default_item_id,
    };
    let notes = body.notes.unwrap_or(existing.notes);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    let slot = sqlx::query_as::<_, ActivitySlot>(
        "UPDATE activity_slots SET slot_name = ?, category_id = ?, is_essential = ?, default_qty = ?, default_item_id = ?, notes = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&slot_name)
    .bind(category_id)
    .bind(is_essential)
    .bind(default_qty)
    .bind(default_item_id)
    .bind(&notes)
    .bind(sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    // Rebuild tag associations if provided
    if let Some(tag_ids) = &body.tag_ids {
        sqlx::query("DELETE FROM activity_slot_tags WHERE slot_id = ?")
            .bind(id)
            .execute(&pool)
            .await?;
        for tag_id in tag_ids {
            sqlx::query("INSERT INTO activity_slot_tags (slot_id, tag_id) VALUES (?, ?)")
                .bind(id)
                .bind(tag_id)
                .execute(&pool)
                .await?;
        }
    }

    // Fetch tags
    let tags = sqlx::query_as::<_, Tag>(
        "SELECT t.* FROM tags t JOIN activity_slot_tags ast ON ast.tag_id = t.id WHERE ast.slot_id = ? ORDER BY t.sort_order, t.id",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(ActivitySlotWithTags {
        id: slot.id,
        activity_id: slot.activity_id,
        slot_name: slot.slot_name,
        category_id: slot.category_id,
        is_essential: slot.is_essential,
        default_qty: slot.default_qty,
        default_item_id: slot.default_item_id,
        notes: slot.notes,
        sort_order: slot.sort_order,
        tags,
    }))
}

pub async fn delete_slot(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM activity_slots WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}

// ── Tips ──

pub async fn list_tips(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<Tip>>, AppError> {
    let rows = sqlx::query_as::<_, Tip>(
        "SELECT * FROM tips WHERE activity_id = ? ORDER BY sort_order, id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_tip(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateTip>,
) -> Result<Json<Tip>, AppError> {
    let row = sqlx::query_as::<_, Tip>(
        "INSERT INTO tips (activity_id, content, sort_order) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(&body.content)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update_tip(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateTip>,
) -> Result<Json<Tip>, AppError> {
    let existing = sqlx::query_as::<_, Tip>("SELECT * FROM tips WHERE id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    let content = body.content.unwrap_or(existing.content);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    let row = sqlx::query_as::<_, Tip>(
        "UPDATE tips SET content = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&content)
    .bind(sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_tip(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    sqlx::query("DELETE FROM tips WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    Ok(())
}
