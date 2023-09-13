use crate::model::ModelManager;
use crate::web::mw_res_map::mw_res_map;

pub use self::error::{Error, Result};
use std::net::SocketAddr;

use axum::middleware;
use axum::Router;
use tower_cookies::CookieManagerLayer;
use web::routes_static::routes as routes_static ;
use web::routes_hello::routes as routes_hello;
use web::routes_login::routes as routes_login;

mod ctx;
mod error;
mod log;
mod model;
mod web;

#[tokio::main]
async fn main() -> Result<()> {
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
    println!("->> LISTENING on {addr}\n");
    axum::Server::bind(&addr)
        .serve(routes_all.into_make_service())
        .await
        .unwrap();
    Ok(())
}

