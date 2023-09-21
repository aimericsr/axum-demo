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
use axum::http::HeaderValue;
use axum::http::Method;
pub use config::config;
use tokio::signal;
use tower_http::timeout::TimeoutLayer;
use tracing::Level;

use crate::model::ModelManager;
use crate::observability::tracing::get_subscriber;
use crate::web::mw_auth::mw_ctx_require;
use crate::web::mw_res_map::mw_res_map;
use crate::web::rest::routes_health::routes as routes_health;
use crate::web::rest::routes_hello::routes as routes_hello;
use crate::web::rest::routes_login::routes as routes_login;
use crate::web::rest::routes_prometheus::routes as routes_prometheus;
use crate::web::rest::routes_static::serve_dir as routes_static;
use crate::web::routes_docs::routes as routes_docs;
use crate::web::rpc;

use axum::middleware;
use axum::Router;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use std::net::SocketAddr;
use std::time::Duration;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use tracing::info;

// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    // Tracing
    get_subscriber(Level::INFO.to_string());

    // -- FOR DEV ONLY
    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;

    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_docs())
        .merge(routes_login(mm.clone()))
        .merge(routes_hello())
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
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(
            CorsLayer::new()
                .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
                .allow_methods([Method::GET]),
        )
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([0, 0, 0, 0], config().application.port));
    info!("LISTENING on {addr}\n");

    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("signal received, starting graceful shutdown");
    opentelemetry::global::shutdown_tracer_provider();
}
