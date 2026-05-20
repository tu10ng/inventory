use axum::extract::State;
use axum::response::sse::{Event, Sse};
use axum::Json;
use sqlx::SqlitePool;
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::error::AppError;
use crate::handlers::llm_configs;
use crate::models::{
    AiParseRequest, AiParseResponse, AiParsedItem, AttributeDefinition, Item, LlmConfig,
    OrganizeAction, OrganizeApplyRequest, OrganizeApplyResponse, OrganizePreviewRequest,
    OrganizePreviewResponse, OrganizeUpdateFields, SseEvent, Type,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    stream: Option<bool>,
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

// ── Vision (multimodal) API ──

/// Vision API format adapter — each provider has a different message layout.
enum VisionProvider {
    /// Moonshot/Kimi: system 消息 content 是纯字符串，user content 顺序 image_url 在前
    Kimi,
    /// OpenAI 及兼容厂商: system 消息 content 是纯字符串，user content 顺序 text 在前
    OpenAI,
}

impl VisionProvider {
    fn from_provider_name(name: &str) -> Self {
        match name.to_lowercase().as_str() {
            "kimi" | "moonshot" => Self::Kimi,
            _ => Self::OpenAI,
        }
    }

    /// 构造 vision API 请求体 JSON，适配不同厂商的消息 layout
    fn build_request_body(
        &self,
        model: &str,
        system_prompt: &str,
        user_text: &str,
        images: &[(String, String)], // (mime_type, base64_data)
    ) -> serde_json::Value {
        let mut user_content: Vec<serde_json::Value> = Vec::new();

        match self {
            Self::Kimi => {
                // Kimi: image_url 在 text 之前
                for (mime, b64) in images {
                    user_content.push(serde_json::json!({
                        "type": "image_url",
                        "image_url": {"url": format!("data:{};base64,{}", mime, b64)}
                    }));
                }
                user_content.push(serde_json::json!({"type": "text", "text": user_text}));
            }
            Self::OpenAI => {
                // OpenAI: text 在最前，然后是 image_url
                user_content.push(serde_json::json!({"type": "text", "text": user_text}));
                for (mime, b64) in images {
                    user_content.push(serde_json::json!({
                        "type": "image_url",
                        "image_url": {"url": format!("data:{};base64,{}", mime, b64)}
                    }));
                }
            }
        }

        serde_json::json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": user_content}
            ]
        })
    }
}

/// Call a multimodal/vision LLM with base64-encoded images.
/// Returns the LLM's text response.
pub(crate) async fn call_llm_vision(
    config: &LlmConfig,
    system_prompt: &str,
    user_text_prompt: &str,
    images: &[(String, String)], // (mime_type, base64_data)
) -> Result<String, AppError> {
    let provider = VisionProvider::from_provider_name(&config.provider_name);
    let body = provider.build_request_body(&config.model, system_prompt, user_text_prompt, images);

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&body)
        .timeout(std::time::Duration::from_secs(180))
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("无法连接到 {} ({}): {}", config.provider_name, url, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        return Err(anyhow::anyhow!("{} Vision API 返回 {}: {}", config.provider_name, status, text).into());
    }

    let chat_resp: ChatResponse = resp.json().await
        .map_err(|e| anyhow::anyhow!("解析 {} Vision API 响应失败: {}", config.provider_name, e))?;

    let content = chat_resp.choices.first()
        .and_then(|c| c.message.content.as_ref())
        .ok_or_else(|| anyhow::anyhow!("{} Vision API 返回空响应", config.provider_name))?;

    tracing::info!(
        "[LLM Vision] {} 模型={} 响应({} 字符): {}",
        config.provider_name,
        config.model,
        content.chars().count(),
        &content[..content.len().min(500)]
    );

    Ok(content.clone())
}

// ── Shared helpers ──

pub(crate) fn resolve_type_id(type_name: &str, types: &[Type]) -> Option<i64> {
    let type_name_lower = type_name.to_lowercase();
    let matched = types.iter().find(|t| {
        t.name.to_lowercase() == type_name_lower
            || t.name.to_lowercase().contains(&type_name_lower)
            || type_name_lower.contains(&t.name.to_lowercase())
    });
    matched.map(|t| t.id)
}

pub(crate) fn resolve_parsed_item(item: &mut AiParsedItem, types: &[Type]) {
    if let Some(ref type_name) = item.type_name {
        item.type_id = resolve_type_id(type_name, types);
    }
}

/// Collect unique type_name values that need creation,
/// insert them, back-fill type_id, and return the new types.
pub(crate) async fn auto_create_types_for_items(
    items: &mut [AiParsedItem],
    pool: &SqlitePool,
) -> Result<Vec<Type>, AppError> {
    let mut new_types: Vec<Type> = Vec::new();
    let mut to_create: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for item in items.iter() {
        if item.type_name.is_some() && item.type_id.is_none() {
            if let Some(ref tn) = item.type_name {
                to_create.insert(tn.clone());
            }
        }
    }

    let mut created_map: std::collections::HashMap<String, i64> =
        std::collections::HashMap::new();
    for type_name in &to_create {
        let max_sort: Option<(i64,)> = sqlx::query_as(
            "SELECT COALESCE(MAX(sort_order), 0) FROM types",
        )
        .fetch_optional(pool)
        .await?;
        let sort_order = max_sort.map(|r| r.0).unwrap_or(0) + 1;

        let tag = sqlx::query_as::<_, Type>(
            "INSERT INTO types (name, sort_order) VALUES (?, ?) \
             ON CONFLICT(name) DO UPDATE SET name=name \
             RETURNING id, name, sort_order, parent_id",
        )
        .bind(type_name)
        .bind(sort_order)
        .fetch_one(pool)
        .await?;

        created_map.insert(type_name.clone(), tag.id);
        new_types.push(tag);
    }

    for item in items.iter_mut() {
        if item.type_name.is_some() && item.type_id.is_none() {
            if let Some(ref tn) = item.type_name {
                if let Some(&tid) = created_map.get(tn) {
                    item.type_id = Some(tid);
                }
            }
        }
    }

    Ok(new_types)
}

