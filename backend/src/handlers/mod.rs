pub mod activities;
pub mod categories;
pub mod items;
pub mod people;
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
        // Items
        .route("/api/items", get(items::list).post(items::create))
        .route("/api/items/{id}", get(items::get).put(items::update).delete(items::delete))
        // People
        .route("/api/people", get(people::list).post(people::create))
        .route("/api/people/{id}", put(people::update).delete(people::delete))
        // Activities
        .route("/api/activities", get(activities::list).post(activities::create))
        .route("/api/activities/{id}", get(activities::get).put(activities::update).delete(activities::delete))
        .route("/api/activities/{id}/items", get(activities::list_items).post(activities::add_item))
        .route("/api/activities/{activity_id}/items/{item_id}", delete(activities::remove_item))
        .route("/api/activities/{id}/tips", get(activities::list_tips).post(activities::create_tip))
        .route("/api/tips/{id}", put(activities::update_tip).delete(activities::delete_tip))
        // Trips
        .route("/api/trips", get(trips::list).post(trips::create))
        .route("/api/trips/{id}", get(trips::get).put(trips::update).delete(trips::delete))
        .route("/api/trips/{id}/populate", post(trips::populate))
        .route("/api/trips/{id}/items", get(trip_items::list).post(trip_items::create))
        // Trip items
        .route("/api/trip-items/{id}", put(trip_items::update).delete(trip_items::delete))
        .route("/api/trip-items/{id}/check", patch(trip_items::check))
}
