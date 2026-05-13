use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{AttributeDefinition, CreateAttributeDefinition, UpdateAttributeDefinition};

/// Reserved for future scope-based filtering (category_id / tag_id).
#[derive(serde::Deserialize, Default)]
#[allow(dead_code)]
pub struct ListQuery {
    pub category_id: Option<i64>,
    pub tag_id: Option<i64>,
}

pub async fn list(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<AttributeDefinition>>, AppError> {
    // Returns all definitions; scope filtering is done client-side
    // because the scope logic (OR match on category_scope/tag_scope, empty=global)
    // is complex to express in a single SQL query
    let rows = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateAttributeDefinition>,
) -> Result<Json<AttributeDefinition>, AppError> {
    if body.key.trim().is_empty() {
        return Err(AppError::validation("属性键不能为空"));
    }
    if body.label.trim().is_empty() {
        return Err(AppError::validation("属性标签不能为空"));
    }
    let row = sqlx::query_as::<_, AttributeDefinition>(
        "INSERT INTO attribute_definitions (key, label, attr_type, config, category_scope, tag_scope, sort_order, is_identity, is_required, default_value, search_weight) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?) RETURNING *",
    )
    .bind(&body.key)
    .bind(&body.label)
    .bind(&body.attr_type)
    .bind(&body.config)
    .bind(&body.category_scope)
    .bind(&body.tag_scope)
    .bind(body.sort_order)
    .bind(body.is_identity)
    .bind(body.is_required)
    .bind(&body.default_value)
    .bind(body.search_weight)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateAttributeDefinition>,
) -> Result<Json<AttributeDefinition>, AppError> {
    let existing = sqlx::query_as::<_, AttributeDefinition>("SELECT * FROM attribute_definitions WHERE id = ?")
        .bind(id)
        .fetch_optional(&pool)
        .await?
        .ok_or_else(|| AppError::not_found("属性定义", id))?;

    let key = body.key.unwrap_or(existing.key);
    let label = body.label.unwrap_or(existing.label);
    let attr_type = body.attr_type.unwrap_or(existing.attr_type);
    let config = body.config.unwrap_or(existing.config);
    let category_scope = body.category_scope.unwrap_or(existing.category_scope);
    let tag_scope = body.tag_scope.unwrap_or(existing.tag_scope);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);
    let is_identity = body.is_identity.unwrap_or(existing.is_identity);
    let is_required = body.is_required.unwrap_or(existing.is_required);
    let default_value = body.default_value.unwrap_or(existing.default_value);
    let search_weight = body.search_weight.unwrap_or(existing.search_weight);

    if key.trim().is_empty() {
        return Err(AppError::validation("属性键不能为空"));
    }
    if label.trim().is_empty() {
        return Err(AppError::validation("属性标签不能为空"));
    }

    let row = sqlx::query_as::<_, AttributeDefinition>(
        "UPDATE attribute_definitions SET key = ?, label = ?, attr_type = ?, config = ?, category_scope = ?, tag_scope = ?, sort_order = ?, is_identity = ?, is_required = ?, default_value = ?, search_weight = ? WHERE id = ? RETURNING *",
    )
    .bind(&key)
    .bind(&label)
    .bind(&attr_type)
    .bind(&config)
    .bind(&category_scope)
    .bind(&tag_scope)
    .bind(sort_order)
    .bind(is_identity)
    .bind(is_required)
    .bind(&default_value)
    .bind(search_weight)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM attribute_definitions WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("属性定义", id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn list_includes_identity_attrs() {
        let pool = crate::db::init_test_pool().await;
        let Json(defs) = list(State(pool.clone())).await.unwrap();

        // Should have name, brand, model as identity attributes
        let name_def = defs.iter().find(|d| d.key == "name");
        assert!(name_def.is_some());
        assert!(name_def.unwrap().is_identity);

        let brand_def = defs.iter().find(|d| d.key == "brand");
        assert!(brand_def.is_some());
        assert!(brand_def.unwrap().is_identity);

        let model_def = defs.iter().find(|d| d.key == "model");
        assert!(model_def.is_some());
        assert!(model_def.unwrap().is_identity);
    }

    #[tokio::test]
    async fn list_includes_search_weight() {
        let pool = crate::db::init_test_pool().await;
        let Json(defs) = list(State(pool.clone())).await.unwrap();

        let name_def = defs.iter().find(|d| d.key == "name").unwrap();
        assert_eq!(name_def.search_weight, 10);

        let brand_def = defs.iter().find(|d| d.key == "brand").unwrap();
        assert_eq!(brand_def.search_weight, 5);

        let notes_def = defs.iter().find(|d| d.key == "notes").unwrap();
        assert_eq!(notes_def.search_weight, 1);
    }

    #[tokio::test]
    async fn create_new_attr() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateAttributeDefinition {
            key: "test_field".to_string(),
            label: "测试字段".to_string(),
            attr_type: "text".to_string(),
            config: "{}".to_string(),
            category_scope: "".to_string(),
            tag_scope: "".to_string(),
            sort_order: 100,
            is_identity: false,
            is_required: false,
            default_value: "".to_string(),
            search_weight: 2,
        };

        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();
        assert_eq!(created.key, "test_field");
        assert_eq!(created.label, "测试字段");
        assert_eq!(created.attr_type, "text");
        assert_eq!(created.search_weight, 2);
    }

    #[tokio::test]
    async fn update_attr_fields() {
        let pool = crate::db::init_test_pool().await;

        // First create a new attr
        let body = CreateAttributeDefinition {
            key: "update_test".to_string(),
            label: "原始标签".to_string(),
            attr_type: "text".to_string(),
            config: "{}".to_string(),
            category_scope: "".to_string(),
            tag_scope: "".to_string(),
            sort_order: 50,
            is_identity: false,
            is_required: false,
            default_value: "".to_string(),
            search_weight: 0,
        };
        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();

        // Update search_weight
        let Json(updated) = update(
            State(pool.clone()),
            axum::extract::Path(created.id),
            Json(UpdateAttributeDefinition {
                key: None,
                label: None,
                attr_type: None,
                config: None,
                category_scope: None,
                tag_scope: None,
                sort_order: None,
                is_identity: None,
                is_required: None,
                default_value: None,
                search_weight: Some(15),
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.search_weight, 15);
        // Other fields preserved
        assert_eq!(updated.label, "原始标签");
    }

    #[tokio::test]
    async fn delete_attr() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateAttributeDefinition {
            key: "to_delete".to_string(),
            label: "待删除".to_string(),
            attr_type: "text".to_string(),
            config: "{}".to_string(),
            category_scope: "".to_string(),
            tag_scope: "".to_string(),
            sort_order: 200,
            is_identity: false,
            is_required: false,
            default_value: "".to_string(),
            search_weight: 0,
        };
        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();

        let _ = delete(State(pool.clone()), axum::extract::Path(created.id))
            .await
            .unwrap();

        // Verify deleted
        let result = delete(State(pool.clone()), axum::extract::Path(created.id)).await;
        assert!(result.is_err());
    }
}
