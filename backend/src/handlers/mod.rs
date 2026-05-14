pub mod activities;
pub mod ai;
pub mod attributes;
pub mod categories;
pub mod display_rules;
pub mod excel;
pub mod items;
pub mod ocr;
pub mod people;
pub mod relations;
pub mod statuses;
pub mod tags;
pub mod trip_items;
pub mod trips;

use axum::routing::{delete, get, patch, post, put};
use axum::Router;
use sqlx::SqlitePool;

pub fn router() -> Router<SqlitePool> {
    Router::new()
        // Categories
        .route("/api/categories", get(categories::list).post(categories::create))
        .route("/api/categories/{id}", put(categories::update).delete(categories::delete))
        // Tags
        .route("/api/tags", get(tags::list).post(tags::create))
        .route("/api/tags/{id}", put(tags::update).delete(tags::delete))
        // Excel import
        .route("/api/import/excel-preview", post(excel::excel_preview))
        // Items — literal paths BEFORE {id}
        .route("/api/items/export", get(items::export_items))
        .route("/api/items/import-preview", post(items::import_preview))
        .route("/api/items/import", post(items::import_items))
        .route("/api/items/batch", post(items::batch))
        .route("/api/items", get(items::list).post(items::create))
        .route("/api/items/{id}", get(items::get).put(items::update).delete(items::delete))
        .route("/api/item-stats", get(items::usage_stats))
        .route("/api/item-stats/{id}", get(items::usage_detail))
        .route("/api/ai/ocr", post(ocr::ocr_images))
        .route("/api/ai/parse-items", post(ai::parse_items))
        .route("/api/ai/parse-items-stream", post(ai::parse_items_stream))
        .route("/api/ai/organize-preview", post(ai::organize_preview))
        .route("/api/ai/organize-apply", post(ai::organize_apply))
        // Status definitions
        .route("/api/status-definitions", get(statuses::list).post(statuses::create))
        .route("/api/status-definitions/{id}", put(statuses::update).delete(statuses::delete))
        // Attribute definitions
        .route("/api/attribute-definitions", get(attributes::list).post(attributes::create))
        .route("/api/attribute-definitions/{id}", put(attributes::update).delete(attributes::delete))
        // People
        .route("/api/people", get(people::list).post(people::create))
        .route("/api/people/{id}", put(people::update).delete(people::delete))
        // Activities
        .route("/api/activities", get(activities::list).post(activities::create))
        .route("/api/activities/{id}", get(activities::get).put(activities::update).delete(activities::delete))
        .route("/api/activities/{id}/slots", get(activities::list_slots).post(activities::create_slot))
        .route("/api/activities/{id}/tips", get(activities::list_tips).post(activities::create_tip))
        .route("/api/activity-slots/{id}", put(activities::update_slot).delete(activities::delete_slot))
        .route("/api/tips/{id}", put(activities::update_tip).delete(activities::delete_tip))
        // Display rules
        .route("/api/display-rules", get(display_rules::list).post(display_rules::create))
        .route("/api/display-rules/{id}", put(display_rules::update).delete(display_rules::delete))
        // Relation types (new meta-layer)
        .route("/api/relation-types", get(relations::list_relation_types).post(relations::create_relation_type))
        .route("/api/relation-types/{id}", put(relations::update_relation_type).delete(relations::delete_relation_type))
        // Item relations
        .route("/api/items/{id}/relations", get(relations::list_item_relations).post(relations::create_item_relation))
        .route("/api/item-relations/{id}", delete(relations::delete_item_relation))
        // Activity includes
        .route("/api/activities/{id}/includes", get(relations::list_activity_includes).post(relations::create_activity_include))
        .route("/api/activity-includes/{id}", delete(relations::delete_activity_include))
        // Trips
        .route("/api/trips", get(trips::list).post(trips::create))
        .route("/api/trips/{id}", get(trips::get).put(trips::update).delete(trips::delete))
        .route("/api/trips/{id}/populate", post(trips::populate))
        .route("/api/trips/{id}/resync", post(trips::resync))
        .route("/api/trips/{id}/resync-preview", post(trips::resync_preview))
        .route("/api/trips/{id}/clone", post(trips::clone))
        // Trip items: enriched MUST be registered before the generic /items route
        .route("/api/trips/{id}/items/enriched", get(trip_items::list_enriched))
        .route("/api/trips/{id}/items/bulk", patch(trip_items::bulk_update))
        .route("/api/trips/{id}/items", get(trip_items::list).post(trip_items::create))
        // Trip items (standalone)
        .route("/api/trip-items/{id}", put(trip_items::update).delete(trip_items::delete))
        .route("/api/trip-items/{id}/check", patch(trip_items::check))
        .route("/api/trip-items/{id}/save-as-slot", post(trip_items::save_as_slot))
}
