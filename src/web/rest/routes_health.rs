use axum::{extract::State, routing::get, Json, Router};
use hyper::{header, HeaderMap};
use opentelemetry::KeyValue;

use crate::startup::SharedState;

pub fn routes() -> Router<SharedState> {
    Router::new().nest("/health", sub_routes())
}

fn sub_routes() -> Router<SharedState> {
    Router::new()
        .route("/", get(health))
        .route("/ready", get(health_ready))
        .route("/live", get(health_live))
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "",
    tag = "health",
    responses(
        (status = 200, description = "General health check"),
    )
)]
async fn health() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    headers
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
async fn health_ready(State(state): State<SharedState>) -> Json<Vec<String>> {
    state.foobar.add(1, &[KeyValue::new("test", "value")]);
    state.foobar.add(5, &[KeyValue::new("test2", "value2")]);
    Json(vec!["ready".to_owned(), "true".to_owned()])
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/live",
    tag = "health",
    responses(
        (status = 200, description = "Live health check"),
        (status = 408, description = "Timeout"),
    )
)]
async fn health_live() -> Json<Vec<String>> {
    Json(vec!["alive".to_owned(), "true".to_owned()])
}
