use axum::extract::{Path, State};
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{LlmConfig, LlmConfigPublic, UpdateLlmConfig};

/// GET /api/llm-configs — list all configs with masked api_key
pub async fn list(
    State(pool): State<SqlitePool>,
) -> Result<Json<Vec<LlmConfigPublic>>, AppError> {
    let configs = sqlx::query_as::<_, LlmConfig>(
        "SELECT id, task, provider_name, base_url, api_key, model, is_active, created_at, updated_at
         FROM llm_configs ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;

    let public: Vec<LlmConfigPublic> = configs.into_iter().map(LlmConfigPublic::from).collect();
    Ok(Json(public))
}

/// PUT /api/llm-configs/{id} — update a config
/// If api_key is empty string, keep the existing value.
pub async fn update(
    State(pool): State<SqlitePool>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateLlmConfig>,
) -> Result<Json<LlmConfigPublic>, AppError> {
    let existing = sqlx::query_as::<_, LlmConfig>(
        "SELECT id, task, provider_name, base_url, api_key, model, is_active, created_at, updated_at
         FROM llm_configs WHERE id = ?",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::validation("LLM 配置不存在"))?;

    let provider_name = body.provider_name.unwrap_or(existing.provider_name);
    let base_url = body.base_url.unwrap_or(existing.base_url);
    let api_key = match body.api_key {
        Some(ref s) if s.is_empty() => existing.api_key,
        Some(s) => s,
        None => existing.api_key,
    };
    let model = body.model.unwrap_or(existing.model);
    let is_active = body.is_active.unwrap_or(existing.is_active);

    let updated = sqlx::query_as::<_, LlmConfig>(
        "UPDATE llm_configs SET provider_name = ?, base_url = ?, api_key = ?, model = ?, is_active = ?, updated_at = datetime('now')
         WHERE id = ?
         RETURNING id, task, provider_name, base_url, api_key, model, is_active, created_at, updated_at",
    )
    .bind(&provider_name)
    .bind(&base_url)
    .bind(&api_key)
    .bind(&model)
    .bind(is_active)
    .bind(id)
    .fetch_one(&pool)
    .await?;

    Ok(Json(LlmConfigPublic::from(updated)))
}

/// Internal helper: load the active config for a given task.
/// Returns None if no active config is found in the DB.
pub(crate) async fn get_active(
    pool: &SqlitePool,
    task: &str,
) -> Option<LlmConfig> {
    sqlx::query_as::<_, LlmConfig>(
        "SELECT id, task, provider_name, base_url, api_key, model, is_active, created_at, updated_at
         FROM llm_configs WHERE task = ? AND is_active = 1 LIMIT 1",
    )
    .bind(task)
    .fetch_optional(pool)
    .await
    .ok()
    .flatten()
}