async fn call_llm(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
) -> Result<String, AppError> {
    let chat_req = ChatRequest {
        model: config.model.clone(),
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
        stream: None,
    };

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
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

    tracing::info!(
        "[LLM] {} 模型={} 响应({} 字符): {}",
        config.provider_name,
        config.model,
        content.chars().count(),
        &content[..content.len().min(500)]
    );

    Ok(content.clone())
}

// ── Parse Items ──

fn build_system_prompt(
    types: &[Type],
    attr_defs: &[AttributeDefinition],
) -> String {
    // Use root types (parent_id IS NULL) as category descriptions
    let mut cats_desc = String::new();
    let root_types: Vec<&Type> = types.iter().filter(|t| t.parent_id.is_none()).collect();
    for rt in &root_types {
        cats_desc.push_str(&format!("- {} (id: {})\n", rt.name, rt.id));
    }

    let mut types_desc = String::from("## 类型列表（支持层级，优先选择最具体的子类型）\n");
    // Build tree-structured output
    for rt in &root_types {
        let children: Vec<&Type> = types.iter().filter(|t| t.parent_id == Some(rt.id)).collect();
        types_desc.push_str(&format!("  {} (root):\n", rt.name));
        for t in &children {
            let indent = "  ".repeat(2);
            types_desc.push_str(&format!("{}- id:{} {} (父级: {})\n", indent, t.id, t.name, rt.name));
            // Grandchildren
            for grandchild in types.iter().filter(|ct| ct.parent_id == Some(t.id)) {
                let indent3 = "  ".repeat(3);
                types_desc.push_str(&format!("{}- id:{} {} (父级: {})\n", indent3, grandchild.id, grandchild.name, t.name));
            }
        }
        // Also include orphan direct children under root
        for t in types.iter().filter(|t| t.parent_id.is_none() && t.id != rt.id) {
            // These are other root types or ad-hoc types, skip duplicate output
            let _ = t;
        }
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
        let scope_info = if ad.category_scope.is_empty() {
            "全局".to_string()
        } else {
            let ids: Vec<&str> = ad.category_scope.split(',').collect();
            let names: Vec<String> = ids
                .iter()
                .filter_map(|id| {
                    let id_num: i64 = id.trim().parse().ok()?;
                    root_types.iter().find(|rt| rt.id == id_num).map(|rt| rt.name.clone())
                })
                .collect();
            if names.is_empty() {
                "全局".to_string()
            } else {
                names.join("/")
            }
        };
        attrs_desc.push_str(&format!(
            "- {}: {} ({}, 适用分类: {})\n",
            ad.key, ad.label, type_hint, scope_info
        ));
    }

    format!(
        r#"你是一个装备管理助手。用户可能用自然语言描述物品，也可能粘贴订单/购物小票的 OCR 识别文本，或者提供 Excel 表格数据（每行格式为"列名: 值 | 列名: 值"）。你需要解析出每个物品的结构化信息。

## 品牌识别规则（重要）
- 品牌指的是母公司，不是子品牌或产品线
- 例如：QUECHUA、FORCLAZ、VAN RYSEL、SIMOND、KIPSTA、BTWIN、ROCKRIDER、DOMYOS、KALENJI、INESIS 等都是迪卡侬(Decathlon)的子品牌/产品线，不是品牌
- 如果商品的母公司是迪卡侬，brand 填 "迪卡侬"
- 子品牌名称可以放入 model 字段，或保留在 name 中
- 常见品牌：ARC'TERYX(始祖鸟)、Black Diamond、Patagonia、Osprey、The North Face、MAMMUT、迪卡侬(Decathlon) 等
- 如果无法确定母公司，brand 留空字符串

## 可用根类型（相当于分类）
{cats_desc}

## 可用类型（物品子类型，树形层级）
{types_desc}

## 输出字段说明
对于每个物品，输出以下字段：
- type_name: 从上面的类型列表中选择最合适的类型名称，没有合适的留 null
- attrs: 物品属性对象，必须包含以下基础属性：
  - name: 物品名称（简洁，如"冲锋衣"、"登山杖"）
  - brand: 品牌（母公司名，如"迪卡侬"、"始祖鸟"，未知留空字符串）
  - model: 型号（如"Beta LT"、"TREK 100"，未知留空字符串）
  - notes: 备注（通常为空字符串）
  - default_qty: 默认数量，通常为 1
  - 以及其他注册属性：
{attrs_desc}

## attrs 自由格式（重要）
- attrs 是一个自由的 JSON 对象，除了上面列出的注册属性外，你还可以添加任意自定义键值对
- 从订单 OCR 文本中提取的信息都放入 attrs
- 推荐的自定义 key: product_code(货号), sub_brand(子品牌), color(颜色), size(尺码), price(价格/元), order_number(订单号)

## 订单 OCR 文本处理规则
如果输入包含订单号、价格、颜色、尺码等信息，按订单处理：

- **product_code / 货号**: 迪卡侬商品唯一代码（一串字母+数字，不是 MH100 这种产品线名）
  如果在 OCR 文本中找到类似"货号:2985376"的字段则提取，否则留空
  - 注意：订单号（如 cn1755410837578350）不是货号，不要误填
- **sub_brand / 子品牌**: 迪卡侬的子品牌/产品线名，如 MH100、MH500、TREK 100、RCR
  以及 QUECHUA、FORCLAZ、VAN RYSEL、SIMOND 等
  - **严禁从 name 中删除子品牌名**。name 必须保留 OCR 原文的完整商品名
  - 例如 OCR 是"QUECHUA男士速干徒步长裤 MH500 AF"，name 就是"QUECHUA男士速干徒步长裤 MH500 AF"原样，不要缩写成"男士速干徒步长裤"
- **color / 颜色**: 如"灰色"、"珍珠米色"、"雪白"
- **size / 尺码**: 如"2XL"、"XL/2XL"、"均码"，保留原始格式
- **price / 价格**: 迪卡侬价格通常以"分"为单位显示（¥2990 = ¥29.90），需要转换为"元"的数值
  - 例如：¥9990 → price: 99.90，¥2990 → price: 29.90，¥39990 → price: 399.90
- **order_number / 订单号**: 如"cn1755410837578350"，如果识别到订单号，所有物品都应包含此字段
- **数量**: 订单中的 "X1"、"X2" 对应 default_qty
- 一个订单行 = 一个物品。同货号但不同颜色/尺码的，应拆分为独立物品
- 从 OCR 文本中识别列标题模式（如"品名 颜色 尺码 数量 单价"）

## Excel 表格数据处理规则
如果输入是"列名: 值 | 列名: 值"格式的多行数据，按表格处理：

- **列语义分析**：阅读列名和每列的数据内容，判断该列的语义：
  - "商品名称"/"品名"/"产品"/"名称" → name
  - "品牌"/"牌子"/"商标" → brand
  - "型号"/"规格"/"款式" → model
  - "价格"/"金额"/"售价"/"单价" → price（number，去掉￥等符号，如"分"为单位则转换为元）
  - "数量"/"个数" → default_qty
  - "颜色"/"颜色分类" → color
  - "尺码"/"规格"/"大小" → size
  - "备注"/"说明"/"描述" → notes
  - "货号"/"商品编码" → product_code
  - 其他列 → 映射到已有属性或放入 attrs 自由字段

- **保留所有原始信息**：每列的数据都要有去处，不要丢弃任何信息
- **name 保持完整**：保留商品/物品的完整原始名称，包括子品牌、产品线名
- **品牌推断**：先查有无品牌列，没有则根据 name 和其他信息推断（运用你的商品知识）
- **无法映射的列**：放入 attrs 作为自定义 key
- **在 new_attrs 中提议新建属性**：对于 attrs 中新增的自定义 key，在 new_attrs 中提议创建对应的属性定义
- 空行跳过，完全重复的行只保留一条

## 通用规则
1. 尽量根据品牌和型号推断物品属性
2. 如果用户只说了一个概括性的描述，拆分成独立物品（如"登山装备：冲锋衣、登山杖"→ 两个物品）
3. type_name 从类型列表中选择最合适的；如果没有匹配的，请给出一个合理的简短类型名（2-4字，如"手表"、"头灯"）
4. 数值属性不确定时给出合理估计值，而不是全填 0
5. 品牌规则优先：子品牌 ≠ 品牌，请根据品牌识别规则正确填写 brand 字段
6. **属性适用范围**：每个属性都有"适用分类"标注（如"服装"、"服装/装备"、"全局"）。只填充该物品分类对应的属性值，不适用该分类的属性不要填写（留空或不包含在 attrs 中）。例如：食品类物品不需要填写保暖、防水、身体部位等服装属性

请以 JSON 格式输出，格式为：{{"items": [...]}}"#
    )
}

