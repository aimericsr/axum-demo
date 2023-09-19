// region:    --- Modules
mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod observability;
mod utils;
mod web;

pub mod _dev_utils;
pub use self::error::{Error, Result};
use crate::model::ModelManager;
use crate::observability::tracing::create_tracer_from_env;
use crate::web::mw_auth::mw_ctx_require;
use crate::web::mw_res_map::mw_res_map;
use axum::response::Html;
use axum::response::Response;
use axum::routing::get;
use axum::Json;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use axum_tracing_opentelemetry::opentelemetry_tracing_layer;
pub use config::config;
use serde_json::json;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use tracing_subscriber::Registry;

use crate::web::routes_prometheus::routes as routes_prometheus;
use crate::web::rpc;
use axum::middleware;
use axum::Router;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use web::routes_hello::routes as routes_hello;
use web::routes_login::routes as routes_login;
use web::routes_static::serve_dir as routes_static;
// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing

    // let subscriber = FmtSubscriber::builder()
    //     // all spans/events with a level higher than TRACE (e.g, debug, info, warn, etc.)
    //     // will be written to stdout.
    //     .with_max_level(Level::INFO)
    //     // completes the builder.
    //     .finish();

    // tracing::subscriber::set_global_default(subscriber).expect("setting default subscriber failed");

    // tracing_subscriber::fmt()
    //     .with_env_filter(EnvFilter::from_default_env())
    //     .init();

    let registry = Registry::default().with(tracing_subscriber::fmt::layer().pretty());

    match create_tracer_from_env() {
        Some(tracer) => registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer))
            .try_init()
            .expect("Failed to register tracer with registry and jaeger"),
        None => registry
            .try_init()
            .expect("Failed to register tracer with registry and no jaeger"),
    }

    //init_tracing_opentelemetry::tracing_subscriber_ext::init_subscribers()?;

    // -- FOR DEV ONLY
    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;

    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(routes_login(mm.clone()))
        .merge(routes_health())
        .merge(routes_prometheus())
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_res_map))
        // above CookieManagerLayer because we need it
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}

fn routes_health() -> Router {
    Router::new()
        .route("/health/ready", get(health_ready))
        .route("/health/live", get(health_live))
        .route("/health", get(health))
}

async fn health_ready() -> Json<Vec<String>> {
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
