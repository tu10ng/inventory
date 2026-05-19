use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateType, Type, TypeTreeNode, UpdateType};

pub async fn list(State(pool): State<SqlitePool>) -> Result<Json<Vec<Type>>, AppError> {
    let rows = sqlx::query_as::<_, Type>(
        "SELECT id, name, sort_order, parent_id FROM types ORDER BY COALESCE(parent_id, id), sort_order, id"
    )
        .fetch_all(&pool)
        .await?;
    Ok(Json(rows))
}

pub async fn tree(State(pool): State<SqlitePool>) -> Result<Json<Vec<TypeTreeNode>>, AppError> {
    let all_types = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types ORDER BY COALESCE(parent_id, id), sort_order, id")
        .fetch_all(&pool)
        .await?;

    let mut map: std::collections::HashMap<i64, TypeTreeNode> = std::collections::HashMap::new();
    let mut roots: Vec<TypeTreeNode> = Vec::new();

    for t in &all_types {
        map.insert(t.id, TypeTreeNode {
            id: t.id,
            name: t.name.clone(),
            sort_order: t.sort_order,
            parent_id: t.parent_id,
            children: Vec::new(),
        });
    }

    for t in &all_types {
        let node = map.remove(&t.id).unwrap();
        if let Some(pid) = t.parent_id {
            if let Some(parent) = map.get_mut(&pid) {
                parent.children.push(node);
            } else {
                roots.push(node);
            }
        } else {
            roots.push(node);
        }
    }

    // Add any remaining orphaned nodes
    for (_, node) in map {
        roots.push(node);
    }

    roots.sort_by_key(|n| (n.sort_order, n.id));
    Ok(Json(roots))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateType>,
) -> Result<Json<Type>, AppError> {
    body.validate()?;

    // Validate parent_id exists
    if let Some(pid) = body.parent_id {
        sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types WHERE id = ?")
            .bind(pid)
            .fetch_optional(&pool)
            .await?
            .ok_or_else(|| AppError::not_found("父类型", pid))?;
    }

    let row = sqlx::query_as::<_, Type>(
        "INSERT INTO types (name, sort_order, parent_id) VALUES (?, ?, ?) RETURNING id, name, sort_order, parent_id",
    )
    .bind(&body.name)
    .bind(body.sort_order)
    .bind(body.parent_id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateType>,
) -> Result<Json<Type>, AppError> {
    let existing = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("类型", id))?;

    let name = body.name.unwrap_or(existing.name);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);
    let parent_id = body.parent_id.unwrap_or(existing.parent_id);

    if name.trim().is_empty() {
        return Err(AppError::validation("类型名称不能为空"));
    }

    // Validate parent_id if changed
    if let Some(Some(pid)) = body.parent_id {
        if pid != id {
            sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types WHERE id = ?")
                .bind(pid)
                .fetch_optional(&pool)
                .await?
                .ok_or_else(|| AppError::not_found("父类型", pid))?;
            // Check for cycles: ensure pid is not a descendant of id
            let mut current = Some(pid);
            while let Some(cid) = current {
                if cid == id {
                    return Err(AppError::validation("不能将自己或子类型的后代设为父类型（循环引用）"));
                }
                let ancestor = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types WHERE id = ?")
                    .bind(cid)
                    .fetch_optional(&pool)
                    .await?;
                current = ancestor.and_then(|a| a.parent_id);
            }
        }
    }

    let row = sqlx::query_as::<_, Type>(
        "UPDATE types SET name = ?, sort_order = ?, parent_id = ? WHERE id = ? RETURNING id, name, sort_order, parent_id",
    )
    .bind(&name)
    .bind(sort_order)
    .bind(parent_id)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    // Check for children
    let child_count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM types WHERE parent_id = ?")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    if child_count.0 > 0 {
        return Err(AppError::validation("该类型下有子类型，请先删除或移动子类型"));
    }

    let result = sqlx::query("DELETE FROM types WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("类型", id));
    }
    Ok(())
}