pub async fn parse_items(
    State(pool): State<SqlitePool>,
    Json(body): Json<AiParseRequest>,
) -> Result<Json<AiParseResponse>, AppError> {
    let types = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;
    let attr_defs = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order",
    )
    .fetch_all(&pool)
    .await?;

    let config = load_llm_config(&pool, "parse").await?;
    let system_prompt = build_system_prompt(&types, &attr_defs);
    let content = call_llm(&config, &system_prompt, &body.text).await?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI JSON output: {}. Raw: {}", e, content))?;

    let items_val = parsed
        .get("items")
        .ok_or_else(|| anyhow::anyhow!("AI output missing 'items' field"))?;

    let mut items: Vec<AiParsedItem> = serde_json::from_value(items_val.clone())
        .map_err(|e| anyhow::anyhow!("Failed to deserialize items: {}. Raw: {}", e, items_val))?;

    for item in &mut items {
        resolve_parsed_item(item, &types);
    }

    let new_types = auto_create_types_for_items(&mut items, &pool).await?;

    Ok(Json(AiParseResponse { items, new_types }))
}

// ── Organize ──

fn build_organize_prompt(
    types: &[Type],
    items: &[Item],
    attr_defs: &[AttributeDefinition],
) -> String {
    let root_types: Vec<&Type> = types.iter().filter(|t| t.parent_id.is_none()).collect();

    let mut cats_desc = String::new();
    for rt in &root_types {
        cats_desc.push_str(&format!("- id:{} {}\n", rt.id, rt.name));
    }

    let mut types_desc = String::new();
    for t in types {
        let parent_name = t.parent_id
            .and_then(|pid| types.iter().find(|rt| rt.id == pid))
            .map(|p| p.name.as_str())
            .unwrap_or("-");
        types_desc.push_str(&format!("- id:{} {} (父级: {})\n", t.id, t.name, parent_name));
    }

    let mut attrs_desc = String::new();
    for ad in attr_defs {
        attrs_desc.push_str(&format!(
            "- key:{} label:{} type:{} category_scope:{}\n",
            ad.key,
            ad.label,
            ad.attr_type,
            if ad.category_scope.is_empty() { "全局" } else { &ad.category_scope }
        ));
    }

    // Basic fields already displayed in item line; don't repeat in attrs section
    let basic_keys: &[&str] = &["name", "brand", "model", "notes", "default_qty"];

    let mut items_desc = String::new();
    for item in items {
        let type_ref = item
            .type_id
            .and_then(|tid| types.iter().find(|t| t.id == tid));
        let type_name = type_ref.map(|t| t.name.as_str()).unwrap_or("");
        // Get root type from the type's parent chain
        let root_name = type_ref
            .and_then(|t| {
                let parent_id = t.parent_id?;
                types.iter().find(|rt| rt.id == parent_id).map(|rt| rt.name.as_str())
            })
            .unwrap_or("-");
        let item_name = item.attr_str("name");
        let brand = item.attr_str("brand");
        let model = item.attr_str("model");
        let notes = item.attr_str("notes");
        items_desc.push_str(&format!(
            "- id:{} name:\"{}\" brand:\"{}\" model:\"{}\" root_type:\"{}\" type:\"{}\" notes:\"{}\"",
            item.id, item_name, brand, model, root_name, type_name, notes
        ));

        // Append non-basic attrs values that are set
        if let Some(obj) = item.attrs.as_object() {
            let mut extra_parts: Vec<String> = Vec::new();
            for (k, v) in obj {
                if basic_keys.contains(&k.as_str()) {
                    continue;
                }
                if v.is_null() {
                    continue;
                }
                let val_str = if v.is_string() {
                    v.as_str().unwrap_or("").to_string()
                } else {
                    v.to_string()
                };
                if val_str.is_empty() {
                    continue;
                }
                // Find label for this key
                let label = attr_defs
                    .iter()
                    .find(|ad| ad.key == *k)
                    .map(|ad| ad.label.as_str())
                    .unwrap_or(k.as_str());
                extra_parts.push(format!("{}={}", label, val_str));
            }
            if !extra_parts.is_empty() {
                items_desc.push_str(&format!(" | {}", extra_parts.join(", ")));
            }
        }
        items_desc.push('\n');
    }

    format!(
        r#"你是一个户外装备数据库管理助手。以下是当前物品库中的所有物品，请检查数据质量问题并提出整理建议。

## 可用根类型（相当于分类）
{cats_desc}

## 可用类型（物品子类型，树形层级）
{types_desc}

## 可用属性定义
{attrs_desc}

## 当前物品列表（含已设置的属性值）
{items_desc}

## 检查项目
1. **合并物品需拆分**：名称中包含"和"、"+"等连接词的物品，应拆分为独立物品（如"墨镜和眼镜布"→"墨镜"+"眼镜布"）
2. **字段错位**：信息放在了错误的字段中。常见现象：
   - model（型号）字段填了材质/类型等非型号信息（如 model="羊毛"），应移至 notes 或 attrs 字段
3. **缺少类型**：物品当前没有类型但应该有（根据名称可推断子类型类型）。
   此时在 fields 中增加 "type_name": "类型名"。这是 fields 中出现 type_name 的唯一场景——已有类型的物品不要修改 type_name。
4. **重复物品**：名称/品牌/型号完全相同的物品
5. **品类特有属性缺失**（重要）：检查物品是否缺少该分类的关键属性
   - 服装/装备（分类id=1或2）：缺少 body_parts（主覆盖部位）的，根据名称和类型推断补充；同时可补充 body_parts_secondary（副覆盖部位，逗号分隔0~多个）。覆盖部位选项：头/眼/口/颈/躯干/手臂/手/腿/脚/腰/臀/全身
   - 营养（分类id=3）：缺少 food_type（食品类型）的，根据名称推断。选项：能量胶/能量棒/巧克力/果泥/威化/饼干/坚果/肉干/饮品/补剂/其他
   - 电子（分类id=4）：缺少 electronics_type（电子类型）的，根据名称推断。选项：照明/通讯/导航/摄影/电源/穿戴/其他
   - 补充属性时，通过 update action 的 fields.attrs 设置对应的 key 值（如 "body_parts": "躯干", "food_type": "能量胶"）

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
    "attrs": {{"name": "新名称（不改则不包含此字段）", "brand": "新品牌", "body_parts": "躯干", ...}}
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
    {{"type_name": "类型名", "attrs": {{"name": "物品1", "brand": "", "model": "", "notes": "", "default_qty": 1}}}},
    {{"attrs": {{"name": "物品2", ...}}}}
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
4. split 的 new_items 中的 type_name 必须从上面的类型列表中选择
5. 如果没有发现任何问题，返回 {{"actions": []}}
6. 保守一些，只提出明显的问题，不要过度修改
7. **不要精简名称**：名称中包含品牌名（如"迪卡侬SIMOND软壳"）是用户的命名习惯，不是问题，不要建议移除
8. **类型与名称是独立维度**：
   - 类型用于分组筛选、模板匹配和分类浏览；名称用于唯一识别具体物品
   - 类型词出现在名称中是正常且正确的（如"VAN RYSEL 骑行头盔"类型"骑行头盔"），两者功能不同，不构成冗余
   - **禁止删除正确类型**（即不要将 type_name 设为 null 或空），除非类型分类归属完全错误（如服装标了食品）；仅因名称中含相同词而清除类型是错误操作
9. **每次 update 只解决一个问题**：不要在 update 中同时修改不相关的字段。例如补充 body_parts 时只改 attrs 中的 body_parts，不要顺带修改 type_name 或 name。如有多个独立问题，拆分为独立的 update action。"#
    )
}

