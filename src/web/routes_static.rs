use axum::{Router, routing::get_service};
use tower_http::services::ServeDir;

// use the tower ecosystem for the filesystem API
pub fn routes() -> Router {
    Router::new().nest_service("/", get_service(ServeDir::new("./static")))
}
