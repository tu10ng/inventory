use serde::{Deserialize, Deserializer, Serialize};

/// Deserialize a present-but-null JSON field as `Some(None)`,
/// and an absent field as `None` (via `#[serde(default)]`).
fn deserialize_some<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer).map(Some)
}

// ── Categories ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub icon: String,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateCategory {
    pub name: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub sort_order: i64,
}

impl CreateCategory {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("分类名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCategory {
    pub name: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i64>,
}

// ── Tags ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tag {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTag {
    pub name: String,
    pub category_id: i64,
    #[serde(default)]
    pub sort_order: i64,
}

impl CreateTag {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("标签名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTag {
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub sort_order: Option<i64>,
}

// ── Items ──

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub model: String,
    pub category_id: i64,
    pub default_qty: i64,
    pub notes: String,
    pub tag_id: Option<i64>,
    #[sqlx(default)]
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

fn default_attrs() -> serde_json::Value {
    serde_json::json!({})
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub name: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub model: String,
    pub category_id: i64,
    #[serde(default = "default_qty")]
    pub default_qty: i64,
    #[serde(default)]
    pub notes: String,
    pub tag_id: Option<i64>,
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

fn default_qty() -> i64 {
    1
}

impl CreateItem {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("物品名称不能为空"));
        }
        if self.name.len() > 200 {
            return Err(crate::error::AppError::validation("物品名称不能超过200字符"));
        }
        if self.default_qty < 1 {
            return Err(crate::error::AppError::validation("默认数量必须大于0"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateItem {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub category_id: Option<i64>,
    pub default_qty: Option<i64>,
    pub notes: Option<String>,
    pub attrs: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub tag_id: Option<Option<i64>>,
}

// ── Activities ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Activity {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub icon: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivity {
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub icon: String,
}

impl CreateActivity {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("活动名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateActivity {
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
}

fn default_true() -> bool {
    true
}

// ── Activity Slots ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivitySlot {
    pub id: i64,
    pub activity_id: i64,
    pub slot_name: String,
    pub category_id: i64,
    pub is_essential: bool,
    pub default_qty: i64,
    pub notes: String,
    pub sort_order: i64,
    pub default_item_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ActivitySlotWithTags {
    pub id: i64,
    pub activity_id: i64,
    pub slot_name: String,
    pub category_id: i64,
    pub is_essential: bool,
    pub default_qty: i64,
    pub notes: String,
    pub sort_order: i64,
    pub default_item_id: Option<i64>,
    pub tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivitySlot {
    pub slot_name: String,
    pub category_id: i64,
    #[serde(default = "default_true")]
    pub is_essential: bool,
    #[serde(default = "default_qty")]
    pub default_qty: i64,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub tag_ids: Vec<i64>,
    pub default_item_id: Option<i64>,
}

impl CreateActivitySlot {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.slot_name.trim().is_empty() {
            return Err(crate::error::AppError::validation("槽位名称不能为空"));
        }
        if self.default_qty < 1 {
            return Err(crate::error::AppError::validation("默认数量必须大于0"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateActivitySlot {
    pub slot_name: Option<String>,
    pub category_id: Option<i64>,
    pub is_essential: Option<bool>,
    pub default_qty: Option<i64>,
    pub notes: Option<String>,
    pub sort_order: Option<i64>,
    pub tag_ids: Option<Vec<i64>>,
    pub default_item_id: Option<Option<i64>>,
}

// ── Tips ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Tip {
    pub id: i64,
    pub activity_id: i64,
    pub content: String,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateTip {
    pub content: String,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateTip {
    pub content: Option<String>,
    pub sort_order: Option<i64>,
}

// ── People ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Person {
    pub id: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct CreatePerson {
    pub name: String,
}

impl CreatePerson {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("人员名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdatePerson {
    pub name: Option<String>,
}

// ── Trips ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Trip {
    pub id: i64,
    pub name: String,
    pub activity_id: Option<i64>,
    pub start_date: String,
    pub end_date: String,
    pub notes: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateTrip {
    pub name: String,
    pub activity_id: Option<i64>,
    #[serde(default)]
    pub start_date: String,
    #[serde(default)]
    pub end_date: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_status")]
    pub status: String,
}

fn default_status() -> String {
    "planning".to_string()
}

impl CreateTrip {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("行程名称不能为空"));
        }
        if self.name.len() > 200 {
            return Err(crate::error::AppError::validation("行程名称不能超过200字符"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTrip {
    pub name: Option<String>,
    pub activity_id: Option<i64>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
}

// ── Trip Items ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TripItem {
    pub id: i64,
    pub trip_id: i64,
    pub item_id: Option<i64>,
    pub custom_name: String,
    pub person_id: Option<i64>,
    pub qty: i64,
    pub checked: bool,
    pub item_status: String,
    pub notes: String,
    pub sort_order: i64,
    pub is_essential: bool,
    pub slot_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTripItem {
    pub item_id: Option<i64>,
    #[serde(default)]
    pub custom_name: String,
    pub person_id: Option<i64>,
    #[serde(default = "default_qty")]
    pub qty: i64,
    #[serde(default)]
    pub checked: bool,
    #[serde(default)]
    pub item_status: String,
    #[serde(default)]
    pub notes: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub is_essential: bool,
    pub slot_id: Option<i64>,
}

impl CreateTripItem {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.item_id.is_none() && self.custom_name.trim().is_empty() {
            return Err(crate::error::AppError::validation("必须选择物品或填写自定义名称"));
        }
        if self.qty < 1 {
            return Err(crate::error::AppError::validation("数量必须大于0"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateTripItem {
    #[serde(default, deserialize_with = "deserialize_some")]
    pub item_id: Option<Option<i64>>,
    pub custom_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub person_id: Option<Option<i64>>,
    pub qty: Option<i64>,
    pub checked: Option<bool>,
    pub item_status: Option<String>,
    pub notes: Option<String>,
    pub sort_order: Option<i64>,
    pub is_essential: Option<bool>,
}

#[derive(Debug, Deserialize)]
pub struct CheckBody {
    pub checked: bool,
}

// ── Enriched Trip Items ──

#[derive(Debug, Serialize)]
pub struct SlotInfo {
    pub id: i64,
    pub slot_name: String,
    pub category_id: i64,
    pub is_essential: bool,
}

#[derive(Debug, Serialize)]
pub struct TripItemEnriched {
    pub id: i64,
    pub trip_id: i64,
    pub item_id: Option<i64>,
    pub custom_name: String,
    pub person_id: Option<i64>,
    pub qty: i64,
    pub checked: bool,
    pub item_status: String,
    pub notes: String,
    pub sort_order: i64,
    pub is_essential: bool,
    pub slot_id: Option<i64>,
    pub slot: Option<SlotInfo>,
    pub candidates: Vec<Item>,
}

// ── Attribute Definitions ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttributeDefinition {
    pub id: i64,
    pub key: String,
    pub label: String,
    pub attr_type: String,
    pub config: String,
    pub category_scope: String,
    pub tag_scope: String,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateAttributeDefinition {
    pub key: String,
    pub label: String,
    #[serde(default = "default_attr_type")]
    pub attr_type: String,
    #[serde(default)]
    pub config: String,
    #[serde(default)]
    pub category_scope: String,
    #[serde(default)]
    pub tag_scope: String,
    #[serde(default)]
    pub sort_order: i64,
}

fn default_attr_type() -> String {
    "number".to_string()
}

#[derive(Debug, Deserialize)]
pub struct UpdateAttributeDefinition {
    pub key: Option<String>,
    pub label: Option<String>,
    pub attr_type: Option<String>,
    pub config: Option<String>,
    pub category_scope: Option<String>,
    pub tag_scope: Option<String>,
    pub sort_order: Option<i64>,
}

// ── Status Definitions ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct StatusDefinition {
    pub id: i64,
    pub scope: String,
    pub value: String,
    pub label: String,
    pub color: String,
    pub icon: String,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateStatusDefinition {
    pub scope: String,
    pub value: String,
    pub label: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct UpdateStatusDefinition {
    pub scope: Option<String>,
    pub value: Option<String>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub sort_order: Option<i64>,
}

// ── Usage Stats ──

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct ItemUsageCount {
    pub item_id: i64,
    pub trip_count: i64,
}

#[derive(Debug, Serialize)]
pub struct ItemUsageStats {
    pub item_id: i64,
    pub trips: Vec<TripRef>,
}

#[derive(Debug, Serialize, sqlx::FromRow)]
pub struct TripRef {
    pub id: i64,
    pub name: String,
    pub status: String,
}

// ── Resync Preview ──

#[derive(Debug, Serialize)]
pub struct ResyncPreview {
    pub items_to_remove: Vec<ResyncPreviewItem>,
    pub items_to_add: Vec<ResyncPreviewItem>,
}

#[derive(Debug, Serialize)]
pub struct ResyncPreviewItem {
    pub trip_item_id: Option<i64>,
    pub slot_name: Option<String>,
    pub item_name: Option<String>,
    pub custom_name: Option<String>,
    pub reason: String,
}

// ── Bulk Update ──

#[derive(Debug, Deserialize)]
pub struct BulkUpdateTripItems {
    pub ids: Vec<i64>,
    pub checked: Option<bool>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub person_id: Option<Option<i64>>,
    pub item_status: Option<String>,
}

// ── AI Parse ──

#[derive(Debug, Deserialize)]
pub struct AiParseRequest {
    pub text: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiParsedItem {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub brand: String,
    #[serde(default)]
    pub model: String,
    #[serde(default)]
    pub category_name: Option<String>,
    #[serde(default)]
    pub tag_name: Option<String>,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub tag_id: Option<i64>,
    #[serde(default)]
    pub notes: String,
    #[serde(default = "default_qty")]
    pub default_qty: i64,
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AiParseResponse {
    pub items: Vec<AiParsedItem>,
    pub new_tags: Vec<Tag>,
}

// ── AI Organize ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action_type")]
pub enum OrganizeAction {
    #[serde(rename = "update")]
    Update {
        item_id: i64,
        reason: String,
        fields: OrganizeUpdateFields,
    },
    #[serde(rename = "split")]
    Split {
        item_id: i64,
        reason: String,
        new_items: Vec<AiParsedItem>,
    },
    #[serde(rename = "delete")]
    Delete {
        item_id: i64,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrganizeUpdateFields {
    pub name: Option<String>,
    pub brand: Option<String>,
    pub model: Option<String>,
    pub category_name: Option<String>,
    pub tag_name: Option<String>,
    pub category_id: Option<i64>,
    pub tag_id: Option<Option<i64>>,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct OrganizePreviewResponse {
    pub actions: Vec<OrganizeAction>,
    pub new_tags: Vec<Tag>,
}

#[derive(Debug, Deserialize)]
pub struct OrganizeApplyRequest {
    pub actions: Vec<OrganizeAction>,
}

#[derive(Debug, Serialize)]
pub struct OrganizeApplyResponse {
    pub updated: i64,
    pub created: i64,
    pub deleted: i64,
    pub new_tags: Vec<Tag>,
}

// ── Import / Export ──

#[derive(Debug, Serialize)]
pub struct ExportData {
    pub version: i32,
    pub exported_at: String,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub attribute_definitions: Vec<AttributeDefinition>,
    pub items: Vec<Item>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ImportStrategy {
    Skip,
    Update,
}

#[derive(Debug, Deserialize)]
pub struct ImportRequest {
    pub version: i32,
    pub categories: Vec<Category>,
    pub tags: Vec<Tag>,
    pub attribute_definitions: Vec<AttributeDefinition>,
    pub items: Vec<Item>,
    pub strategy: ImportStrategy,
}

#[derive(Debug, Serialize)]
pub struct ImportPreviewResult {
    pub total_items: usize,
    pub new_items: usize,
    pub skip_or_update_items: usize,
    pub preview_items: Vec<ImportItemPreview>,
}

#[derive(Debug, Serialize)]
pub struct ImportItemPreview {
    pub name: String,
    pub brand: String,
    pub model: String,
    pub action: String,
    pub existing_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub categories_created: u64,
    pub tags_created: u64,
    pub attribute_definitions_created: u64,
    pub items_created: u64,
    pub items_updated: u64,
    pub items_skipped: u64,
}