pub async fn organize_preview(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrganizePreviewRequest>,
) -> Result<Json<OrganizePreviewResponse>, AppError> {
    let types = sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;

    let items: Vec<Item> = match &body.item_ids {
        Some(ids) if !ids.is_empty() => {
            let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
            let sql = format!(
                "SELECT id, type_id, attrs FROM items WHERE id IN ({}) ORDER BY id",
                placeholders.join(", ")
            );
            let mut query = sqlx::query_as::<_, Item>(&sql);
            for id in ids {
                query = query.bind(id);
            }
            query.fetch_all(&pool).await?
        }
        _ => {
            sqlx::query_as::<_, Item>("SELECT id, type_id, attrs FROM items ORDER BY id")
                .fetch_all(&pool)
                .await?
        }
    };

    let attr_defs = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order",
    )
    .fetch_all(&pool)
    .await?;

    let config = load_llm_config(&pool, "organize").await?;
    let system_prompt = build_organize_prompt(&types, &items, &attr_defs);
    let content = call_llm(&config, &system_prompt, "请分析以上物品列表，输出整理建议。").await?;

    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| anyhow::anyhow!("Failed to parse AI JSON: {}. Raw: {}", e, content))?;

    let actions_val = parsed
        .get("actions")
        .ok_or_else(|| anyhow::anyhow!("AI output missing 'actions' field"))?;

    let mut actions: Vec<OrganizeAction> = serde_json::from_value(actions_val.clone())
        .map_err(|e| {
            anyhow::anyhow!("Failed to deserialize actions: {}. Raw: {}", e, actions_val)
        })?;

    // Resolve type_name to IDs in actions
    for action in &mut actions {
        match action {
            OrganizeAction::Update { fields, .. } => {
                if let Some(ref type_name) = fields.type_name {
                    fields.type_id = Some(resolve_type_id(type_name, &types));
                }
            }
            OrganizeAction::Split { new_items, .. } => {
                for item in new_items.iter_mut() {
                    resolve_parsed_item(item, &types);
                }
            }
            OrganizeAction::Delete { .. } => {}
        }
    }

    // Auto-create types for split new_items
    let mut tag_items_to_process: Vec<AiParsedItem> = Vec::new();
    for action in &actions {
        if let OrganizeAction::Split { new_items, .. } = action {
            for item in new_items {
                if item.type_name.is_some() && item.type_id.is_none() {
                    tag_items_to_process.push(item.clone());
                }
            }
        }
    }
    let new_types = auto_create_types_for_items(&mut tag_items_to_process, &pool).await?;

    // Back-fill type_ids from created types
    for action in &mut actions {
        if let OrganizeAction::Split { new_items, .. } = action {
            for item in new_items.iter_mut() {
                if item.type_name.is_some() && item.type_id.is_none() {
                    if let Some(ref tn) = item.type_name {
                        if let Some(tag) = new_types.iter().find(|t| {
                            t.name.to_lowercase() == tn.to_lowercase()
                        }) {
                            item.type_id = Some(tag.id);
                        }
                    }
                }
            }
        }
    }

    // Also handle update actions with unresolved type_name → create types
    let mut update_new_types: Vec<Type> = Vec::new();
    for action in &mut actions {
        if let OrganizeAction::Update { fields, .. } = action {
            if let Some(ref type_name) = fields.type_name {
                if fields.type_id == Some(None) || fields.type_id.is_none() {
                    // Check if type exists
                    let existing = resolve_type_id(type_name, &types);
                    if let Some(tid) = existing {
                        fields.type_id = Some(Some(tid));
                    } else {
                        // Also check newly created types
                        let from_new = new_types
                            .iter()
                            .chain(update_new_types.iter())
                            .find(|t| {
                                t.name.to_lowercase() == type_name.to_lowercase()
                            });
                        if let Some(t) = from_new {
                            fields.type_id = Some(Some(t.id));
                        } else {
                            // Create the type
                            let max_sort: Option<(i64,)> = sqlx::query_as(
                                "SELECT COALESCE(MAX(sort_order), 0) FROM types",
                            )
                            .fetch_optional(&pool)
                            .await?;
                            let sort_order = max_sort.map(|r| r.0).unwrap_or(0) + 1;

                            let tag = sqlx::query_as::<_, Type>(
                                "INSERT INTO types (name, sort_order) VALUES (?, ?) \
                                 ON CONFLICT(name) DO UPDATE SET name=name \
                                 RETURNING id, name, sort_order, parent_id",
                            )
                            .bind(type_name)
                            .bind(sort_order)
                            .fetch_one(&pool)
                            .await?;

                            fields.type_id = Some(Some(tag.id));
                            update_new_types.push(tag);
                        }
                    }
                }
            }
        }
    }

    let mut all_new_types = new_types;
    all_new_types.extend(update_new_types);

    Ok(Json(OrganizePreviewResponse {
        actions,
        new_types: all_new_types,
    }))
}

