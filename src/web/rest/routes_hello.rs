use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use serde::Deserialize;
use tracing::debug;

pub fn routes() -> Router {
    Router::new()
        .route("/hello", get(handler_hello))
        .route("/hello2/:name", get(handler_hello_2))
}

#[utoipa::path(
    get,
    path = "/hello",
    responses(
        (status = 200, description = "Greetings with the name provided or default to World", example = "Hello <strong>World</strong>"),
    )
)]
async fn handler_hello(Query(params): Query<HelloParams>) -> impl IntoResponse {
    let name = params.name.as_deref().unwrap_or("World");
    Html(format!("Hello <strong>{name}</strong>"))
}

#[derive(Debug, Deserialize)]
pub struct HelloParams {
    name: Option<String>,
}

#[utoipa::path(
    get,
    path = "/hello2/{name}",
    params(
        ("name" = String, Path, description = "Name to greet")
    ),
    responses(
        (status = 200, description = "Greetings with the name provided ", example = "Hello <strong>World</strong>")
    )
)]
async fn handler_hello_2(Path(name): Path<String>) -> impl IntoResponse {
    debug!("{:<12} - handler_hello2 - {name:?}", "HANDLER");

    Html(format!("Hello2 <strong>{name}</strong>"))
}
