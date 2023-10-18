// region:    --- Modules

use crate::config::Config;
pub use crate::error::{Error, Result};
use crate::model::ModelManager;
use crate::observability::metrics::track_metrics;
use crate::web;
use crate::web::mw_auth::mw_ctx_require;
use crate::web::mw_res_map::mw_res_map;
use crate::web::rest::routes_health::routes as routes_health;
use crate::web::rest::routes_hello::routes as routes_hello;
use crate::web::rest::routes_login::routes as routes_login;
use crate::web::rest::routes_prometheus::routes as routes_prometheus;
use crate::web::rest::routes_static::routes as routes_static;
use crate::web::routes_docs::routes as routes_docs;
use crate::web::rpc;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware;
use axum::middleware::{from_fn, from_fn_with_state, map_response};
use axum::BoxError;
use axum::Router;
use axum::{error_handling::HandleErrorLayer, http::StatusCode};
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use hyper::server::conn::AddrIncoming;
use hyper::Server;
use std::net::SocketAddr;
use std::result::Result as ResultIO;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tower_http::timeout::TimeoutLayer;
use tracing::info;
use tracing::instrument;

/// Type to hold the newly built server and its port
pub struct Application {
    port: u16,
    server: Server<AddrIncoming, IntoMakeServiceWithConnectInfo<Router, SocketAddr>>,
}

impl Application {
    /// build the axum server with the provided configuration without lunch it
    pub async fn build(config: &Config) -> Result<Self> {
        let mm = setup_db_migrations().await;

        let routes_all = routes(mm);

        let addr = SocketAddr::from(([0, 0, 0, 0], config.application.port));
        info!("LISTENING on {addr}");

        let server = axum::Server::bind(&addr)
            .serve(routes_all.into_make_service_with_connect_info::<SocketAddr>());

        Ok(Self {
            port: config.application.port,
            server,
        })
    }

    /// Lunch the already build server to start listening to requests<br><br>
    /// We append the function with_graceful_shutdown to the axum Server here because the type that it return is
    /// private to hyper crate so we can't put it in the Application struct
    pub async fn run_until_stopped(self) -> ResultIO<(), hyper::Error> {
        self.server.with_graceful_shutdown(shutdown_signal()).await
    }

    /// Returns the port on which the application will be listening to
    pub fn port(&self) -> u16 {
        self.port
    }
}

#[instrument()]
async fn setup_db_migrations() -> ModelManager {
    info!("Create connection to db");
    let mm = ModelManager::new()
        .await
        .expect("Failed to create modelManager");
    info!("Creating migrations");
    mm.clone()
        .migrate()
        .await
        .expect("Failed to migrate database");
    info!("Created migrations");
    mm
}

fn routes(mm: ModelManager) -> Router {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let rate_limit_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async move {
            StatusCode::TOO_MANY_REQUESTS
        }))
        .layer(GovernorLayer {
            config: Box::leak(governor_conf),
        });

    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET]);

    let routes_rpc = rpc::routes(mm.clone()).route_layer(from_fn(mw_ctx_require));

    let routes_all = Router::new()
        .merge(routes_docs())
        .merge(routes_health())
        .merge(routes_static())
        .merge(routes_hello())
        .merge(routes_login(mm.clone()))
        .nest("/api", routes_rpc)
        .merge(routes_prometheus())
        .layer(map_response(mw_res_map))
        .layer(from_fn_with_state(mm.clone(), web::mw_auth::mw_ctx_resolve))
        .layer(CookieManagerLayer::new())
        .layer(middleware::from_fn(track_metrics))
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //start OpenTelemetry trace on incoming request
        .layer(OtelAxumLayer::default())
        .layer(cors_layer)
        .layer(rate_limit_layer)
        .layer(TimeoutLayer::new(Duration::from_secs(5)));
    routes_all
}

/// Graceful shutdown to be able to send the last logs to the otlp backend before stopping the application
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