pub async fn organize_apply(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrganizeApplyRequest>,
) -> Result<Json<OrganizeApplyResponse>, AppError> {
    let mut updated: i64 = 0;
    let mut created: i64 = 0;
    let mut deleted: i64 = 0;

    let mut tx = pool.begin().await?;

    // Pre-fetch valid type IDs to validate AI-provided references
    let valid_type_ids: std::collections::HashSet<i64> =
        sqlx::query_as::<_, (i64,)>("SELECT id FROM types")
            .fetch_all(&mut *tx)
            .await?
            .into_iter()
            .map(|r| r.0)
            .collect();

    for action in &body.actions {
        match action {
            OrganizeAction::Update { item_id, fields, .. } => {
                apply_update_fields(&mut tx, *item_id, fields, &valid_type_ids)
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
                    let type_id = new_item.type_id.filter(|id| valid_type_ids.contains(id));

                    let attrs_str = serde_json::to_string(&new_item.attrs).unwrap_or_else(|_| "{}".to_string());
                    let result = sqlx::query(
                        "INSERT INTO items (type_id, attrs) VALUES (?, ?)",
                    )
                    .bind(type_id)
                    .bind(attrs_str)
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
    }))
}

async fn apply_update_fields(
    tx: &mut sqlx::Transaction<'_, sqlx::Sqlite>,
    item_id: i64,
    fields: &OrganizeUpdateFields,
    valid_type_ids: &std::collections::HashSet<i64>,
) -> Result<(), AppError> {
    // Merge attrs JSON if provided
    if let Some(ref new_attrs) = fields.attrs {
        // Read existing attrs
        let existing_attrs: Option<(String,)> = sqlx::query_as(
            "SELECT attrs FROM items WHERE id = ?",
        )
        .bind(item_id)
        .fetch_optional(&mut **tx)
        .await?;

        if let Some((existing_str,)) = existing_attrs {
            let existing: serde_json::Value = serde_json::from_str(&existing_str).unwrap_or_default();
            let mut merged = existing.clone();
            if let serde_json::Value::Object(ref new_obj) = new_attrs {
                if let serde_json::Value::Object(ref mut merged_obj) = merged {
                    for (k, v) in new_obj {
                        merged_obj.insert(k.clone(), v.clone());
                    }
                }
            }
            let merged_str = serde_json::to_string(&merged).unwrap_or_default();
            sqlx::query("UPDATE items SET attrs = ? WHERE id = ?")
                .bind(&merged_str)
                .bind(item_id)
                .execute(&mut **tx)
                .await?;
        }
    }

    if let Some(ref type_id_opt) = fields.type_id {
        let safe_type_id = match type_id_opt {
            Some(id) if valid_type_ids.contains(id) => &Some(*id),
            Some(_) => &None,
            None => &None,
        };
        sqlx::query("UPDATE items SET type_id = ? WHERE id = ?")
            .bind(safe_type_id)
            .bind(item_id)
            .execute(&mut **tx)
            .await?;
    }

    Ok(())
}

