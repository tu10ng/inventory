use axum::extract::State;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::Json;
use sqlx::SqlitePool;
use std::convert::Infallible;
use tokio_stream::wrappers::UnboundedReceiverStream;
use tokio_stream::StreamExt as _;

use crate::error::AppError;
use crate::models::{
    AiParseRequest, AiParseResponse, AiParsedItem, AttributeDefinition, Category, Item,
    OrganizeAction, OrganizeApplyRequest, OrganizeApplyResponse, OrganizePreviewRequest,
    OrganizePreviewResponse, OrganizeUpdateFields, SseEvent, Tag,
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

// ── Shared helpers ──

pub(crate) fn resolve_category_id(cat_name: &str, categories: &[Category]) -> i64 {
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

pub(crate) fn resolve_tag_id(tag_name: &str, cat_id: Option<i64>, tags: &[Tag]) -> Option<i64> {
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

pub(crate) fn resolve_parsed_item(item: &mut AiParsedItem, categories: &[Category], tags: &[Tag]) {
    if let Some(ref cat_name) = item.category_name {
        item.category_id = Some(resolve_category_id(cat_name, categories));
    }
    if let Some(ref tag_name) = item.tag_name {
        item.tag_id = resolve_tag_id(tag_name, item.category_id, tags);
    }
}

/// Collect unique (tag_name, category_id) pairs that need creation,
/// insert them, back-fill tag_id, and return the new tags.
pub(crate) async fn auto_create_tags_for_items(
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
    let api_base = std::env::var("OPENAI_BASE_URL")
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
        stream: None,
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
        let scope_info = if ad.category_scope.is_empty() {
            "全局".to_string()
        } else {
            let ids: Vec<&str> = ad.category_scope.split(',').collect();
            let names: Vec<String> = ids
                .iter()
                .filter_map(|id| {
                    let id_num: i64 = id.trim().parse().ok()?;
                    categories.iter().find(|c| c.id == id_num).map(|c| c.name.clone())
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

## 可用分类
{cats_desc}

## 可用标签（物品子类型）
{tags_desc}

## 输出字段说明
对于每个物品，输出以下字段：
- category_name: 从上面的分类列表中选择最合适的分类名称
- tag_name: 从上面的标签列表中选择最合适的标签名称，没有合适的留 null
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
3. category_name 必须从上面的分类列表中选择
4. tag_name 从标签列表中选择最合适的；如果没有匹配的，请给出一个合理的简短标签名（2-4字，如"手表"、"头灯"）
5. 数值属性不确定时给出合理估计值，而不是全填 0
6. 品牌规则优先：子品牌 ≠ 品牌，请根据品牌识别规则正确填写 brand 字段
7. **属性适用范围**：每个属性都有"适用分类"标注（如"服装"、"服装/装备"、"全局"）。只填充该物品分类对应的属性值，不适用该分类的属性不要填写（留空或不包含在 attrs 中）。例如：食品类物品不需要填写保暖、防水、身体部位等服装属性

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

fn build_organize_prompt(
    categories: &[Category],
    tags: &[Tag],
    items: &[Item],
    attr_defs: &[AttributeDefinition],
) -> String {
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
        let cat = categories.iter().find(|c| c.id == item.category_id);
        let cat_name = cat.map(|c| c.name.as_str()).unwrap_or("?");
        let cat_id = item.category_id;
        let tag = item
            .tag_id
            .and_then(|tid| tags.iter().find(|t| t.id == tid));
        let tag_name = tag.map(|t| t.name.as_str()).unwrap_or("");
        let item_name = item.attr_str("name");
        let brand = item.attr_str("brand");
        let model = item.attr_str("model");
        let notes = item.attr_str("notes");
        items_desc.push_str(&format!(
            "- id:{} cat_id:{} name:\"{}\" brand:\"{}\" model:\"{}\" category:\"{}\" tag:\"{}\" notes:\"{}\"",
            item.id, cat_id, item_name, brand, model, cat_name, tag_name, notes
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

## 可用分类
{cats_desc}

## 可用标签（物品子类型）
{tags_desc}

## 可用属性定义
{attrs_desc}

## 当前物品列表（含已设置的属性值）
{items_desc}

## 检查项目
1. **合并物品需拆分**：名称中包含"和"、"+"等连接词的物品，应拆分为独立物品（如"墨镜和眼镜布"→"墨镜"+"眼镜布"）
2. **字段错位**：信息放在了错误的字段中。常见现象：
   - model（型号）字段填了材质/类型等非型号信息（如 model="羊毛"），应移至 notes 或 attrs 字段
3. **缺少标签**：物品当前没有标签但应该有（根据名称可推断子类型标签）。
   此时在 fields 中增加 "tag_name": "标签名"。这是 fields 中出现 tag_name 的唯一场景——已有标签的物品不要修改 tag_name。
4. **分类错误**：物品的分类明显不正确
5. **重复物品**：名称/品牌/型号完全相同的物品
6. **品类特有属性缺失**（重要）：检查物品是否缺少该分类的关键属性
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
    "attrs": {{"name": "新名称（不改则不包含此字段）", "brand": "新品牌", "body_parts": "躯干", ...}},
    "category_name": "新分类名（不改则不包含此字段）"
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
    {{"category_name": "分类", "tag_name": "标签名", "attrs": {{"name": "物品1", "brand": "", "model": "", "notes": "", "default_qty": 1}}}},
    {{"category_name": "分类", "attrs": {{"name": "物品2", ...}}}}
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
7. **不要精简名称**：名称中包含品牌名（如"迪卡侬SIMOND软壳"）是用户的命名习惯，不是问题，不要建议移除
8. **标签与名称是独立维度**：
   - 标签用于分组筛选、模板匹配和分类浏览；名称用于唯一识别具体物品
   - 标签词出现在名称中是正常且正确的（如"VAN RYSEL 骑行头盔"标签"骑行头盔"），两者功能不同，不构成冗余
   - **禁止删除正确标签**（即不要将 tag_name 设为 null 或空），除非标签分类归属完全错误（如服装标了食品）；仅因名称中含相同词而清除标签是错误操作
9. **每次 update 只解决一个问题**：不要在 update 中同时修改不相关的字段。例如补充 body_parts 时只改 attrs 中的 body_parts，不要顺带修改 tag_name、category_name 或 name。如有多个独立问题，拆分为独立的 update action。"#
    )
}

pub async fn organize_preview(
    State(pool): State<SqlitePool>,
    Json(body): Json<OrganizePreviewRequest>,
) -> Result<Json<OrganizePreviewResponse>, AppError> {
    let categories =
        sqlx::query_as::<_, Category>("SELECT * FROM categories ORDER BY sort_order")
            .fetch_all(&pool)
            .await?;
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY sort_order")
        .fetch_all(&pool)
        .await?;

    let items: Vec<Item> = match &body.item_ids {
        Some(ids) if !ids.is_empty() => {
            // Build dynamic IN query with placeholders
            let placeholders: Vec<String> = ids.iter().enumerate().map(|(i, _)| format!("?{}", i + 1)).collect();
            let sql = format!(
                "SELECT id, name, brand, model, category_id, default_qty, notes, tag_id, attrs FROM items WHERE id IN ({}) ORDER BY category_id, name",
                placeholders.join(", ")
            );
            let mut query = sqlx::query_as::<_, Item>(&sql);
            for id in ids {
                query = query.bind(id);
            }
            query.fetch_all(&pool).await?
        }
        _ => {
            sqlx::query_as::<_, Item>("SELECT id, name, brand, model, category_id, default_qty, notes, tag_id, attrs FROM items ORDER BY category_id, name")
                .fetch_all(&pool)
                .await?
        }
    };

    let attr_defs = sqlx::query_as::<_, AttributeDefinition>(
        "SELECT * FROM attribute_definitions ORDER BY sort_order",
    )
    .fetch_all(&pool)
    .await?;

    let system_prompt = build_organize_prompt(&categories, &tags, &items, &attr_defs);
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

                    let attrs_str = serde_json::to_string(&new_item.attrs).unwrap_or_else(|_| "{}".to_string());
                    let name = new_item.attrs.get("name").and_then(|v| v.as_str()).unwrap_or("");
                    let brand = new_item.attrs.get("brand").and_then(|v| v.as_str()).unwrap_or("");
                    let model = new_item.attrs.get("model").and_then(|v| v.as_str()).unwrap_or("");
                    let default_qty = new_item.attrs.get("default_qty").and_then(|v| v.as_i64()).unwrap_or(1);
                    let notes = new_item.attrs.get("notes").and_then(|v| v.as_str()).unwrap_or("");
                    let result = sqlx::query(
                        "INSERT INTO items (name, brand, model, category_id, default_qty, notes, tag_id, attrs) \
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
                    )
                    .bind(name)
                    .bind(brand)
                    .bind(model)
                    .bind(cat_id)
                    .bind(default_qty)
                    .bind(notes)
                    .bind(tag_id)
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
    valid_cat_ids: &std::collections::HashSet<i64>,
    valid_tag_ids: &std::collections::HashSet<i64>,
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
        let safe_tag_id = match tag_id_opt {
            Some(id) if valid_tag_ids.contains(id) => &Some(*id),
            Some(_) => &None,
            None => &None,
        };
        sqlx::query("UPDATE items SET tag_id = ? WHERE id = ?")
            .bind(safe_tag_id)
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
    categories: &[Category],
    tags: &[Tag],
    attr_defs: &[AttributeDefinition],
) -> String {
    let mut base = build_system_prompt(categories, tags, attr_defs);
    base.push_str("\n\n## 输出格式要求\n请先简要说明你的解析思路和判断依据（2-5句话），然后单独一行输出 `---JSON---`，之后输出纯 JSON 对象（不要用 markdown 代码块包裹）。\n\nJSON 格式：{\"items\": [...], \"new_attrs\": [...]}\n- items: 解析出的物品列表\n- new_attrs: 如果需要新建属性定义，列出新建的属性（key/label/attr_type/config），已有属性不要重复");
    base
}

/// Call LLM with streaming enabled. Sends `Thinking` events via `tx` as tokens arrive.
/// Returns the full accumulated response text.
pub(crate) async fn call_llm_stream(
    system_prompt: &str,
    user_prompt: &str,
    tx: &tokio::sync::mpsc::UnboundedSender<SseEvent>,
) -> Result<String, AppError> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .map_err(|_| anyhow::anyhow!("OPENAI_API_KEY environment variable not set"))?;
    let api_base = std::env::var("OPENAI_BASE_URL")
        .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
    let model = std::env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o".to_string());

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
        response_format: None,
        tools: None,
        stream: Some(true),
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
        let msg = format!("AI API returned {}: {}", status, text);
        let _ = tx.send(SseEvent::Error { message: msg.clone() });
        return Err(AppError::Internal(anyhow::anyhow!("{}", msg)));
    }

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

    Ok(full_text)
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
            tag_scope: String::new(),
            sort_order: 0,
            is_identity: false,
            is_required: false,
            default_value: String::new(),
            search_weight: 0,
        });
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
        // Load categories, tags, attr_defs
        let categories = match sqlx::query_as::<_, Category>(
            "SELECT * FROM categories ORDER BY sort_order",
        )
        .fetch_all(&pool)
        .await
        {
            Ok(c) => c,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("加载分类失败: {:#}", e),
                });
                return;
            }
        };

        let tags = match sqlx::query_as::<_, Tag>("SELECT * FROM tags ORDER BY sort_order")
            .fetch_all(&pool)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("加载标签失败: {:#}", e),
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

        let system_prompt = build_streaming_system_prompt(&categories, &tags, &attr_defs);

        let full_text = match call_llm_stream(&system_prompt, &body.text, &tx).await {
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

        // Resolve categories and tags
        for item in &mut items {
            resolve_parsed_item(item, &categories, &tags);
        }

        // Auto-create tags
        let new_tags = match auto_create_tags_for_items(&mut items, &pool).await {
            Ok(t) => t,
            Err(e) => {
                let _ = tx.send(SseEvent::Error {
                    message: format!("创建标签失败: {:#?}", e),
                });
                drop(tx);
                return;
            }
        };

        // Extract new attribute definitions from AI response
        let new_attr_defs = extract_new_attr_defs_from_text(&full_text);

        // Insert or ignore new attr defs into DB
        let mut saved_attr_defs: Vec<AttributeDefinition> = Vec::new();
        for attr in &new_attr_defs {
            match sqlx::query_as::<_, AttributeDefinition>(
                "INSERT OR IGNORE INTO attribute_definitions (key, label, attr_type, config, category_scope, tag_scope, sort_order, is_identity, is_required, default_value, search_weight) \
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

        let _ = tx.send(SseEvent::Result { items, new_tags, new_attr_defs: saved_attr_defs });
        drop(tx);
    });

    let stream = UnboundedReceiverStream::new(rx).map(|event| {
        let data = serde_json::to_string(&event).unwrap_or_default();
        Ok(Event::default().data(data))
    });

    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_categories() -> Vec<Category> {
        vec![
            Category { id: 1, name: "服装".to_string(), icon: "👕".to_string(), sort_order: 1 },
            Category { id: 2, name: "装备".to_string(), icon: "🎒".to_string(), sort_order: 2 },
            Category { id: 3, name: "营养".to_string(), icon: "🍫".to_string(), sort_order: 3 },
            Category { id: 8, name: "其他".to_string(), icon: "📦".to_string(), sort_order: 8 },
        ]
    }

    fn sample_tags() -> Vec<Tag> {
        vec![
            Tag { id: 1, name: "冲锋衣".to_string(), category_id: 1, sort_order: 1 },
            Tag { id: 2, name: "登山杖".to_string(), category_id: 2, sort_order: 1 },
            Tag { id: 3, name: "头灯".to_string(), category_id: 2, sort_order: 2 },
        ]
    }

    #[test]
    fn resolve_category_exact() {
        let cats = sample_categories();
        assert_eq!(resolve_category_id("服装", &cats), 1);
        assert_eq!(resolve_category_id("装备", &cats), 2);
    }

    #[test]
    fn resolve_category_fuzzy() {
        let cats = sample_categories();
        // "装" is contained in "服装" and "装备"
        // The find() stops at first match, which is "服装"
        assert_eq!(resolve_category_id("装", &cats), 1);
        // "营养品" contains "营养"
        assert_eq!(resolve_category_id("营养品", &cats), 3);
    }

    #[test]
    fn resolve_category_unknown_fallback() {
        let cats = sample_categories();
        // "不存在的分类" doesn't match any → fallback to "其他"
        assert_eq!(resolve_category_id("不存在的分类", &cats), 8);
    }

    #[test]
    fn resolve_tag_exact() {
        let tags = sample_tags();
        assert_eq!(resolve_tag_id("冲锋衣", Some(1), &tags), Some(1));
        assert_eq!(resolve_tag_id("登山杖", Some(2), &tags), Some(2));
    }

    #[test]
    fn resolve_tag_wrong_category() {
        let tags = sample_tags();
        // "冲锋衣" exists but only in category_id=1. If we pass category_id=2, it should fallback
        // to the without-category-constraint match
        let result = resolve_tag_id("冲锋衣", Some(2), &tags);
        // With cat constraint fails, without-cat fallback succeeds
        assert_eq!(result, Some(1));
    }

    #[test]
    fn resolve_tag_unknown() {
        let tags = sample_tags();
        assert_eq!(resolve_tag_id("不存在的标签", Some(1), &tags), None);
    }

    #[test]
    fn resolve_parsed_item_sets_ids() {
        let cats = sample_categories();
        let tags = sample_tags();
        let mut item = AiParsedItem {
            category_name: Some("服装".to_string()),
            tag_name: Some("冲锋衣".to_string()),
            category_id: None,
            tag_id: None,
            attrs: Default::default(),
        };
        resolve_parsed_item(&mut item, &cats, &tags);
        assert_eq!(item.category_id, Some(1));
        assert_eq!(item.tag_id, Some(1));
    }

    #[test]
    fn resolve_parsed_item_no_names() {
        let cats = sample_categories();
        let tags = sample_tags();
        let mut item = AiParsedItem {
            category_name: None,
            tag_name: None,
            category_id: None,
            tag_id: None,
            attrs: Default::default(),
        };
        resolve_parsed_item(&mut item, &cats, &tags);
        assert_eq!(item.category_id, None);
        assert_eq!(item.tag_id, None);
    }

    // ── extract_items_from_text ──

    #[test]
    fn extract_items_with_marker() {
        let text = "Some thinking...\n---JSON---\n{\"items\":[{\"category_name\":\"服装\",\"attrs\":{\"name\":\"冲锋衣\"}}]}";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].attrs.get("name").and_then(|v| v.as_str()), Some("冲锋衣"));
    }

    #[test]
    fn extract_items_without_marker() {
        let text = "{\"items\":[{\"category_name\":\"装备\",\"attrs\":{\"name\":\"登山杖\"}}]}";
        let items = extract_items_from_text(text).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].category_name.as_deref(), Some("装备"));
    }

    #[test]
    fn extract_items_with_markdown_fence() {
        let text = "Thoughts...\n---JSON---\n```json\n{\"items\":[{\"category_name\":\"服装\",\"attrs\":{\"name\":\"软壳\"}}]}\n```";
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