use axum::{routing::get, Json, Router};
use tokio::time::{sleep, Duration};
use tracing::info;

pub fn routes() -> Router {
    Router::new()
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
        .route("/health", get(health))
}

async fn health_ready() -> Json<Vec<String>> {
    sleep(Duration::from_secs(8)).await;
    info!("Health ready");
    Json(vec!["ready".to_owned(), "true".to_owned()])
}

async fn health_live() -> Json<Vec<String>> {
    info!("Health live");
    Json(vec!["alive".to_owned(), "true".to_owned()])
}

async fn health() -> Json<Vec<String>> {
    info!("Health");
    Json(vec!["general".to_owned(), "true".to_owned()])
}