// ── Streaming LLM call ──

#[derive(serde::Deserialize)]
struct ChatStreamChunk {
    choices: Vec<ChatStreamChoice>,
}

#[derive(serde::Deserialize)]
struct ChatStreamChoice {
    delta: ChatStreamDelta,
}

#[derive(serde::Deserialize)]
struct ChatStreamDelta {
    content: Option<String>,
}

fn build_streaming_system_prompt(
    types: &[Type],
    attr_defs: &[AttributeDefinition],
) -> String {
    let mut base = build_system_prompt(types, attr_defs);
    base.push_str("\n\n## 输出格式要求\n请先简要说明你的解析思路和判断依据（1-3句话），然后单独一行输出 `---JSON---`，之后输出纯 JSON 对象（不要用 markdown 代码块包裹）。\n\nJSON 格式：{\"items\": [...]}\n- items: 解析出的物品列表，每个物品包含 type_name 和 attrs\n- 只需要输出 items 数组，不需要输出 new_attrs 字段");
    base
}

/// Call LLM with streaming enabled. Sends `Thinking` events via `tx` as tokens arrive.
/// Returns the full accumulated response text.
pub(crate) async fn call_llm_stream(
    config: &LlmConfig,
    system_prompt: &str,
    user_prompt: &str,
    tx: &tokio::sync::mpsc::UnboundedSender<SseEvent>,
) -> Result<String, AppError> {
    let chat_req = ChatRequest {
        model: config.model.clone(),
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
        response_format: None,
        tools: None,
        stream: Some(true),
    };

    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", config.base_url.trim_end_matches('/'));

    let resp = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.api_key))
        .json(&chat_req)
        .send()
        .await
        .map_err(|e| anyhow::anyhow!("Failed to call AI API: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        let msg = format!("AI API returned {}: {}", status, text);
        let _ = tx.send(SseEvent::Error { message: msg.clone() });
        return Err(AppError::Internal(anyhow::anyhow!("{}", msg)));
    }

    let provider_name = config.provider_name.clone();
    let model = config.model.clone();

    let mut full_text = String::new();
    let mut buffer = String::new();
    let mut combined_stream = resp.bytes_stream();

    while let Some(chunk_result) = combined_stream.next().await {
        match chunk_result {
            Ok(chunk) => {
                let text = String::from_utf8_lossy(&chunk);
                buffer.push_str(&text);

                while let Some(line_end) = buffer.find('\n') {
                    let line = buffer[..line_end].trim().to_string();
                    buffer = buffer[line_end + 1..].to_string();

                    if line.is_empty() || line.starts_with(':') {
                        continue;
                    }

                    if let Some(data) = line.strip_prefix("data: ") {
                        if data == "[DONE]" {
                            break;
                        }
                        if let Ok(chunk) = serde_json::from_str::<ChatStreamChunk>(data) {
                            if let Some(content) = chunk
                                .choices
                                .first()
                                .and_then(|c| c.delta.content.as_ref())
                            {
                                full_text.push_str(content);
                                let _ = tx.send(SseEvent::Thinking {
                                    content: content.clone(),
                                });
                            }
                        }
                    }
                }
            }
            Err(e) => {
                let msg = format!("Stream error: {}", e);
                let _ = tx.send(SseEvent::Error { message: msg.clone() });
                return Err(AppError::Internal(anyhow::anyhow!("{}", msg)));
            }
        }
    }

    tracing::info!(
        "[LLM Stream] {} 模型={} 响应({} 字符): {}",
        provider_name,
        model,
        full_text.chars().count(),
        &full_text[..full_text.len().min(500)]
    );

    Ok(full_text)
}

/// Load the active LLM config for a given task from DB.
/// Falls back to environment variables if no DB config exists.
pub(crate) async fn load_llm_config(pool: &SqlitePool, task: &str) -> Result<LlmConfig, AppError> {
    // Try DB first
    if let Some(cfg) = llm_configs::get_active(pool, task).await {
        if !cfg.api_key.is_empty() {
            return Ok(cfg);
        }
        // DB config exists but api_key is empty — use env var for key, DB for rest
        let api_key = std::env::var("OPENAI_API_KEY").ok();
        if let Some(key) = api_key {
            return Ok(LlmConfig {
                api_key: key,
                ..cfg
            });
        }
        // No env key either — return DB config as-is (will fail at API call if key is empty)
        return Ok(cfg);
    }

    // Fallback to environment variables
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("未配置 LLM API Key。请在设置页配置，或设置 OPENAI_API_KEY 环境变量。"))?;
    let base_url = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model =
        std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

    Ok(LlmConfig {
        id: 0,
        task: task.to_string(),
        provider_name: "env".to_string(),
        base_url,
        api_key,
        model,
        is_active: true,
        created_at: String::new(),
        updated_at: String::new(),
    })
}

