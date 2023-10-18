use crate::config::config;
use axum::handler::HandlerWithoutStateExt;
use axum::http::StatusCode;
use axum::Router;
use tower_http::services::ServeDir;

pub fn routes() -> Router {
    Router::new().nest_service(
        "/assets",
        ServeDir::new(&config().application.web_folder)
            .not_found_service(handle_404.into_service()),
    )
}

async fn handle_404() -> (StatusCode, &'static str) {
    (StatusCode::NOT_FOUND, "Resource not found.")
}
