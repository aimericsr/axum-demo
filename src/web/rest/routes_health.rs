use axum::{routing::get, Json, Router};

pub fn routes() -> Router {
    Router::new().nest("/health", sub_routes())
}

fn sub_routes() -> Router {
    Router::new()
        .route("/", get(health))
        .route("/ready", get(health_ready))
        .route("/live", get(health_live))
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/",
    tag = "health",
    responses(
        (status = 200, description = "General health check"),
    )
)]
async fn health() -> Json<Vec<String>> {
    Json(vec!["general".to_owned(), "true".to_owned()])
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/ready",
    tag = "health",
    responses(
        (status = 200, description = "Ready health check"),
    )
)]
async fn health_ready() -> Json<Vec<String>> {
    Json(vec!["ready".to_owned(), "true".to_owned()])
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/live",
    tag = "health",
    responses(
        (status = 200, description = "Live health check"),
    )
)]
async fn health_live() -> Json<Vec<String>> {
    Json(vec!["alive".to_owned(), "true".to_owned()])
}
