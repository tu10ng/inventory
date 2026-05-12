use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::header;
use axum::response::Response;
use axum::Json;
use sqlx::Row;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{
    AttributeDefinition, Category, CreateItem, ExportData, ImportItemPreview,
    ImportPreviewResult, ImportRequest, ImportResult, ImportStrategy, Item,
    ItemUsageCount, ItemUsageStats, Tag, TripRef, UpdateItem,
};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Item>>, AppError> {
    let rows = sqlx::query_as::<_, Item>(
        "SELECT id, category_id, tag_id, attrs FROM items ORDER BY category_id, id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn get(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<Item>, AppError> {
    let row = sqlx::query_as::<_, Item>(
        "SELECT id, category_id, tag_id, attrs FROM items WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("物品", id))?;
    Ok(Json(row))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateItem>,
) -> Result<Json<Item>, AppError> {
    body.validate()?;
    // Ensure name is in attrs (frontend sends it that way, but be safe)
    let attrs_str = serde_json::to_string(&body.attrs).unwrap_or_else(|_| "{}".to_string());
    let name = body.attrs.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let brand = body.attrs.get("brand").and_then(|v| v.as_str()).unwrap_or("");
    let model = body.attrs.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let default_qty = body.attrs.get("default_qty").and_then(|v| v.as_i64()).unwrap_or(1);
    let notes = body.attrs.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    // Old physical columns still exist (legacy) — fill them from attrs for compatibility
    let row = sqlx::query_as::<_, Item>(
        "INSERT INTO items (name, brand, model, category_id, default_qty, notes, tag_id, attrs) VALUES (?, ?, ?, ?, ?, ?, ?, ?) RETURNING id, category_id, tag_id, attrs",
    )
    .bind(name)
    .bind(brand)
    .bind(model)
    .bind(body.category_id)
    .bind(default_qty)
    .bind(notes)
    .bind(body.tag_id)
    .bind(&attrs_str)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateItem>,
) -> Result<Json<Item>, AppError> {
    let existing = sqlx::query_as::<_, Item>(
        "SELECT id, category_id, tag_id, attrs FROM items WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("物品", id))?;

    let category_id = body.category_id.unwrap_or(existing.category_id);
    let tag_id = match body.tag_id {
        Some(v) => v,
        None => existing.tag_id,
    };
    // Merge incoming attrs on top of existing instead of replacing
    let attrs = match &body.attrs {
        Some(new_attrs) => {
            let mut merged = existing.attrs.clone();
            if let serde_json::Value::Object(ref mut obj) = merged {
                if let serde_json::Value::Object(ref new_obj) = new_attrs {
                    for (k, v) in new_obj {
                        obj.insert(k.clone(), v.clone());
                    }
                }
            }
            merged
        }
        None => existing.attrs.clone(),
    };

    // Validate
    let name = attrs
        .get("name")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim()
        .to_string();
    if name.is_empty() {
        return Err(AppError::validation("物品名称不能为空"));
    }
    if name.len() > 200 {
        return Err(AppError::validation("物品名称不能超过200字符"));
    }
    let qty = attrs.get("default_qty").and_then(|v| v.as_i64()).unwrap_or(1);
    if qty < 1 {
        return Err(AppError::validation("默认数量必须大于0"));
    }

    let attrs_str = serde_json::to_string(&attrs).unwrap_or_else(|_| "{}".to_string());
    // Also sync legacy physical columns from attrs
    let name = attrs.get("name").and_then(|v| v.as_str()).unwrap_or("");
    let brand = attrs.get("brand").and_then(|v| v.as_str()).unwrap_or("");
    let model = attrs.get("model").and_then(|v| v.as_str()).unwrap_or("");
    let default_qty = attrs.get("default_qty").and_then(|v| v.as_i64()).unwrap_or(1);
    let notes = attrs.get("notes").and_then(|v| v.as_str()).unwrap_or("");
    let row = sqlx::query_as::<_, Item>(
        "UPDATE items SET name = ?, brand = ?, model = ?, category_id = ?, default_qty = ?, notes = ?, tag_id = ?, attrs = ? WHERE id = ? RETURNING id, category_id, tag_id, attrs",
    )
    .bind(name)
    .bind(brand)
    .bind(model)
    .bind(category_id)
    .bind(default_qty)
    .bind(notes)
    .bind(tag_id)
    .bind(&attrs_str)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM items WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("物品", id));
    }
    Ok(())
}

pub async fn usage_stats(State(pool): State<SqlitePool>) -> Result<Json<Vec<ItemUsageCount>>, AppError> {
    let rows = sqlx::query_as::<_, ItemUsageCount>(
        "SELECT item_id, COUNT(DISTINCT trip_id) as trip_count FROM trip_items WHERE item_id IS NOT NULL GROUP BY item_id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn usage_detail(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<Json<ItemUsageStats>, AppError> {
    let trips = sqlx::query_as::<_, TripRef>(
        "SELECT DISTINCT t.id, t.name, t.status FROM trips t JOIN trip_items ti ON ti.trip_id = t.id WHERE ti.item_id = ? ORDER BY t.id DESC",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;
    Ok(Json(ItemUsageStats { item_id: id, trips }))
}

// ── Import / Export ──

pub async fn export_items(State(pool): State<SqlitePool>) -> Result<Response<Body>, AppError> {
    let categories = sqlx::query_as::<_, Category>(
        "SELECT id, name, icon, sort_order FROM categories ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;

    let tags = sqlx::query_as::<_, Tag>(
        "SELECT id, name, category_id, sort_order FROM tags ORDER BY category_id, sort_order, id",
    )
    .fetch_all(&pool)
    .await?;

    let attribute_definitions = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT id, key, label, attr_type, config, category_scope, tag_scope, sort_order, is_identity, is_required, default_value, search_weight FROM attribute_definitions ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;

    let items = sqlx::query_as::<_, Item>(
        "SELECT id, category_id, tag_id, attrs FROM items ORDER BY category_id, id",
    )
    .fetch_all(&pool)
    .await?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let export_data = ExportData {
        version: 2,
        exported_at: now.to_string(),
        categories,
        tags,
        attribute_definitions,
        items,
    };

    let json = serde_json::to_string_pretty(&export_data).map_err(|e| {
        AppError::Internal(anyhow::anyhow!("序列化导出数据失败: {}", e))
    })?;

    let resp = Response::builder()
        .header(header::CONTENT_TYPE, "application/json; charset=utf-8")
        .header(
            header::CONTENT_DISPOSITION,
            "attachment; filename=\"inventory-export.json\"",
        )
        .body(Body::from(json))
        .expect("Failed to build export response");

    Ok(resp)
}

pub async fn import_preview(
    State(pool): State<SqlitePool>,
    Json(body): Json<ImportRequest>,
) -> Result<Json<ImportPreviewResult>, AppError> {
    if body.version < 1 || body.version > 2 {
        return Err(AppError::bad_request("不支持的导出文件版本"));
    }

    // Build lowercase name -> id map of existing items (name is in attrs JSON now)
    let existing_rows = sqlx::query("SELECT id, json_extract(attrs, '$.name') as name FROM items")
        .fetch_all(&pool)
        .await?;

    let mut name_to_id: HashMap<String, i64> = HashMap::new();
    for row in &existing_rows {
        let id: i64 = row.get(0);
        let name: String = row.get(1);
        name_to_id.insert(name.to_lowercase(), id);
    }

    let total_items = body.items.len();
    let mut new_items = 0usize;
    let mut skip_or_update_items = 0usize;
    let mut preview_items: Vec<ImportItemPreview> = Vec::with_capacity(total_items.min(50));

    let action_label = match body.strategy {
        ImportStrategy::Skip => "skip",
        ImportStrategy::Update => "update",
    };

    for item in &body.items {
        let item_name = item.attr_str("name");
        let key = item_name.to_lowercase();
        if let Some(existing_id) = name_to_id.get(&key) {
            skip_or_update_items += 1;
            if preview_items.len() < 50 {
                preview_items.push(ImportItemPreview {
                    name: item_name.clone(),
                    action: action_label.to_string(),
                    existing_id: Some(*existing_id),
                });
            }
        } else {
            new_items += 1;
            if preview_items.len() < 50 {
                preview_items.push(ImportItemPreview {
                    name: item_name.clone(),
                    action: "new".to_string(),
                    existing_id: None,
                });
            }
        }
    }

    Ok(Json(ImportPreviewResult {
        total_items,
        new_items,
        skip_or_update_items,
        preview_items,
    }))
}

pub async fn import_items(
    State(pool): State<SqlitePool>,
    Json(body): Json<ImportRequest>,
) -> Result<Json<ImportResult>, AppError> {
    if body.version < 1 || body.version > 2 {
        return Err(AppError::bad_request("不支持的导出文件版本"));
    }

    let mut tx = pool.begin().await?;

    // 1. Categories: upsert by name, build old_id -> new_id mapping
    let mut cat_remap: HashMap<i64, i64> = HashMap::new();
    let mut categories_created: u64 = 0;
    for cat in &body.categories {
        let existing = sqlx::query("SELECT id FROM categories WHERE name = ?")
            .bind(&cat.name)
            .fetch_optional(&mut *tx)
            .await?;
        if let Some(row) = existing {
            let existing_id: i64 = row.get(0);
            cat_remap.insert(cat.id, existing_id);
        } else {
            let new_id = sqlx::query(
                "INSERT INTO categories (name, icon, sort_order) VALUES (?, ?, ?)",
            )
            .bind(&cat.name)
            .bind(&cat.icon)
            .bind(cat.sort_order)
            .execute(&mut *tx)
            .await?
            .last_insert_rowid();
            cat_remap.insert(cat.id, new_id);
            categories_created += 1;
        }
    }

    // 2. Tags: upsert by name, remap category_id
    let mut tag_remap: HashMap<i64, i64> = HashMap::new();
    let mut tags_created: u64 = 0;
    for tag in &body.tags {
        let new_category_id = cat_remap.get(&tag.category_id).copied().unwrap_or(tag.category_id);
        let existing = sqlx::query("SELECT id FROM tags WHERE name = ?")
            .bind(&tag.name)
            .fetch_optional(&mut *tx)
            .await?;
        if let Some(row) = existing {
            let existing_id: i64 = row.get(0);
            tag_remap.insert(tag.id, existing_id);
            sqlx::query("UPDATE tags SET category_id = ? WHERE id = ?")
                .bind(new_category_id)
                .bind(existing_id)
                .execute(&mut *tx)
                .await?;
        } else {
            let new_id = sqlx::query(
                "INSERT INTO tags (name, category_id, sort_order) VALUES (?, ?, ?)",
            )
            .bind(&tag.name)
            .bind(new_category_id)
            .bind(tag.sort_order)
            .execute(&mut *tx)
            .await?
            .last_insert_rowid();
            tag_remap.insert(tag.id, new_id);
            tags_created += 1;
        }
    }

    // 3. Attribute definitions: upsert by key
    let mut attribute_definitions_created: u64 = 0;
    for adef in &body.attribute_definitions {
        let existing = sqlx::query("SELECT id FROM attribute_definitions WHERE key = ?")
            .bind(&adef.key)
            .fetch_optional(&mut *tx)
            .await?;
        if existing.is_none() {
            sqlx::query(
                "INSERT INTO attribute_definitions (key, label, attr_type, config, category_scope, tag_scope, sort_order, is_identity, is_required, default_value, search_weight) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&adef.key)
            .bind(&adef.label)
            .bind(&adef.attr_type)
            .bind(&adef.config)
            .bind(&adef.category_scope)
            .bind(&adef.tag_scope)
            .bind(adef.sort_order)
            .bind(adef.is_identity)
            .bind(adef.is_required)
            .bind(&adef.default_value)
            .bind(adef.search_weight)
            .execute(&mut *tx)
            .await?;
            attribute_definitions_created += 1;
        }
    }

    // 4. Items: match by name (case-insensitive, name from attrs JSON)
    let existing_rows = sqlx::query("SELECT id, LOWER(json_extract(attrs, '$.name')) as name_lower FROM items")
        .fetch_all(&mut *tx)
        .await?;
    let mut name_to_id: HashMap<String, i64> = HashMap::new();
    for row in &existing_rows {
        let id: i64 = row.get(0);
        let name_lower: String = row.get(1);
        name_to_id.insert(name_lower, id);
    }

    let mut items_created: u64 = 0;
    let mut items_updated: u64 = 0;
    let mut items_skipped: u64 = 0;

    for item in &body.items {
        let item_name = item.attr_str("name");
        let key = item_name.to_lowercase();
        if let Some(existing_id) = name_to_id.get(&key) {
            match body.strategy {
                ImportStrategy::Skip => {
                    items_skipped += 1;
                }
                ImportStrategy::Update => {
                    let new_category_id = cat_remap
                        .get(&item.category_id)
                        .copied()
                        .unwrap_or(item.category_id);
                    let new_tag_id = item.tag_id.and_then(|tid| tag_remap.get(&tid).copied());
                    let attrs_str =
                        serde_json::to_string(&item.attrs).unwrap_or_else(|_| "{}".to_string());
                    let name = item.attr_str("name");
                    let brand = item.attr_str("brand");
                    let model = item.attr_str("model");
                    let default_qty = item.attr_i64("default_qty").max(1);
                    let notes = item.attr_str("notes");
                    sqlx::query(
                        "UPDATE items SET name = ?, brand = ?, model = ?, category_id = ?, default_qty = ?, notes = ?, tag_id = ?, attrs = ? WHERE id = ?",
                    )
                    .bind(&name)
                    .bind(&brand)
                    .bind(&model)
                    .bind(new_category_id)
                    .bind(default_qty)
                    .bind(&notes)
                    .bind(new_tag_id)
                    .bind(&attrs_str)
                    .bind(existing_id)
                    .execute(&mut *tx)
                    .await?;
                    items_updated += 1;
                }
            }
        } else {
            let new_category_id = cat_remap
                .get(&item.category_id)
                .copied()
                .unwrap_or(item.category_id);
            let new_tag_id = item.tag_id.and_then(|tid| tag_remap.get(&tid).copied());
            let attrs_str =
                serde_json::to_string(&item.attrs).unwrap_or_else(|_| "{}".to_string());
            let name = item.attr_str("name");
            let brand = item.attr_str("brand");
            let model = item.attr_str("model");
            let default_qty = item.attr_i64("default_qty").max(1);
            let notes = item.attr_str("notes");
            sqlx::query(
                "INSERT INTO items (name, brand, model, category_id, default_qty, notes, tag_id, attrs) VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            )
            .bind(&name)
            .bind(&brand)
            .bind(&model)
            .bind(new_category_id)
            .bind(default_qty)
            .bind(&notes)
            .bind(new_tag_id)
            .bind(&attrs_str)
            .execute(&mut *tx)
            .await?;
            items_created += 1;
        }
    }

    tx.commit().await?;

    Ok(Json(ImportResult {
        categories_created,
        tags_created,
        attribute_definitions_created,
        items_created,
        items_updated,
        items_skipped,
    }))
}
