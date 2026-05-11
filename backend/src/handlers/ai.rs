use axum::extract::State;
use axum::Json;
use sqlx::SqlitePool;

use crate::error::AppError;
use crate::models::{
    AiParseRequest, AiParseResponse, AiParsedItem, AttributeDefinition, Category, Item,
    OrganizeAction, OrganizeApplyRequest, OrganizeApplyResponse, OrganizePreviewResponse,
    OrganizeUpdateFields, Tag,
};

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

// ── Shared helpers ──

fn resolve_category_id(cat_name: &str, categories: &[Category]) -> i64 {
    let cat_name_lower = cat_name.to_lowercase();
    let matched = categories.iter().find(|c| {
        c.name.to_lowercase() == cat_name_lower
            || c.name.to_lowercase().contains(&cat_name_lower)
            || cat_name_lower.contains(&c.name.to_lowercase())
    });
    matched.map(|c| c.id).unwrap_or_else(|| {
        categories
            .iter()
            .find(|c| c.name == "其他")
            .or(categories.first())
            .map(|c| c.id)
            .unwrap_or(1)
    })
}

fn resolve_tag_id(tag_name: &str, cat_id: Option<i64>, tags: &[Tag]) -> Option<i64> {
    let tag_name_lower = tag_name.to_lowercase();
    let matched = tags.iter().find(|t| {
        let name_match = t.name.to_lowercase() == tag_name_lower
            || t.name.to_lowercase().contains(&tag_name_lower)
            || tag_name_lower.contains(&t.name.to_lowercase());
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
    matched.map(|t| t.id)
}

fn resolve_parsed_item(item: &mut AiParsedItem, categories: &[Category], tags: &[Tag]) {
    if let Some(ref cat_name) = item.category_name {
        item.category_id = Some(resolve_category_id(cat_name, categories));
    }
    if let Some(ref tag_name) = item.tag_name {
        item.tag_id = resolve_tag_id(tag_name, item.category_id, tags);
    }
}

/// Collect unique (tag_name, category_id) pairs that need creation,
/// insert them, back-fill tag_id, and return the new tags.
async fn auto_create_tags_for_items(
    items: &mut [AiParsedItem],
    pool: &SqlitePool,
) -> Result<Vec<Tag>, AppError> {
    let mut new_tags: Vec<Tag> = Vec::new();
    let mut to_create: std::collections::HashMap<(String, i64), ()> =
        std::collections::HashMap::new();
    for item in items.iter() {
        if item.tag_name.is_some() && item.tag_id.is_none() {
            if let (Some(ref tn), Some(cid)) = (&item.tag_name, item.category_id) {
                to_create.entry((tn.clone(), cid)).or_default();
            }
        }
    }

    let mut created_map: std::collections::HashMap<(String, i64), i64> =
        std::collections::HashMap::new();
    for (tag_name, category_id) in to_create.keys() {
        let max_sort: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(sort_order), 0) FROM tags WHERE category_id = ?",
        )
        .bind(category_id)
        .fetch_optional(pool)
        .await?;
        let sort_order = max_sort.map(|r| r.0).unwrap_or(0) + 1;

        let tag = sqlx::query_as::<_, Tag>(
            "INSERT INTO tags (name, category_id, sort_order) VALUES (?, ?, ?) \
             ON CONFLICT(name) DO UPDATE SET name=name \
             RETURNING *",
        )
        .bind(tag_name)
        .bind(category_id)
        .bind(sort_order)
        .fetch_one(pool)
        .await?;

        created_map.insert((tag_name.clone(), *category_id), tag.id);
        new_tags.push(tag);
    }

    for item in items.iter_mut() {
        if item.tag_name.is_some() && item.tag_id.is_none() {
            if let (Some(ref tn), Some(cid)) = (&item.tag_name, item.category_id) {
                if let Some(&tid) = created_map.get(&(tn.clone(), cid)) {
                    item.tag_id = Some(tid);
                }
            }
        }
    }

    Ok(new_tags)
}

async fn call_llm(system_prompt: &str, user_prompt: &str) -> Result<String, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;
    let api_base = std::env::var("OPENAI_API_BASE")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model =
        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    let chat_req = ChatRequest {
        model,
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system_prompt.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user_prompt.to_string(),
            },
        ],
        response_format: Some(serde_json::json!({"type": "json_object"})),
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

    Ok(content.clone())
}