/// Extract items JSON from the LLM response text.
pub(crate) fn extract_items_from_text(full_text: &str) -> Result<Vec<AiParsedItem>, String> {
    // Try to find JSON after ---JSON--- marker
    let json_str = if let Some(pos) = full_text.find("---JSON---") {
        let after = &full_text[pos + "---JSON---".len()..];
        after.trim().to_string()
    } else if let Some(pos) = full_text.find("{\"items\"") {
        full_text[pos..].to_string()
    } else if let Some(pos) = full_text.find('{') {
        full_text[pos..].to_string()
    } else {
        return Err("AI 返回内容中未找到 JSON 结果".to_string());
    };

    // Strip markdown code fences if present
    let json_str = json_str.trim();
    let json_str = json_str
        .strip_prefix("```json")
        .or_else(|| json_str.strip_prefix("```"))
        .map(|s| s.trim())
        .unwrap_or(json_str);
    let json_str = json_str
        .strip_suffix("```")
        .map(|s| s.trim())
        .unwrap_or(json_str);

    let parsed: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|e| format!("解析 AI JSON 失败: {}. Raw: {}", e, json_str))?;

    let items_val = parsed
        .get("items")
        .ok_or_else(|| "AI 输出缺少 'items' 字段".to_string())?;

    let items: Vec<AiParsedItem> = serde_json::from_value(items_val.clone())
        .map_err(|e| format!("解析物品列表失败: {}. Raw: {}", e, items_val))?;

    Ok(items)
}

