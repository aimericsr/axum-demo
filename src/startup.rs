use crate::config::Config;
pub use crate::error::{Error, Result};
use crate::model::ModelManager;
use crate::observability::ObservabilityGuard;
use crate::web;
use crate::web::Error as ErrorWeb;
use crate::web::mw_res_map::mw_res_map;
use crate::web::rest::routes_health::routes as routes_health;
use crate::web::rest::routes_hello::routes as routes_hello;
use crate::web::rest::routes_login::routes as routes_login;
use crate::web::rest::routes_static::routes as routes_static;
use crate::web::routes_docs::routes as routes_docs;
use axum::BoxError;
use axum::Router;
use axum::error_handling::HandleErrorLayer;
use axum::extract::ConnectInfo;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware::AddExtension;
use axum::middleware::{from_fn_with_state, map_response};
use axum::serve::Serve;
use http::request::Parts;
use opentelemetry::metrics::Counter;
use opentelemetry::metrics::Meter;
use std::net::SocketAddr;
use std::result::Result as ResultIO;
use std::sync::Arc;
use std::time::Duration;
use tokio::net::TcpListener;
use tokio::signal;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_governor::{GovernorLayer, governor::GovernorConfigBuilder};
use tower_http::cors::CorsLayer;
use tower_otel::axum::metrics::OtelMetricsLayer;
use tower_otel::axum::traces::OtelLoggerLayer;
use tracing::info;
use tracing::instrument;

/// Type to hold the newly built server and his listening port
pub struct Application {
    port: u16,
    server: Serve<
        TcpListener,
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
    >,
}

impl Application {
    /// build the axum server with the provided configuration without lunch it
    #[instrument(skip_all)]
    pub async fn build(config: Config, meter: Meter) -> Result<Self> {
        let mm = setup_db_migrations().await;

        let routes = routes(mm, meter);

        let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.application.port))
            .await
            .unwrap();

        let port = addr.local_addr().unwrap().port();

        let server = axum::serve(
            addr,
            routes.into_make_service_with_connect_info::<SocketAddr>(),
        );

        info!("Listening on {port:?}");

        Ok(Self { port, server })
    }

    /// Lunch the already build server with graceful shutdown and start listening to requests
    pub async fn run_until_stopped(
        self,
        observability_guard: ObservabilityGuard,
    ) -> ResultIO<(), std::io::Error> {
        self.server
            .with_graceful_shutdown(shutdown_signal(observability_guard))
            .await
    }

    /// Returns the port on which the application will be listening to
    pub fn port(&self) -> u16 {
        self.port
    }
}

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

#[derive(Clone, Debug)]
pub struct SharedState {
    pub metric: OtelMetric,
    pub mm: ModelManager,
}

#[derive(Clone, Debug)]
pub struct OtelMetric {
    pub app_domain_health_user_count: Counter<u64>,
}

fn routes(mm: ModelManager, meter: Meter) -> Router {
    let governor_conf = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(5)
            .burst_size(20)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let rate_limit_layer = ServiceBuilder::new().layer(GovernorLayer {
        config: governor_conf,
    });

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            ErrorWeb::Timeout
        }))
        .timeout(Duration::from_secs(1));

    let concurrency_limit_layer = ServiceBuilder::new().concurrency_limit(10_000);

    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET]);

    let app_domain_health_user_count = meter
        .u64_counter("app.domain.health.user.count")
        .with_unit("users")
        .with_description("The number of users requesting the health info of this service.")
        .build();
    let otel_metric = OtelMetric {
        app_domain_health_user_count,
    };

    let state = SharedState {
        metric: otel_metric,
        mm,
    };

    let logger = OtelLoggerLayer::default()
        .with_filter(Arc::new(|_req: &Parts| true))
        .with_span_attributes(Arc::new(|_req: &Parts| vec![("MY_APP", "axum")]));

    // Build the main Router
    Router::new()
        .merge(routes_health().with_state(state.clone()))
        .merge(routes_hello())
        .merge(routes_login().with_state(state.clone()))
        .merge(routes_static())
        .layer(from_fn_with_state(
            state.mm.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        .fallback(|| async { ErrorWeb::FallBack })
        .layer(cors_layer)
        .layer(rate_limit_layer)
        .merge(routes_docs())
        .layer(timeout_layer)
        .layer(map_response(mw_res_map))
        .layer(logger)
        .layer(OtelMetricsLayer::new(meter))
        .layer(concurrency_limit_layer)
}

/// Graceful shutdown to be able to send the last traces/metrics to the otlp backend before stopping the application.
///
/// SIGINT and SIGTERM are listen, only linux-based system are supported as signal management greatly dependens on the OS.
async fn shutdown_signal(observability_guard: ObservabilityGuard) {
    #[cfg(unix)]
    let ctrl_c = async {
        signal::unix::signal(signal::unix::SignalKind::interrupt())
            .expect("failed to install signal handler for SIGINT")
            .recv()
            .await;
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler for SIGTERM")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {
            info!("signal SIGINT received");
        },
        _ = terminate => {
            info!("signal SIGTERM received");
        },
    }
    info!("Graceful shutdown started successfully with a timeout of 5 seconds");

    tokio::select! {
        _  = observability_guard.shutdown() => {
            info!("Graceful shutdown has been completed successfully");
        },
        _ = tokio::time::sleep(Duration::from_secs(5)) => {
            info!("Timeout of 5 seconds has been reached without the shutdown to complete, some traces/metrics may have been lost, exiting the appliction");
        },
    }
}
