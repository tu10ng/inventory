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

// ── Types ──

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Type {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub sort_order: i64,
    pub parent_id: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeTreeNode {
    pub id: i64,
    pub name: String,
    pub category_id: i64,
    pub sort_order: i64,
    pub parent_id: Option<i64>,
    pub children: Vec<TypeTreeNode>,
}

#[derive(Debug, Deserialize)]
pub struct CreateType {
    pub name: String,
    pub category_id: i64,
    #[serde(default)]
    pub parent_id: Option<i64>,
    #[serde(default)]
    pub sort_order: i64,
}

impl CreateType {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("类型名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateType {
    pub name: Option<String>,
    pub category_id: Option<i64>,
    pub parent_id: Option<Option<i64>>,
    pub sort_order: Option<i64>,
}

fn default_qty() -> i64 {
    1
}

// ── Items ──

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub category_id: i64,
    pub type_id: Option<i64>,
    #[sqlx(default)]
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

fn default_attrs() -> serde_json::Value {
    serde_json::json!({})
}

/// Helper: extract a string field from attrs JSON.
impl Item {
    pub fn attr_str(&self, key: &str) -> String {
        self.attrs
            .get(key)
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string()
    }

    #[allow(dead_code)]
    pub fn attr_i64(&self, key: &str) -> i64 {
        self.attrs
            .get(key)
            .and_then(|v| v.as_i64())
            .unwrap_or(0)
    }
}

#[derive(Debug, Deserialize)]
pub struct CreateItem {
    pub category_id: i64,
    pub type_id: Option<i64>,
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

impl CreateItem {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        let name = self
            .attrs
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if name.is_empty() {
            return Err(crate::error::AppError::validation("物品名称不能为空"));
        }
        if name.len() > 200 {
            return Err(crate::error::AppError::validation("物品名称不能超过200字符"));
        }
        let qty = self
            .attrs
            .get("default_qty")
            .and_then(|v| v.as_i64())
            .unwrap_or(1);
        if qty < 0 {
            return Err(crate::error::AppError::validation("默认数量不能为负数"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateItem {
    pub category_id: Option<i64>,
    pub attrs: Option<serde_json::Value>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub type_id: Option<Option<i64>>,
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
pub struct ActivitySlotWithTypes {
    pub id: i64,
    pub activity_id: i64,
    pub slot_name: String,
    pub category_id: i64,
    pub is_essential: bool,
    pub default_qty: i64,
    pub notes: String,
    pub sort_order: i64,
    pub default_item_id: Option<i64>,
    pub types: Vec<Type>,
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
    pub type_ids: Vec<i64>,
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
    pub type_ids: Option<Vec<i64>>,
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

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct AttributeDefinition {
    pub id: i64,
    pub key: String,
    pub label: String,
    pub attr_type: String,
    pub config: String,
    pub category_scope: String,
    pub type_scope: String,
    pub sort_order: i64,
    #[sqlx(default)]
    #[serde(default)]
    pub is_identity: bool,
    #[sqlx(default)]
    #[serde(default)]
    pub is_required: bool,
    #[sqlx(default)]
    #[serde(default)]
    pub default_value: String,
    #[sqlx(default)]
    #[serde(default)]
    pub search_weight: i64,
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
    pub type_scope: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub is_identity: bool,
    #[serde(default)]
    pub is_required: bool,
    #[serde(default)]
    pub default_value: String,
    #[serde(default)]
    pub search_weight: i64,
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
    pub type_scope: Option<String>,
    pub sort_order: Option<i64>,
    pub is_identity: Option<bool>,
    pub is_required: Option<bool>,
    pub default_value: Option<String>,
    pub search_weight: Option<i64>,
}

// ── Display Rules ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct DisplayRule {
    pub id: i64,
    pub name: String,
    pub category_id: Option<i64>,
    pub group_by_key: String,
    pub sort_by_key: String,
    pub sort_dir: String,
    pub visible_columns: String,
    pub sort_order: i64,
    pub config: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateDisplayRule {
    pub name: String,
    pub category_id: Option<i64>,
    #[serde(default)]
    pub group_by_key: String,
    #[serde(default)]
    pub sort_by_key: String,
    #[serde(default = "default_sort_dir")]
    pub sort_dir: String,
    #[serde(default = "default_empty_array")]
    pub visible_columns: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default = "default_empty_obj")]
    pub config: String,
}

fn default_sort_dir() -> String {
    "asc".to_string()
}

fn default_empty_array() -> String {
    "[]".to_string()
}

fn default_empty_obj() -> String {
    "{}".to_string()
}

impl CreateDisplayRule {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("规则名称不能为空"));
        }
        if self.sort_dir != "asc" && self.sort_dir != "desc" {
            return Err(crate::error::AppError::validation("排序方向只能为 asc 或 desc"));
        }
        // Validate visible_columns is valid JSON
        serde_json::from_str::<serde_json::Value>(&self.visible_columns).map_err(|_| {
            crate::error::AppError::validation("可见列不是有效的 JSON 数组")
        })?;
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateDisplayRule {
    pub name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_some")]
    pub category_id: Option<Option<i64>>,
    pub group_by_key: Option<String>,
    pub sort_by_key: Option<String>,
    pub sort_dir: Option<String>,
    pub visible_columns: Option<String>,
    pub sort_order: Option<i64>,
    pub config: Option<String>,
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
    pub category_name: Option<String>,
    #[serde(default)]
    pub type_name: Option<String>,
    #[serde(default)]
    pub category_id: Option<i64>,
    #[serde(default)]
    pub type_id: Option<i64>,
    #[serde(default = "default_attrs")]
    pub attrs: serde_json::Value,
}

#[derive(Debug, Serialize)]
pub struct AiParseResponse {
    pub items: Vec<AiParsedItem>,
    pub new_types: Vec<Type>,
}

// ── AI Organize ──

#[derive(Debug, Deserialize)]
pub struct OrganizePreviewRequest {
    #[serde(default)]
    pub item_ids: Option<Vec<i64>>,
}

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
    pub category_name: Option<String>,
    pub type_name: Option<String>,
    pub category_id: Option<i64>,
    pub type_id: Option<Option<i64>>,
    pub attrs: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct OrganizePreviewResponse {
    pub actions: Vec<OrganizeAction>,
    pub new_types: Vec<Type>,
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
}

// ── Import / Export ──

#[derive(Debug, Serialize)]
pub struct ExportData {
    pub version: i32,
    pub exported_at: String,
    pub categories: Vec<Category>,
    pub types: Vec<Type>,
    pub attribute_definitions: Vec<AttributeDefinition>,
    pub items: Vec<Item>,
    pub display_rules: Vec<DisplayRule>,
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
    pub types: Vec<Type>,
    pub attribute_definitions: Vec<AttributeDefinition>,
    pub items: Vec<Item>,
    #[serde(default)]
    pub display_rules: Vec<DisplayRule>,
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
    pub action: String,
    pub existing_id: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct ImportResult {
    pub categories_created: u64,
    pub types_created: u64,
    pub attribute_definitions_created: u64,
    pub items_created: u64,
    pub items_updated: u64,
    pub items_skipped: u64,
    pub display_rules_created: u64,
}

// ── Excel Import ──

/// 后端解析 xlsx 后的原始数据（无业务逻辑）
#[derive(Debug, Serialize)]
pub struct ExcelPreviewResponse {
    pub file_name: String,
    pub sheet_names: Vec<String>,
    pub active_sheet: String,
    pub headers: Vec<String>,       // 第一行作为列名，空白列用 "列{N}"
    pub rows: Vec<Vec<String>>,     // 所有数据行，每行长度与 headers 对齐
    pub total_rows: usize,
}

// ── SSE Events (for streaming AI parse) ──

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SseEvent {
    #[serde(rename = "thinking")]
    Thinking { content: String },
    #[serde(rename = "progress")]
    Progress { message: String },
    #[serde(rename = "result")]
    Result {
        items: Vec<AiParsedItem>,
        new_types: Vec<Type>,
        #[serde(default)]
        new_attr_defs: Vec<AttributeDefinition>,
    },
    #[serde(rename = "error")]
    Error { message: String },
}

// ── Relation Types ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RelationType {
    pub id: i64,
    pub name: String,
    pub label: String,
    pub color: String,
    pub icon: String,
    pub bidirectional: bool,
    pub sort_order: i64,
}

#[derive(Debug, Deserialize)]
pub struct CreateRelationType {
    pub name: String,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub icon: String,
    #[serde(default)]
    pub bidirectional: bool,
    #[serde(default)]
    pub sort_order: i64,
}

impl CreateRelationType {
    pub fn validate(&self) -> Result<(), crate::error::AppError> {
        if self.name.trim().is_empty() {
            return Err(crate::error::AppError::validation("关系类型名称不能为空"));
        }
        Ok(())
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateRelationType {
    pub name: Option<String>,
    pub label: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
    pub bidirectional: Option<bool>,
    pub sort_order: Option<i64>,
}

// ── Item Relations ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ItemRelation {
    pub id: i64,
    pub source_item_id: i64,
    pub target_item_id: i64,
    pub relation_type_id: i64,
    pub notes: String,
}

/// Item relation with related item info for display
#[derive(Debug, Serialize)]
pub struct ItemRelationEnriched {
    pub id: i64,
    pub source_item_id: i64,
    pub target_item_id: i64,
    pub relation_type_id: i64,
    pub notes: String,
    pub target_name: String,
    pub relation_label: String,
    pub relation_color: String,
    pub relation_icon: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateItemRelation {
    pub target_item_id: i64,
    pub relation_type_id: i64,
    #[serde(default)]
    pub notes: String,
}

// ── Activity Includes ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityInclude {
    pub id: i64,
    pub activity_id: i64,
    pub included_activity_id: i64,
    pub sort_order: i64,
}

/// Activity include with the included activity's info
#[derive(Debug, Serialize)]
pub struct ActivityIncludeEnriched {
    pub id: i64,
    pub activity_id: i64,
    pub included_activity_id: i64,
    pub sort_order: i64,
    pub included_name: String,
    pub included_icon: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityInclude {
    pub included_activity_id: i64,
    #[serde(default)]
    pub sort_order: i64,
}

// ── Batch Items ──

#[derive(Debug, Deserialize)]
pub struct BatchItemsRequest {
    pub ids: Vec<i64>,
    pub action: String,
    #[serde(default)]
    pub changes: Option<serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct BatchItemsResponse {
    pub updated: u64,
    pub deleted: u64,
}

// ── Tests ──

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    // ── CreateItem::validate() ──

    #[test]
    fn create_item_empty_name() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({}),
        };
        assert!(item.validate().is_err());
    }

    #[test]
    fn create_item_whitespace_name() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "   "}),
        };
        assert!(item.validate().is_err());
    }

    #[test]
    fn create_item_name_too_long() {
        let long_name = "x".repeat(201);
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": long_name}),
        };
        assert!(item.validate().is_err());
    }

    #[test]
    fn create_item_zero_qty() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "测试物品", "default_qty": 0}),
        };
        assert!(item.validate().is_ok());
    }

    #[test]
    fn create_item_negative_qty() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "测试物品", "default_qty": -1}),
        };
        assert!(item.validate().is_err());
    }

    #[test]
    fn create_item_valid_minimal() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "测试物品"}),
        };
        assert!(item.validate().is_ok());
    }

    #[test]
    fn create_item_valid_full() {
        let item = CreateItem {
            category_id: 2,
            type_id: Some(1),
            attrs: json!({"name": "冲锋衣", "brand": "始祖鸟", "model": "Beta LT", "default_qty": 2, "warmth_rating": 30}),
        };
        assert!(item.validate().is_ok());
    }

    #[test]
    fn create_item_name_exactly_200_chars() {
        let name = "x".repeat(200);
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": name}),
        };
        assert!(item.validate().is_ok());
    }

    #[test]
    fn create_item_default_qty_not_present() {
        let item = CreateItem {
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "测试"}),
        };
        // default_qty defaults to 1, which is valid
        assert!(item.validate().is_ok());
    }

    // ── Item attr helpers ──

    #[test]
    fn item_attr_str_present() {
        let item = Item {
            id: 1,
            category_id: 1,
            type_id: None,
            attrs: json!({"name": "冲锋衣", "brand": "始祖鸟"}),
        };
        assert_eq!(item.attr_str("name"), "冲锋衣");
        assert_eq!(item.attr_str("brand"), "始祖鸟");
    }

    #[test]
    fn item_attr_str_missing() {
        let item = Item {
            id: 1,
            category_id: 1,
            type_id: None,
            attrs: json!({}),
        };
        assert_eq!(item.attr_str("name"), "");
        assert_eq!(item.attr_str("nonexistent"), "");
    }

    #[test]
    fn item_attr_i64_present() {
        let item = Item {
            id: 1,
            category_id: 1,
            type_id: None,
            attrs: json!({"default_qty": 3, "warmth_rating": 25}),
        };
        assert_eq!(item.attr_i64("default_qty"), 3);
        assert_eq!(item.attr_i64("warmth_rating"), 25);
    }

    #[test]
    fn item_attr_i64_missing() {
        let item = Item {
            id: 1,
            category_id: 1,
            type_id: None,
            attrs: json!({}),
        };
        assert_eq!(item.attr_i64("default_qty"), 0);
    }
}