/// Extract new attribute definitions from the AI response text.
fn extract_new_attr_defs_from_text(full_text: &str) -> Vec<AttributeDefinition> {
    let json_str = if let Some(pos) = full_text.find("---JSON---") {
        let after = &full_text[pos + "---JSON---".len()..];
        after.trim().to_string()
    } else if let Some(pos) = full_text.find('{') {
        full_text[pos..].to_string()
    } else {
        return Vec::new();
    };

    let json_str = json_str.trim();
    let json_str = json_str
        .strip_prefix("```json")
        .or_else(|| json_str.strip_prefix("```"))
        .map(|s| s.trim())
        .unwrap_or(json_str);
    let json_str = json_str
        .strip_suffix("```")
        .map(|s| s.trim())
        .unwrap_or(json_str);

    let parsed: serde_json::Value = match serde_json::from_str(json_str) {
        Ok(v) => v,
        Err(_) => return Vec::new(),
    };

    let new_attrs_val = match parsed.get("new_attrs") {
        Some(v) => v,
        None => return Vec::new(),
    };

    let new_attrs: Vec<serde_json::Value> = match new_attrs_val.as_array() {
        Some(a) => a.clone(),
        None => return Vec::new(),
    };

    let mut result: Vec<AttributeDefinition> = Vec::new();
    for attr in new_attrs {
        let key = attr
            .get("key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let label = attr
            .get("label")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        let attr_type = attr
            .get("attr_type")
            .and_then(|v| v.as_str())
            .unwrap_or("text")
            .to_string();
        let config = attr
            .get("config")
            .map(|v| {
                if v.is_string() {
                    v.as_str().unwrap_or("{}").to_string()
                } else {
                    v.to_string()
                }
            })
            .unwrap_or_else(|| "{}".to_string());

        if key.is_empty() || label.is_empty() {
            continue;
        }

        result.push(AttributeDefinition {
            id: 0,
            key,
            label,
            attr_type,
            config,
            category_scope: String::new(),
            type_scope: String::new(),
            sort_order: 0,
            is_identity: false,
            is_required: false,
            default_value: String::new(),
            search_weight: 0,
        });
    }

    result
}

/// Auto-detect custom attribute keys from parsed item attrs that don't exist
/// in the known attribute_definitions, and generate new AttributeDefinition entries.
fn extract_new_attr_defs_from_items(
    items: &[AiParsedItem],
    existing_attr_defs: &[AttributeDefinition],
) -> Vec<AttributeDefinition> {
    let existing_keys: std::collections::HashSet<&str> =
        existing_attr_defs.iter().map(|a| a.key.as_str()).collect();
    let basic_keys: std::collections::HashSet<&str> =
        ["name", "brand", "model", "notes", "default_qty"]
            .iter()
            .copied()
            .collect();

    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();

    for item in items {
        if let Some(obj) = item.attrs.as_object() {
            for key in obj.keys() {
                if basic_keys.contains(key.as_str()) || existing_keys.contains(key.as_str()) {
                    continue;
                }
                if seen.insert(key.clone()) {
                    result.push(AttributeDefinition {
                        id: 0,
                        key: key.clone(),
                        label: key.replace('_', " "),
                        attr_type: "text".to_string(),
                        config: "{}".to_string(),
                        category_scope: String::new(),
                        type_scope: String::new(),
                        sort_order: 0,
                        is_identity: false,
                        is_required: false,
                        default_value: String::new(),
                        search_weight: 0,
                    });
                }
            }
        }
    }

    result
}

// ── Streaming Parse Items Handler ──

pub async fn parse_items_stream(
    State(pool): State<SqlitePool>,
    Json(body): Json<AiParseRequest>,
) -> Sse<impl futures_core::Stream<Item = Result<Event, Infallible>>> {
    let (tx, rx) = tokio::sync::mpsc::unbounded_channel::<SseEvent>();

    // Spawn background task to do the AI call + DB work
    tokio::spawn(async move {
        let types = match sqlx::query_as::<_, Type>("SELECT id, name, sort_order, parent_id FROM types ORDER BY sort_order")
            .fetch_all(&pool)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("加载类型失败: {:#}", e),
                });
                return;
            }
        };

        let attr_defs = match sqlx::query_as::<_, AttributeDefinition>(
            "SELECT * FROM attribute_definitions ORDER BY sort_order",
        )
        .fetch_all(&pool)
        .await
        {
            Ok(a) => a,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("加载属性定义失败: {:#}", e),
                });
                return;
            }
        };

        let _ = tx.send(SseEvent::Progress {
            message: "AI 正在分析...".to_string(),
        });

        let config = match load_llm_config(&pool, "parse").await {
            Ok(cfg) => cfg,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("{:?}", e),
                });
                drop(tx);
                return;
            }
        };

        let system_prompt = build_streaming_system_prompt(&types, &attr_defs);

        let full_text = match call_llm_stream(&config, &system_prompt, &body.text, &tx).await {
            Ok(text) => text,
            Err(_) => {
                // Error already sent via tx in call_llm_stream
                drop(tx);
                return;
            }
        };

        // Parse items from the full text
        let mut items = match extract_items_from_text(&full_text) {
            Ok(items) => items,
            Err(msg) => {
                let _ = tx.send(SseEvent::Error { message: msg });
                drop(tx);
                return;
            }
        };

        // Resolve types
        for item in &mut items {
            resolve_parsed_item(item, &types);
        }

        // Auto-create tags
        let new_types = match auto_create_types_for_items(&mut items, &pool).await {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("创建类型失败: {:#?}", e),
                });
                drop(tx);
                return;
            }
        };

        // Extract new attribute definitions from AI response + auto-detect from item attrs
        let mut new_attr_defs = extract_new_attr_defs_from_text(&full_text);
        let auto_attrs = extract_new_attr_defs_from_items(&items, &attr_defs);
        // Merge: add auto-detected attrs that aren't already in the AI-provided list
        for aa in auto_attrs {
            if !new_attr_defs.iter().any(|a| a.key == aa.key) {
                new_attr_defs.push(aa);
            }
        }

        // Insert or ignore new attr defs into DB
        let mut saved_attr_defs: Vec<AttributeDefinition> = Vec::new();
        for attr in &new_attr_defs {
            match sqlx::query_as::<_, AttributeDefinition>(
                "INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, category_scope, type_scope, sort_order, is_identity, is_required, default_value, search_weight) \
                 VALUES (?, ?, ?, ?, '', '', COALESCE((SELECT MAX(sort_order) FROM attribute_definitions), 0) + 1, 0, 0, '', 0) \
                 RETURNING *",
            )
            .bind(&attr.key)
            .bind(&attr.label)
            .bind(&attr.attr_type)
            .bind(&attr.config)
            .fetch_optional(&pool)
            .await
            {
                Ok(Some(a)) => saved_attr_defs.push(a),
                Ok(None) => {
                    // Already exists, fetch existing
                    if let Ok(Some(existing)) = sqlx::query_as::<_, AttributeDefinition>(
                        "SELECT * FROM attribute_definitions WHERE key = ?",
                    )
                    .bind(&attr.key)
                    .fetch_optional(&pool)
                    .await
                    {
                        saved_attr_defs.push(existing);
                    }
                }
                Err(e) => {
                    let _ = tx.send(SseEvent::Error {
                        message: format!("保存属性定义失败: {:#?}", e),
                    });
                    drop(tx);
                    return;
                }
            }
        }

        let _ = tx.send(SseEvent::Result { items, new_types, new_attr_defs: saved_attr_defs });
        drop(tx);
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok(Event::default().data(data))
    });

    Sse::new(stream)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_types() -> Vec<Type> {
        vec![
            Type { id: 1, name: "冲锋衣".to_string(), parent_id: None, sort_order: 1 },
            Type { id: 2, name: "登山杖".to_string(), parent_id: None, sort_order: 1 },
            Type { id: 3, name: "头灯".to_string(), parent_id: None, sort_order: 2 },
        ]
    }

    #[test]
    fn resolve_type_exact() {
        let types = sample_types();
        assert_eq!(resolve_type_id("冲锋衣", &types), Some(1));
        assert_eq!(resolve_type_id("登山杖", &types), Some(2));
    }

    #[test]
    fn resolve_type_unknown() {
        let types = sample_types();
        assert_eq!(resolve_type_id("不存在的类型", &types), None);
    }

    #[test]
    fn resolve_parsed_item_sets_ids() {
        let types = sample_types();
        let mut item = AiParsedItem {
            type_name: Some("冲锋衣".to_string()),
            type_id: None,
            attrs: Default::default(),
        };
        resolve_parsed_item(&mut item, &types);
        assert_eq!(item.type_id, Some(1));
    }

    #[test]
    fn resolve_parsed_item_no_names() {
        let types = sample_types();
        let mut item = AiParsedItem {
            type_name: None,
            type_id: None,
            attrs: Default::default(),
        };
        resolve_parsed_item(&mut item, &types);
        assert_eq!(item.type_id, None);
    }

    // ── extract_items_from_text ──

    #[test]
    fn extract_items_with_marker() {
        let text = "Some thinking...\n---JSON---\n{\"items\":[{\"type_name\":\"冲锋衣\",\"attrs\":{\"name\":\"冲锋衣\"}}]}";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].attrs.get("name").and_then(|v| v.as_str()), Some("冲锋衣"));
    }

    #[test]
    fn extract_items_without_marker() {
        let text = "{\"items\":[{\"type_name\":\"登山杖\",\"attrs\":{\"name\":\"登山杖\"}}]}";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].type_name.as_deref(), Some("登山杖"));
    }

    #[test]
    fn extract_items_with_markdown_fence() {
        let text = "Thoughts...\n---JSON---\n```json\n{\"items\":[{\"type_name\":\"软壳\",\"attrs\":{\"name\":\"软壳\"}}]}\n```";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].attrs.get("name").and_then(|v| v.as_str()), Some("软壳"));
    }

    #[test]
    fn extract_items_empty() {
        let text = "---JSON---\n{\"items\":[]}";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 0);
    }
}