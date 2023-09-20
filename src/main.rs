// region:    --- Modules
mod config;
mod crypt;
mod ctx;
mod error;
mod log;
mod model;
mod utils;
mod web;

pub mod _dev_utils;
pub use self::error::{Error, Result};
pub use config::config;

use utoipa::OpenApi;
use utoipa_rapidoc::RapiDoc;
use utoipa_redoc::{Redoc, Servable};
use utoipa_swagger_ui::SwaggerUi;

use crate::model::ModelManager;
use crate::web::mw_auth::mw_ctx_require;
use crate::web::mw_res_map::mw_res_map;
use crate::web::rpc;
use axum::middleware;
use axum::Router;
use std::net::SocketAddr;
use tower_cookies::CookieManagerLayer;
use tracing::info;
use tracing_subscriber::EnvFilter;
use web::rest::routes_hello::routes as routes_hello;
use web::rest::routes_login::routes as routes_login;
use web::rest::routes_static::serve_dir as routes_static;
use web::routes_docs::routes as routes_docs;
// endregion: --- Modules

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    // -- FOR DEV ONLY
    _dev_utils::init_dev().await;

    let mm = ModelManager::new().await?;

    let routes_rpc = rpc::routes(mm.clone()).route_layer(middleware::from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_docs())
        .merge(routes_login(mm.clone()))
        .merge(routes_hello())
        .nest("/api", routes_rpc)
        .layer(middleware::map_response(mw_res_map))
        // above CookieManagerLayer because we need it
        .layer(middleware::from_fn_with_state(
            mm.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        .fallback_service(routes_static());

    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}
