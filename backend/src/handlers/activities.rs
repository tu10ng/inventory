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
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("活动", id))?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateActivity>,
) -> Result<Json<Activity>, AppError> {
    body.validate()?;
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
    Json(body): Json<UpdateActivity>,
) -> Result<Json<Activity>, AppError> {
    let existing = sqlx::query_as::<_, Activity>("SELECT * FROM activities WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("活动", id))?;

    let name = body.name.unwrap_or(existing.name);
    let description = body.description.unwrap_or(existing.description);
    let icon = body.icon.unwrap_or(existing.icon);

    if name.trim().is_empty() {
        return Err(AppError::validation("活动名称不能为空"));
    }

    let row = sqlx::query_as::<_, Activity>(
        "UPDATE activities SET name = ?, description = ?, icon = ? WHERE id = ? RETURNING *",
    )
    .bind(&name)
    .bind(&description)
    .bind(&icon)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM activities WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("活动", id));
    }
    Ok(())
}

// ── Activity Slots ──

pub async fn list_slots(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<ActivitySlotWithTypes>>, AppError> {
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
        "SELECT ast.slot_id, t.id, t.name, t.sort_order FROM activity_slot_types ast JOIN types t ON t.id = ast.type_id WHERE ast.slot_id IN ({}) ORDER BY t.sort_order, t.id",
        placeholders
    );
    let mut query = sqlx::query_as::<_, SlotTagRow>(&query_str);
    for id in &slot_ids {
        query = query.bind(id);
    }
    let tag_rows = query.fetch_all(&pool).await?;

    // Group types by slot_id
    let mut type_map: std::collections::HashMap<i64, Vec<Type>> = std::collections::HashMap::new();
    for row in tag_rows {
        type_map.entry(row.slot_id).or_default().push(Type {
            id: row.id,
            name: row.name,
            sort_order: row.sort_order,
            parent_id: None,
        });
    }

    for slot in slots {
        let types = type_map.remove(&slot.id).unwrap_or_default();
        result.push(ActivitySlotWithTypes {
            id: slot.id,
            activity_id: slot.activity_id,
            slot_name: slot.slot_name,
            is_essential: slot.is_essential,
            default_qty: slot.default_qty,
            notes: slot.notes,
            sort_order: slot.sort_order,
            default_item_id: slot.default_item_id,
            types,
        });
    }

    Ok(Json(result))
}

#[derive(Debug, sqlx::FromRow)]
struct SlotTagRow {
    slot_id: i64,
    id: i64,
    name: String,
    sort_order: i64,
}

pub async fn create_slot(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateActivitySlot>,
) -> Result<Json<ActivitySlotWithTypes>, AppError> {
    body.validate()?;

    let mut tx = pool.begin().await?;

    let slot = sqlx::query_as::<_, ActivitySlot>(
        "INSERT INTO activity_slots (activity_id, slot_name, is_essential, default_qty, notes, sort_order, default_item_id) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, activity_id, slot_name, is_essential, default_qty, notes, sort_order, default_item_id",
    )
    .bind(activity_id)
    .bind(&body.slot_name)
    .bind(body.is_essential)
    .bind(body.default_qty)
    .bind(&body.notes)
    .bind(body.sort_order)
    .bind(body.default_item_id)
    .fetch_one(&mut *tx)
    .await?;

    let mut types = Vec::new();
    for type_id in &body.type_ids {
        sqlx::query("INSERT INTO activity_slot_types (slot_id, type_id) VALUES (?, ?)")
            .bind(slot.id)
            .bind(type_id)
            .execute(&mut *tx)
            .await?;
        let t = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types WHERE id = ?")
            .bind(type_id)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::not_found("类型", *type_id))?;
        types.push(t);
    }

    tx.commit().await?;

    Ok(Json(ActivitySlotWithTypes {
        id: slot.id,
        activity_id: slot.activity_id,
        slot_name: slot.slot_name,
        is_essential: slot.is_essential,
        default_qty: slot.default_qty,
        notes: slot.notes,
        sort_order: slot.sort_order,
        default_item_id: slot.default_item_id,
        types,
    }))
}

pub async fn update_slot(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateActivitySlot>,
) -> Result<Json<ActivitySlotWithTypes>, AppError> {
    let existing = sqlx::query_as::<_, ActivitySlot>("SELECT * FROM activity_slots WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("槽位", id))?;

    let slot_name = body.slot_name.unwrap_or(existing.slot_name);
    let is_essential = body.is_essential.unwrap_or(existing.is_essential);
    let default_qty = body.default_qty.unwrap_or(existing.default_qty);
    let notes = body.notes.unwrap_or(existing.notes);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);
    let default_item_id = match body.default_item_id {
        Some(v) => v,
        None => existing.default_item_id,
    };

    let mut tx = pool.begin().await?;

    let slot = sqlx::query_as::<_, ActivitySlot>(
        "UPDATE activity_slots SET slot_name = ?, is_essential = ?, default_qty = ?, notes = ?, sort_order = ?, default_item_id = ? WHERE id = ? RETURNING id, activity_id, slot_name, is_essential, default_qty, notes, sort_order, default_item_id",
    )
    .bind(&slot_name)
    .bind(is_essential)
    .bind(default_qty)
    .bind(&notes)
    .bind(sort_order)
    .bind(default_item_id)
    .bind(id)
    .fetch_one(&mut *tx)
    .await?;

    // Rebuild tag associations if provided
    if let Some(type_ids) = &body.type_ids {
        sqlx::query("DELETE FROM activity_slot_types WHERE slot_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;
        for type_id in type_ids {
            sqlx::query("INSERT INTO activity_slot_types (slot_id, type_id) VALUES (?, ?)")
                .bind(id)
                .bind(type_id)
                .execute(&mut *tx)
                .await?;
        }
    }

    // Fetch types
    let types = sqlx::query_as::<_, Type>(
        "SELECT t.id, t.name, t.sort_order, t.parent_id FROM types t JOIN activity_slot_types ast ON ast.type_id = t.id WHERE ast.slot_id = ? ORDER BY t.sort_order, t.id",
    )
    .bind(id)
    .fetch_all(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(Json(ActivitySlotWithTypes {
        id: slot.id,
        activity_id: slot.activity_id,
        slot_name: slot.slot_name,
        is_essential: slot.is_essential,
        default_qty: slot.default_qty,
        notes: slot.notes,
        sort_order: slot.sort_order,
        default_item_id: slot.default_item_id,
        types,
    }))
}

pub async fn delete_slot(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM activity_slots WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("槽位", id));
    }
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
    if body.content.trim().is_empty() {
        return Err(AppError::validation("提示内容不能为空"));
    }
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
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("提示", id))?;

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
    let result = sqlx::query("DELETE FROM tips WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("提示", id));
    }
    Ok(())
}
