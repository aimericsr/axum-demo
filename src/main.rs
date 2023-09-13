// Modules

mod config;
mod ctx;
mod error;
mod log;
mod model;
mod web;

// Re-export
pub use self::error::{Error, Result};
pub use config::Config;

//  Import
use crate::model::ModelManager;
use crate::web::mw_res_map::mw_res_map;
use axum::middleware;
use axum::Router;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
use web::routes_hello::routes as routes_hello;
use web::routes_login::routes as routes_login;
use web::routes_static::serve_dir as routes_static;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let mc = ModelManager::new().await?;

    let routes_all = Router::new()
        .merge(routes_hello())
        .merge(routes_login())
        .layer(middleware::map_response(mw_res_map))
        // above CookieManagerLayer because we need it
        .layer(middleware::from_fn_with_state(
            mc.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    info!("LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}
