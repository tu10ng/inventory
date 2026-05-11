mod db;
mod error;
mod handlers;
mod models;

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    // Check tesseract availability (non-fatal: OCR endpoint will also check and return error)
    match std::process::Command::new("which")
        .arg("tesseract")
        .output()
    {
        Ok(output) if output.status.success() => {
            tracing::info!("Tesseract OCR found");
        }
        _ => {
            tracing::warn!("Tesseract OCR not installed — /api/ai/ocr will be unavailable. Install: pacman -S tesseract tesseract-data-chi_sim tesseract-data-eng");
        }
    }

    let pool = db::init_pool().await;

    let app = handlers::router()
        .with_state(pool)
        .layer(CorsLayer::permissive())
        .fallback_service(ServeDir::new("../frontend/build").append_index_html_on_directories(true));

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