// ── Parse Items ──

fn build_system_prompt(
    categories: &[Category],
    tags: &[Tag],
    attr_defs: &[AttributeDefinition],
) -> String {
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

    let mut attrs_desc = String::new();
    for ad in attr_defs {
        let type_hint = match ad.attr_type.as_str() {
            "bar" => {
                let config: serde_json::Value =
                    serde_json::from_str(&ad.config).unwrap_or_default();
                let max = config.get("max").and_then(|v| v.as_i64()).unwrap_or(10);
                format!("数值 0-{}", max)
            }
            "stars" => {
                let config: serde_json::Value =
                    serde_json::from_str(&ad.config).unwrap_or_default();
                let max = config.get("max").and_then(|v| v.as_i64()).unwrap_or(5);
                format!("星级 0-{}", max)
            }
            "bool" => "布尔 0或1".to_string(),
            "number" => "数值".to_string(),
            "weight" => "重量（克）".to_string(),
            "text" => "文本".to_string(),
            _ => ad.attr_type.clone(),
        };
        attrs_desc.push_str(&format!(
            "- {}: {} ({})\n",
            ad.key, ad.label, type_hint
        ));
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
- default_qty: 默认数量，通常为 1
- attrs: 物品属性对象，包含以下属性：
{attrs_desc}

## 规则
1. 尽量根据品牌和型号推断物品属性
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
    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order")
            .fetch_all(&pool)
            .await?;
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;
    let attr_defs = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order",
    )
    .fetch_all(&pool)
    .await?;

    let system_prompt = build_system_prompt(&categories, &tags, &attr_defs);
    let content = call_llm(&system_prompt, &body.text).await?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI JSON output: {}. Raw: {}", e, content))?;

    let items_val = parsed
        .get("items")
        .ok_or_else(|| anyhow::anyhow!("AI output missing 'items' field"))?;

    let mut items: Vec<AiParsedItem> = serde_json::from_value(items_val.clone())
        .map_err(|e| anyhow::anyhow!("Failed to deserialize items: {}. Raw: {}", e, items_val))?;

    for item in &mut items {
        resolve_parsed_item(item, &categories, &tags);
    }

    let new_tags = auto_create_tags_for_items(&mut items, &pool).await?;

    Ok(Json(AiParseResponse { items, new_tags }))
}

// ── Organize ──

fn build_organize_prompt(categories: &[Category], tags: &[Tag], items: &[Item]) -> String {
    let mut cats_desc = String::new();
    for c in categories {
        cats_desc.push_str(&format!("- id:{} {} (icon: {})\n", c.id, c.name, c.icon));
    }

    let mut tags_desc = String::new();
    for t in tags {
        let cat = categories.iter().find(|c| c.id == t.category_id);
        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("?");
        tags_desc.push_str(&format!("- id:{} {} (分类: {})\n", t.id, t.name, cat_name));
    }

    let mut items_desc = String::new();
    for item in items {
        let cat = categories.iter().find(|c| c.id == item.category_id);
        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("?");
        let tag = item
            .tag_id
            .and_then(|tid| tags.iter().find(|t| t.id == tid));
        let tag_name = tag.map(|t| t.name.as_str()).unwrap_or("");
        items_desc.push_str(&format!(
            "- id:{} name:\"{}\" brand:\"{}\" model:\"{}\" category:\"{}\" tag:\"{}\" notes:\"{}\"\n",
            item.id, item.name, item.brand, item.model, cat_name, tag_name, item.notes
        ));
    }

    format!(
        r#"你是一个户外装备数据库管理助手。以下是当前物品库中的所有物品，请检查数据质量问题并提出整理建议。

## 可用分类
{cats_desc}

## 可用标签（物品子类型）
{tags_desc}

## 当前物品列表
{items_desc}

## 检查项目
1. **合并物品需拆分**：名称中包含"和"、"+"等连接词的物品，应拆分为独立物品（如"墨镜和眼镜布"→"墨镜"+"眼镜布"）
2. **字段错位**：信息放在了错误的字段中，请根据每条记录的具体情况判断最合理的修复方式。常见现象：
   - model（型号）字段填了材质/类型等非型号信息（如 model="羊毛"）
   - tag（标签）和名称之间的信息分配不合理
   - 名称中包含了本应作为标签的信息，或反之
3. **缺少标签**：物品没有标签但应该有（根据名称可以推断出合适的子类型标签）
4. **分类错误**：物品的分类明显不正确
5. **重复物品**：名称/品牌/型号完全相同的物品

## 输出格式
请以 JSON 格式输出，格式为：
```json
{{"actions": [...]}}
```

每个 action 是以下三种之一：

### update — 修改物品字段
```json
{{
  "action_type": "update",
  "item_id": 123,
  "reason": "说明修改原因",
  "fields": {{
    "name": "新名称（不改则不包含此字段）",
    "brand": "新品牌（不改则不包含此字段）",
    "model": "新型号（不改则不包含此字段）",
    "category_name": "新分类名（不改则不包含此字段）",
    "tag_name": "新标签名（不改则不包含此字段，设为 null 表示清除标签）"
  }}
}}
```

### split — 拆分合并物品
```json
{{
  "action_type": "split",
  "item_id": 123,
  "reason": "说明拆分原因",
  "new_items": [
    {{"name": "物品1", "brand": "", "model": "", "category_name": "分类", "tag_name": "标签或null", "notes": "", "default_qty": 1, "attrs": {{}}}},
    {{"name": "物品2", ...}}
  ]
}}
```

### delete — 删除重复/无效物品
```json
{{
  "action_type": "delete",
  "item_id": 123,
  "reason": "说明删除原因"
}}
```

## 规则
1. 只输出需要修改的物品，没有问题的不要包含
2. reason 用中文简要说明
3. update 的 fields 只包含要修改的字段
4. split 的 new_items 中的 category_name 和 tag_name 必须从上面的分类/标签列表中选择
5. 如果没有发现任何问题，返回 {{"actions": []}}
6. 保守一些，只提出明显的问题，不要过度修改
7. **不要精简名称**：名称中包含品牌名（如"迪卡侬SIMOND软壳"）是用户的命名习惯，不是问题，不要建议移除"#
    )
}

