use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{
    ActivityInclude, ActivityIncludeEnriched, CreateActivityInclude, CreateItemRelation,
    CreateRelationType, ItemRelation, ItemRelationEnriched, RelationType, UpdateRelationType,
};

// ── Relation Types CRUD ──

pub async fn list_relation_types(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<RelationType>>, AppError> {
    let rows = sqlx::query_as::<_, RelationType>(
        "SELECT * FROM relation_types ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create_relation_type(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateRelationType>,
) -> Result<Json<RelationType>, AppError> {
    body.validate()?;
    let row = sqlx::query_as::<_, RelationType>(
        "INSERT INTO relation_types (name, label, color, icon, bidirectional, sort_order) VALUES (?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.name)
    .bind(&body.label)
    .bind(&body.color)
    .bind(&body.icon)
    .bind(body.bidirectional)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update_relation_type(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateRelationType>,
) -> Result<Json<RelationType>, AppError> {
    let existing = sqlx::query_as::<_, RelationType>(
        "SELECT * FROM relation_types WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("关系类型", id))?;

    let name = body.name.unwrap_or(existing.name);
    let label = body.label.unwrap_or(existing.label);
    let color = body.color.unwrap_or(existing.color);
    let icon = body.icon.unwrap_or(existing.icon);
    let bidirectional = body.bidirectional.unwrap_or(existing.bidirectional);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);

    if name.trim().is_empty() {
        return Err(AppError::validation("关系类型名称不能为空"));
    }

    let row = sqlx::query_as::<_, RelationType>(
        "UPDATE relation_types SET name = ?, label = ?, color = ?, icon = ?, bidirectional = ?, sort_order = ? WHERE id = ? RETURNING *",
    )
    .bind(&name)
    .bind(&label)
    .bind(&color)
    .bind(&icon)
    .bind(bidirectional)
    .bind(sort_order)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_relation_type(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM relation_types WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("关系类型", id));
    }
    Ok(())
}

// ── Item Relations ──

pub async fn list_item_relations(
    State(pool): State<SqlitePool>,
    Path(item_id): Path<i64>,
) -> Result<Json<Vec<ItemRelationEnriched>>, AppError> {
    // Also include relations where this item is the target (for bidirectional types)
    let rows = sqlx::query_as::<_, (i64, i64, i64, i64, String, String, String, String, String)>(
        "SELECT
            ir.id, ir.source_item_id, ir.target_item_id, ir.relation_type_id, ir.notes,
            COALESCE(json_extract(i.attrs, '$.name'), '?') as target_name,
            rt.label as relation_label,
            rt.color as relation_color,
            rt.icon as relation_icon
        FROM item_relations ir
        JOIN items i ON i.id = ir.target_item_id
        JOIN relation_types rt ON rt.id = ir.relation_type_id
        WHERE ir.source_item_id = ?
        UNION ALL
        SELECT
            ir.id, ir.target_item_id as source_item_id, ir.source_item_id as target_item_id,
            ir.relation_type_id, ir.notes,
            COALESCE(json_extract(i.attrs, '$.name'), '?') as target_name,
            rt.label as relation_label,
            rt.color as relation_color,
            rt.icon as relation_icon
        FROM item_relations ir
        JOIN items i ON i.id = ir.source_item_id
        JOIN relation_types rt ON rt.id = ir.relation_type_id
        WHERE ir.target_item_id = ? AND rt.bidirectional = 1
        ORDER BY relation_label",
    )
    .bind(item_id)
    .bind(item_id)
    .fetch_all(&pool)
    .await?;

    let enriched = rows
        .into_iter()
        .map(|(id, source_item_id, target_item_id, relation_type_id, notes, target_name, relation_label, relation_color, relation_icon)| {
            ItemRelationEnriched {
                id,
                source_item_id,
                target_item_id,
                relation_type_id,
                notes,
                target_name,
                relation_label,
                relation_color,
                relation_icon,
            }
        })
        .collect();

    Ok(Json(enriched))
}

pub async fn create_item_relation(
    State(pool): State<SqlitePool>,
    Path(item_id): Path<i64>,
    Json(body): Json<CreateItemRelation>,
) -> Result<Json<ItemRelation>, AppError> {
    if body.target_item_id == item_id {
        return Err(AppError::validation("不能与自身建立关系"));
    }
    let row = sqlx::query_as::<_, ItemRelation>(
        "INSERT INTO item_relations (source_item_id, target_item_id, relation_type_id, notes) VALUES (?, ?, ?, ?) RETURNING *",
    )
    .bind(item_id)
    .bind(body.target_item_id)
    .bind(body.relation_type_id)
    .bind(&body.notes)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_item_relation(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM item_relations WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("物品关系", id));
    }
    Ok(())
}

// ── Activity Includes ──

pub async fn list_activity_includes(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
) -> Result<Json<Vec<ActivityIncludeEnriched>>, AppError> {
    let rows = sqlx::query_as::<_, (i64, i64, i64, i64, String, String)>(
        "SELECT ai.id, ai.activity_id, ai.included_activity_id, ai.sort_order,
                a.name, a.icon
         FROM activity_includes ai
         JOIN activities a ON a.id = ai.included_activity_id
         WHERE ai.activity_id = ?
         ORDER BY ai.sort_order, ai.id",
    )
    .bind(activity_id)
    .fetch_all(&pool)
    .await?;

    let enriched = rows
        .into_iter()
        .map(|(id, activity_id, included_activity_id, sort_order, included_name, included_icon)| {
            ActivityIncludeEnriched {
                id,
                activity_id,
                included_activity_id,
                sort_order,
                included_name,
                included_icon,
            }
        })
        .collect();

    Ok(Json(enriched))
}

pub async fn create_activity_include(
    State(pool): State<SqlitePool>,
    Path(activity_id): Path<i64>,
    Json(body): Json<CreateActivityInclude>,
) -> Result<Json<ActivityInclude>, AppError> {
    if body.included_activity_id == activity_id {
        return Err(AppError::validation("活动不能引用自身"));
    }
    let row = sqlx::query_as::<_, ActivityInclude>(
        "INSERT INTO activity_includes (activity_id, included_activity_id, sort_order) VALUES (?, ?, ?) RETURNING *",
    )
    .bind(activity_id)
    .bind(body.included_activity_id)
    .bind(body.sort_order)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete_activity_include(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM activity_includes WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("活动引用", id));
    }
    Ok(())
}
