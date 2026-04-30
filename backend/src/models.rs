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

// ── Items ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct Item {
    pub id: i64,
    pub name: String,
    pub brand: String,
    pub model: String,
    pub category_id: i64,
    pub default_qty: i64,
    pub notes: String,
    pub tag_id: Option<i64>,
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
}

fn default_qty() -> i64 {
    1
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

// ── Activity Items (legacy, kept for backward compat) ──

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct ActivityItem {
    pub id: i64,
    pub activity_id: i64,
    pub item_id: i64,
    pub is_essential: bool,
    pub default_qty: i64,
    pub notes: String,
}

#[derive(Debug, Deserialize)]
pub struct CreateActivityItem {
    pub item_id: i64,
    #[serde(default = "default_true")]
    pub is_essential: bool,
    #[serde(default = "default_qty")]
    pub default_qty: i64,
    #[serde(default)]
    pub notes: String,
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
