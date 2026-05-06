use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{AiParseRequest, AiParseResponse, AiParsedItem, Category, Tag};

#[derive(serde::Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(serde::Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    #[serde(skip_serializing_if = "Option::is_none")]
    response_format: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(serde::Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(serde::Deserialize)]
struct ChatChoice {
    message: ChatMessageOut,
}

#[derive(serde::Deserialize)]
struct ChatMessageOut {
    content: Option<String>,
}

fn build_system_prompt(categories: &[Category], tags: &[Tag]) -> String {
    let mut cats_desc = String::new();
    for c in categories {
        cats_desc.push_str(&format!("- {} (icon: {})\n", c.name, c.icon));
    }

    let mut tags_desc = String::new();
    for t in tags {
        let cat = categories.iter().find(|c| c.id == t.category_id);
        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("?");
        tags_desc.push_str(&format!("- {} (分类: {})\n", t.name, cat_name));
    }

    format!(
        r#"你是一个户外装备专家助手。用户会用自然语言描述一批物品，你需要解析出每个物品的结构化信息。

## 可用分类
{cats_desc}

## 可用标签（物品子类型）
{tags_desc}

## 输出字段说明
对于每个物品，输出以下字段：
- name: 物品名称（简洁，如"冲锋衣"、"登山杖"）
- brand: 品牌（如"始祖鸟"、"Black Diamond"，未知留空字符串）
- model: 型号（如"Beta LT"、"飞耐时7"，未知留空字符串）
- category_name: 从上面的分类列表中选择最合适的分类名称
- tag_name: 从上面的标签列表中选择最合适的标签名称，没有合适的留 null
- notes: 备注（通常为空字符串）
- warmth_rating: 保暖等级 0-5（0=不保暖，5=极暖）
- material: 材质（如"Gore-Tex"、"尼龙"，未知留空字符串）
- encumbrance: 累赘度 0-5（0=无感，5=极累赘）
- waterproof: 防水等级 0-5（0=不防水，5=完全防水）
- weight_grams: 重量（克），尽量给出准确值，未知填 0
- season: 适用季节（如"四季"、"冬季"、"三季"，留空表示不限）
- body_parts: 覆盖身体部位（如"上身"、"脚"、"头"，留空表示不适用）
- env_protection: 环境防护 0-5
- durability: 耐久度 0-5
- storage_ml: 容量（毫升），不适用填 0
- breathable: 透气性 0-5
- default_qty: 默认数量，通常为 1

## 规则
1. 尽量根据品牌和型号推断物品属性（重量、材质、防水等级等）
2. 如果用户只说了一个概括性的描述，拆分成独立物品
3. category_name 必须从上面的分类列表中选择
4. tag_name 从标签列表中选择最合适的；如果没有匹配的，请给出一个合理的简短标签名（2-4字，如"手表"、"头灯"）
5. 数值属性不确定时给出合理估计值，而不是全填 0

请以 JSON 格式输出，格式为：{{"items": [...]}}"#
    )
}