pub async fn organize_preview(
    State(pool): State<SqlitePool>,
) -> Result<Json<OrganizePreviewResponse>, AppError> {
    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order")
            .fetch_all(&pool)
            .await?;
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;
    let items = sqlx::query_as::<_, Item>("SELECT id, name, brand, model, category_id, default_qty, notes, tag_id, attrs FROM items ORDER BY category_id, name")
        .fetch_all(&pool)
        .await?;

    let system_prompt = build_organize_prompt(&categories, &tags, &items);
    let content = call_llm(&system_prompt, "请分析以上物品列表，输出整理建议。").await?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI JSON: {}. Raw: {}", e, content))?;

    let actions_val = parsed
        .get("actions")
        .ok_or_else(|| anyhow::anyhow!("AI output missing 'actions' field"))?;

    let mut actions: Vec<OrganizeAction> = serde_json::from_value(actions_val.clone())
        .map_err(|e| {
            anyhow::anyhow!("Failed to deserialize actions: {}. Raw: {}", e, actions_val)
        })?;

    // Resolve category_name/tag_name to IDs in actions
    let mut all_new_items: Vec<&mut AiParsedItem> = Vec::new();
    for action in &mut actions {
        match action {
            OrganizeAction::Update { fields, .. } => {
                if let Some(ref cat_name) = fields.category_name {
                    fields.category_id = Some(resolve_category_id(cat_name, &categories));
                }
                if let Some(ref tag_name) = fields.tag_name {
                    fields.tag_id = Some(resolve_tag_id(tag_name, fields.category_id, &tags));
                }
            }
            OrganizeAction::Split { new_items, .. } => {
                for item in new_items.iter_mut() {
                    resolve_parsed_item(item, &categories, &tags);
                }
                all_new_items.extend(new_items.iter_mut());
            }
            OrganizeAction::Delete { .. } => {}
        }
    }

    // Auto-create tags for split new_items
    // Collect all new_items that need tags across all split actions
    let mut tag_items_to_process: Vec<AiParsedItem> = Vec::new();
    for action in &actions {
        if let OrganizeAction::Split { new_items, .. } = action {
            for item in new_items {
                if item.tag_name.is_some() && item.tag_id.is_none() {
                    tag_items_to_process.push(item.clone());
                }
            }
        }
    }
    let new_tags = auto_create_tags_for_items(&mut tag_items_to_process, &pool).await?;

    // Back-fill tag_ids from created tags
    for action in &mut actions {
        if let OrganizeAction::Split { new_items, .. } = action {
            for item in new_items.iter_mut() {
                if item.tag_name.is_some() && item.tag_id.is_none() {
                    // Find matching created tag
                    if let (Some(ref tn), Some(cid)) = (&item.tag_name, item.category_id) {
                        if let Some(tag) = new_tags.iter().find(|t| {
                            t.name.to_lowercase() == tn.to_lowercase() && t.category_id == cid
                        }) {
                            item.tag_id = Some(tag.id);
                        }
                    }
                }
            }
        }
    }

    // Also handle update actions with unresolved tag_name → create tags
    let mut update_new_tags: Vec<Tag> = Vec::new();
    for action in &mut actions {
        if let OrganizeAction::Update { fields, item_id, .. } = action {
            if let Some(ref tag_name) = fields.tag_name {
                // Need to determine category_id
                let cat_id = fields.category_id.or_else(|| {
                    items.iter().find(|i| i.id == *item_id).map(|i| i.category_id)
                });
                if let Some(cid) = cat_id {
                    if fields.tag_id == Some(None) || fields.tag_id.is_none() {
                        // Check if tag exists
                        let existing = resolve_tag_id(tag_name, Some(cid), &tags);
                        if let Some(tid) = existing {
                            fields.tag_id = Some(Some(tid));
                        } else {
                            // Also check newly created tags
                            let from_new = new_tags
                                .iter()
                                .chain(update_new_tags.iter())
                                .find(|t| {
                                    t.name.to_lowercase() == tag_name.to_lowercase()
                                        && t.category_id == cid
                                });
                            if let Some(t) = from_new {
                                fields.tag_id = Some(Some(t.id));
                            } else {
                                // Create the tag
                                let max_sort: Option<(i64,)> = sqlx::query_as(
                                    "SELECT COALESCE(MAX(sort_order), 0) FROM tags WHERE category_id = ?",
                                )
                                .bind(cid)
                                .fetch_optional(&pool)
                                .await?;
                                let sort_order = max_sort.map(|r| r.0).unwrap_or(0) + 1;

                                let tag = sqlx::query_as::<_, Tag>(
                                    "INSERT INTO tags (name, category_id, sort_order) VALUES (?, ?, ?) \
                                     ON CONFLICT(name) DO UPDATE SET name=name \
                                     RETURNING *",
                                )
                                .bind(tag_name)
                                .bind(cid)
                                .bind(sort_order)
                                .fetch_one(&pool)
                                .await?;

                                fields.tag_id = Some(Some(tag.id));
                                update_new_tags.push(tag);
                            }
                        }
                    }
                }
            }
        }
    }

    let mut all_new_tags = new_tags;
    all_new_tags.extend(update_new_tags);

    Ok(Json(OrganizePreviewResponse {
        actions,
        new_tags: all_new_tags,
    }))
}

