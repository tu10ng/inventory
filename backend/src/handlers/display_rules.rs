use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{CreateDisplayRule, DisplayRule, UpdateDisplayRule};

pub async fn list(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<DisplayRule>>, AppError> {
    let rows = sqlx::query_as::<_, DisplayRule>(
        "SELECT id, name, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config FROM display_rules ORDER BY sort_order, id",
    )
    .fetch_all(&pool)
    .await?;
    Ok(Json(rows))
}

pub async fn create(
    State(pool): State<SqlitePool>,
    Json(body): Json<CreateDisplayRule>,
) -> Result<Json<DisplayRule>, AppError> {
    body.validate()?;

    let row = sqlx::query_as::<_, DisplayRule>(
        "INSERT INTO display_rules (name, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config) VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING id, name, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config",
    )
    .bind(&body.name)
    .bind(&body.group_by_key)
    .bind(&body.sort_by_key)
    .bind(&body.sort_dir)
    .bind(&body.visible_columns)
    .bind(body.sort_order)
    .bind(&body.config)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateDisplayRule>,
) -> Result<Json<DisplayRule>, AppError> {
    let existing = sqlx::query_as::<_, DisplayRule>(
        "SELECT id, name, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config FROM display_rules WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::not_found("展示规则", id))?;

    let name = body.name.unwrap_or(existing.name);
    let group_by_key = body.group_by_key.unwrap_or(existing.group_by_key);
    let sort_by_key = body.sort_by_key.unwrap_or(existing.sort_by_key);
    let sort_dir = body.sort_dir.unwrap_or(existing.sort_dir);
    let visible_columns = body.visible_columns.unwrap_or(existing.visible_columns);
    let sort_order = body.sort_order.unwrap_or(existing.sort_order);
    let config = body.config.unwrap_or(existing.config);

    if name.trim().is_empty() {
        return Err(AppError::validation("规则名称不能为空"));
    }
    if sort_dir != "asc" && sort_dir != "desc" {
        return Err(AppError::validation("排序方向只能为 asc 或 desc"));
    }
    serde_json::from_str::<serde_json::Value>(&visible_columns).map_err(|_| {
        AppError::validation("可见列不是有效的 JSON 数组")
    })?;

    let row = sqlx::query_as::<_, DisplayRule>(
        "UPDATE display_rules SET name = ?, group_by_key = ?, sort_by_key = ?, sort_dir = ?, visible_columns = ?, sort_order = ?, config = ? WHERE id = ? RETURNING id, name, group_by_key, sort_by_key, sort_dir, visible_columns, sort_order, config",
    )
    .bind(&name)
    .bind(&group_by_key)
    .bind(&sort_by_key)
    .bind(&sort_dir)
    .bind(&visible_columns)
    .bind(sort_order)
    .bind(&config)
    .bind(id)
    .fetch_one(&pool)
    .await?;
    Ok(Json(row))
}

pub async fn delete(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
) -> Result<(), AppError> {
    let result = sqlx::query("DELETE FROM display_rules WHERE id = ?")
        .bind(id)
        .execute(&pool)
        .await?;
    if result.rows_affected() == 0 {
        return Err(AppError::not_found("展示规则", id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn list_returns_seed_rules() {
        let pool = crate::db::init_test_pool().await;
        let Json(rules) = list(State(pool.clone())).await.unwrap();
        // Migration seeds 4 rules (3 basic + 1 summary)
        assert_eq!(rules.len(), 4);
        assert!(rules.iter().any(|r| r.name == "服装按部位"));
        assert!(rules.iter().any(|r| r.name == "按重量排序"));
    }

    #[tokio::test]
    async fn create_and_list() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "服装按部位".to_string(),
            group_by_key: "body_parts".to_string(),
            sort_by_key: "name".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: json!(["name", "tag", "body_parts"]).to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };

        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();
        assert_eq!(created.name, "服装按部位");
        assert_eq!(created.group_by_key, "body_parts");
        assert_eq!(created.sort_by_key, "name");

        let Json(rules) = list(State(pool.clone())).await.unwrap();
        assert_eq!(rules.len(), 5); // 4 seed + 1 new
        assert!(rules.iter().any(|r| r.name == "服装按部位"));
    }

    #[tokio::test]
    async fn create_with_empty_name_fails() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "  ".to_string(),
            group_by_key: "".to_string(),
            sort_by_key: "".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: "[]".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };

        let result = create(State(pool.clone()), Json(body)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_with_invalid_sort_dir_fails() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "测试".to_string(),
            group_by_key: "".to_string(),
            sort_by_key: "".to_string(),
            sort_dir: "invalid".to_string(),
            visible_columns: "[]".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };

        let result = create(State(pool.clone()), Json(body)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn create_with_invalid_visible_columns_fails() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "测试".to_string(),
            group_by_key: "".to_string(),
            sort_by_key: "".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: "not-json".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };

        let result = create(State(pool.clone()), Json(body)).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn update_fields() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "原始规则".to_string(),
            group_by_key: "body_parts".to_string(),
            sort_by_key: "name".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: "[]".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };
        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();

        let Json(updated) = update(
            State(pool.clone()),
            axum::extract::Path(created.id),
            Json(UpdateDisplayRule {
                name: None,
                group_by_key: None,
                sort_by_key: None,
                sort_dir: Some("desc".to_string()),
                visible_columns: None,
                sort_order: None,
                config: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.sort_dir, "desc");
        // Other fields preserved
        assert_eq!(updated.name, "原始规则");
        assert_eq!(updated.group_by_key, "body_parts");
    }

    #[tokio::test]
    async fn update_set_sort_dir() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "测试规则".to_string(),
            group_by_key: "".to_string(),
            sort_by_key: "".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: "[]".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
        };
        let Json(created) = create(State(pool.clone()), Json(body)).await.unwrap();
        assert_eq!(created.sort_dir, "asc");

        let Json(updated) = update(
            State(pool.clone()),
            axum::extract::Path(created.id),
            Json(UpdateDisplayRule {
                name: None,
                group_by_key: None,
                sort_by_key: None,
                sort_dir: Some("desc".to_string()),
                visible_columns: None,
                sort_order: None,
                config: None,
            }),
        )
        .await
        .unwrap();

        assert_eq!(updated.sort_dir, "desc");
    }

    #[tokio::test]
    async fn delete_and_verify() {
        let pool = crate::db::init_test_pool().await;

        let body = CreateDisplayRule {
            name: "待删除".to_string(),
            group_by_key: "".to_string(),
            sort_by_key: "".to_string(),
            sort_dir: "asc".to_string(),
            visible_columns: "[]".to_string(),
            sort_order: 0,
            config: "{}".to_string(),
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