pub async fn parse_items(
    State(pool): State<SqlitePool>,
    Json(body): Json<AiParseRequest>,
) -> Result<Json<AiParseResponse>, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;
    let api_base = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = std::env::var("OPENAI_MODEL")
        .unwrap_or_else(|_| "gpt-4o".to_string());
    // Load categories and tags from DB
    let categories = sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;

    let system_prompt = build_system_prompt(&categories, &tags);

    let response_format = serde_json::json!({
        "type": "json_object"
    });

    let chat_req = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".to_string(),
                content: body.text,
            },
        ],
        response_format: Some(response_format),
        tools: None,
    };

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", api_base.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&chat_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call AI API: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("AI API returned {}: {}", status, text).into());
    }

    let chat_resp: ChatResponse = resp
        .json()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to parse AI response: {}", e))?;

    let content = chat_resp
        .choices
        .first()
        .and_then(|c| c.message.content.as_ref())
        .ok_or_else(|| anyhow::anyhow!("AI returned empty response"))?;

    // Parse the JSON from LLM output
    let parsed: serde_json::Value = serde_json::from_str(content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI JSON output: {}. Raw: {}", e, content))?;

    let items_val = parsed
        .get("items")
        .ok_or_else(|| anyhow::anyhow!("AI output missing 'items' field"))?;

    let mut items: Vec<AiParsedItem> = serde_json::from_value(items_val.clone())
        .map_err(|e| anyhow::anyhow!("Failed to deserialize items: {}. Raw: {}", e, items_val))?;

    // Resolve category_name/tag_name to IDs
    for item in &mut items {
        // Resolve category
        if let Some(ref cat_name) = item.category_name {
            let cat_name_lower = cat_name.to_lowercase();
            let matched = categories.iter().find(|c| {
                c.name.to_lowercase() == cat_name_lower
                    || c.name.to_lowercase().contains(&cat_name_lower)
                    || cat_name_lower.contains(&c.name.to_lowercase())
            });
            item.category_id = Some(matched.map(|c| c.id).unwrap_or_else(|| {
                // Default to "其他" category or first category
                categories
                    .iter()
                    .find(|c| c.name == "其他")
                    .or(categories.first())
                    .map(|c| c.id)
                    .unwrap_or(1)
            }));
        }

        // Resolve tag
        if let Some(ref tag_name) = item.tag_name {
            let tag_name_lower = tag_name.to_lowercase();
            let cat_id = item.category_id;
            let matched = tags.iter().find(|t| {
                let name_match = t.name.to_lowercase() == tag_name_lower
                    || t.name.to_lowercase().contains(&tag_name_lower)
                    || tag_name_lower.contains(&t.name.to_lowercase());
                // Prefer tags in the same category
                if let Some(cid) = cat_id {
                    name_match && t.category_id == cid
                } else {
                    name_match
                }
            });
            // Fallback: try without category constraint
            let matched = matched.or_else(|| {
                tags.iter().find(|t| {
                    t.name.to_lowercase() == tag_name_lower
                        || t.name.to_lowercase().contains(&tag_name_lower)
                        || tag_name_lower.contains(&t.name.to_lowercase())
                })
            });
            item.tag_id = matched.map(|t| t.id);
        }
    }

    // Auto-create missing tags
    let mut new_tags: Vec<Tag> = Vec::new();
    {
        // Collect unique (tag_name, category_id) pairs that need creation
        let mut to_create: std::collections::HashMap<(String, i64), ()> = std::collections::HashMap::new();
        for item in &items {
            if item.tag_name.is_some() && item.tag_id.is_none() {
                if let (Some(ref tn), Some(cid)) = (&item.tag_name, item.category_id) {
                    to_create.entry((tn.clone(), cid)).or_default();
                }
            }
        }

        // Insert new tags and collect them
        let mut created_map: std::collections::HashMap<(String, i64), i64> = std::collections::HashMap::new();
        for (tag_name, category_id) in to_create.keys() {
            // Get max sort_order for this category
            let max_sort: Option<(i64,)> = sqlx::query_as(
                "SELECT COALESCE(MAX(sort_order), 0) FROM tags WHERE category_id = ?"
            )
                .bind(category_id)
                .fetch_optional(&pool)
                .await?;
            let sort_order = max_sort.map(|r| r.0).unwrap_or(0) + 1;

            let tag = sqlx::query_as::<_, Tag>(
                "INSERT INTO tags (name, category_id, sort_order) VALUES (?, ?, ?) \
                 ON CONFLICT(name) DO UPDATE SET name=name \
                 RETURNING *"
            )
                .bind(tag_name)
                .bind(category_id)
                .bind(sort_order)
                .fetch_one(&pool)
                .await?;

            created_map.insert((tag_name.clone(), *category_id), tag.id);
            new_tags.push(tag);
        }

        // Back-fill tag_id on items
        for item in &mut items {
            if item.tag_name.is_some() && item.tag_id.is_none() {
                if let (Some(ref tn), Some(cid)) = (&item.tag_name, item.category_id) {
                    if let Some(&tid) = created_map.get(&(tn.clone(), cid)) {
                        item.tag_id = Some(tid);
                    }
                }
            }
        }
    }

    Ok(Json(AiParseResponse { items, new_tags }))
}