pub async fn organize_apply(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrganizeApplyRequest>,
) -> Result<Json<OrganizeApplyResponse>, AppError> {
    let mut updated: i64 = 0;
    let mut created: i64 = 0;
    let mut deleted: i64 = 0;
    let new_tags: Vec<Tag> = Vec::new();

    let mut tx = pool.begin().await?;

    // Pre-fetch valid FK IDs to validate AI-provided references
    let valid_cat_ids: std::collections::HashSet<i64> =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM categories")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| r.0)
            .collect();
    let valid_tag_ids: std::collections::HashSet<i64> =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM tags")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| r.0)
            .collect();

    for action in &body.actions {
        match action {
            OrganizeAction::Update { item_id, fields, .. } => {
                apply_update_fields(&mut tx, *item_id, fields, &valid_cat_ids, &valid_tag_ids)
                    .await?;
                updated += 1;
            }
            OrganizeAction::Split {
                item_id,
                new_items,
                ..
            } => {
                // Get trip_items referencing this item
                let trip_item_ids: Vec<(i64,)> = sqlx::query_as(
                    "SELECT id FROM trip_items WHERE item_id = ?",
                )
                .bind(item_id)
                .fetch_all(&mut *tx)
                .await?;

                // Get activity_slots referencing this item as default
                let slot_ids: Vec<(i64,)> = sqlx::query_as(
                    "SELECT id FROM activity_slots WHERE default_item_id = ?",
                )
                .bind(item_id)
                .fetch_all(&mut *tx)
                .await?;

                // Insert new items first (before deleting original)
                let mut first_new_id: Option<i64> = None;
                for new_item in new_items {
                    let cat_id = new_item.category_id.unwrap_or(1);
                    let cat_id = if valid_cat_ids.contains(&cat_id) { cat_id } else { 1 };
                    let tag_id = new_item.tag_id.filter(|id| valid_tag_ids.contains(id));

                    let result = sqlx::query(
                        "INSERT INTO items (name, brand, model, category_id, tag_id, default_qty, notes, attrs) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(&new_item.name)
                    .bind(&new_item.brand)
                    .bind(&new_item.model)
                    .bind(cat_id)
                    .bind(tag_id)
                    .bind(new_item.default_qty)
                    .bind(&new_item.notes)
                    .bind(serde_json::to_string(&new_item.attrs).unwrap_or_else(|_| "{}".to_string()))
                    .execute(&mut *tx)
                    .await?;

                    let new_id = result.last_insert_rowid();
                    if first_new_id.is_none() {
                        first_new_id = Some(new_id);
                    }
                    created += 1;
                }

                // Point activity_slots.default_item_id to first new item (or NULL)
                for (sid,) in &slot_ids {
                    sqlx::query("UPDATE activity_slots SET default_item_id = ? WHERE id = ?")
                        .bind(first_new_id)
                        .bind(sid)
                        .execute(&mut *tx)
                        .await?;
                }

                // Now safe to delete original item
                sqlx::query("DELETE FROM items WHERE id = ?")
                    .bind(item_id)
                    .execute(&mut *tx)
                    .await?;
                deleted += 1;

                // Point trip_items to first new item
                if let Some(new_id) = first_new_id {
                    for (ti_id,) in &trip_item_ids {
                        sqlx::query("UPDATE trip_items SET item_id = ? WHERE id = ?")
                            .bind(new_id)
                            .bind(ti_id)
                            .execute(&mut *tx)
                            .await?;
                    }
                }
            }
            OrganizeAction::Delete { item_id, .. } => {
                // Clear activity_slots.default_item_id referencing this item (no ON DELETE SET NULL)
                sqlx::query("UPDATE activity_slots SET default_item_id = NULL WHERE default_item_id = ?")
                    .bind(item_id)
                    .execute(&mut *tx)
                    .await?;
                // ON DELETE SET NULL preserves trip_items as manual entries
                sqlx::query("DELETE FROM items WHERE id = ?")
                    .bind(item_id)
                    .execute(&mut *tx)
                    .await?;
                deleted += 1;
            }
        }
    }

    tx.commit().await?;

    Ok(Json(OrganizeApplyResponse {
        updated,
        created,
        deleted,
        new_tags,
    }))
}

async fn apply_update_fields(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: i64,
    fields: &OrganizeUpdateFields,
    valid_cat_ids: &std::collections::HashSet<i64>,
    valid_tag_ids: &std::collections::HashSet<i64>,
) -> Result<(), AppError> {
    // Build dynamic UPDATE
    if let Some(ref name) = fields.name {
        sqlx::query("UPDATE items SET name = ? WHERE id = ?")
            .bind(name)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }
    if let Some(ref brand) = fields.brand {
        sqlx::query("UPDATE items SET brand = ? WHERE id = ?")
            .bind(brand)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }
    if let Some(ref model) = fields.model {
        sqlx::query("UPDATE items SET model = ? WHERE id = ?")
            .bind(model)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }
    if let Some(cat_id) = fields.category_id {
        if valid_cat_ids.contains(&cat_id) {
            sqlx::query("UPDATE items SET category_id = ? WHERE id = ?")
                .bind(cat_id)
                .bind(item_id)
                .execute(&mut **tx)
                .await?;
        }
    }
    if let Some(ref tag_id_opt) = fields.tag_id {
        // Validate: Some(id) must exist, Some(None) clears the tag
        let safe_tag_id = match tag_id_opt {
            Some(id) if valid_tag_ids.contains(id) => &Some(*id),
            Some(_) => &None, // invalid tag_id → clear tag
            None => &None,    // explicit clear
        };
        sqlx::query("UPDATE items SET tag_id = ? WHERE id = ?")
            .bind(safe_tag_id)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }
    if let Some(ref notes) = fields.notes {
        sqlx::query("UPDATE items SET notes = ? WHERE id = ?")
            .bind(notes)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}
