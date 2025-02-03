use std::time::Duration;

use axum::{extract::State, routing::get, Json, Router};
use hyper::{header, HeaderMap};
use tower_otel::traces::get_current_otel_trace_id;
use tracing::instrument;

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
    tag = "Health",
    responses(
        (status = 200, description = "General health check"),
    )
)]
async fn health() -> HeaderMap {
    std::thread::sleep(Duration::from_millis(100));
    let mut headers = HeaderMap::new();
    headers.insert(header::CACHE_CONTROL, "no-cache".parse().unwrap());
    headers
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/ready",
    tag = "Health",
    responses(
        (status = 200, description = "Ready health check"),
    )
)]
#[instrument(
    skip(state),
    level = "info",
    name = "compute_task",
    fields(next = 1),
    //ret(Debug),
    err(Debug)
)]
async fn health_ready(State(state): State<SharedState>) -> Result<Json<Vec<String>>, ()> {
    state.metric.app_domain_health_user_count.add(1, &[]);
    let trace_id = get_current_otel_trace_id().unwrap_or("unknown".to_string());
    dbg!(trace_id);
    //Err(())
    Ok(Json(vec!["ready".to_owned(), "true".to_owned()]))
}

#[utoipa::path(
    get,
    context_path = "/health",
    path = "/live",
    tag = "Health",
    responses(
        (status = 200, description = "Live health check"),
        (status = 408, description = "Timeout"),
    )
)]
async fn health_live() -> Json<Vec<String>> {
    Json(vec!["alive".to_owned(), "true".to_owned()])
}
