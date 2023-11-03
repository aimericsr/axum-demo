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
use crate::web::rest::routes_static::routes as routes_static;
use crate::web::routes_docs::routes as routes_docs;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware;
use axum::middleware::{from_fn, from_fn_with_state, map_response};
use axum::routing::get;
use axum::BoxError;
use axum::Router;
use axum::{error_handling::HandleErrorLayer, http::StatusCode};
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use metrics_exporter_prometheus::PrometheusHandle;
use opentelemetry::global;
use opentelemetry::metrics::Counter;
use opentelemetry::KeyValue;
use std::future::ready;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
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

/// Type to hold the newly built server and his port
pub struct Application {
    port: u16,
    server: Pin<Box<dyn Future<Output = hyper::Result<()>> + Send>>,
}

impl Application {
    /// build the axum server with the provided configuration without lunch it
    #[instrument(skip_all)]
    pub async fn build(config: Config, prom: PrometheusHandle) -> Result<Self> {
        let mm = setup_db_migrations().await;

        let routes_all = routes(mm, prom);

        let addr = SocketAddr::from(([0, 0, 0, 0], config.application.port));

        let server = axum::Server::bind(&addr)
            .serve(routes_all.into_make_service_with_connect_info::<SocketAddr>());
        let port = server.local_addr().port();
        let server = server.with_graceful_shutdown(shutdown_signal());
        info!("Listening on {port:?}");
        Ok(Self {
            port,
            server: Box::pin(server),
        })
    }

    /// Lunch the already build server to start listening to requests
    pub async fn run_until_stopped(self) -> ResultIO<(), hyper::Error> {
        self.server.await
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

#[derive(Clone)]
pub struct SharedState {
    pub root_dir: String,
    pub foobar: Counter<u64>,
    pub mm: ModelManager,
}

fn routes(mm: ModelManager, prom: PrometheusHandle) -> Router {
    let governor_conf = Box::new(
        GovernorConfigBuilder::default()
            .per_second(1)
            .burst_size(10)
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

    //let routes_prom: Router = Router::new().route("/metrics", get(move || ready(prom.render())));
    let metrics = HttpMetricsLayerBuilder::new()
        .with_service_name("axum-demo".to_string())
        .with_service_version("0.0.1".to_string())
        .build();

    let state = SharedState {
        root_dir: String::from("/tmp"),
        foobar: global::meter("axum-app").u64_counter("foobar").init(),
        mm,
    };

    let routes_all = Router::new()
        .merge(routes_health().with_state(state.clone()))
        .merge(metrics.routes::<SharedState>().with_state(state.clone()))
        //.merge(routes_prom)
        .merge(routes_hello())
        .merge(routes_login().with_state(state.clone()))
        //.nest("/api", routes_rpc)
        .layer(map_response(mw_res_map))
        .merge(routes_static())
        .layer(from_fn_with_state(
            state.mm.clone(),
            web::mw_auth::mw_ctx_resolve,
        ))
        .layer(CookieManagerLayer::new())
        //.layer(middleware::from_fn(track_metrics))
        .merge(routes_docs())
        .layer(cors_layer)
        .layer(rate_limit_layer)
        .layer(TimeoutLayer::new(Duration::from_secs(5)))
        .layer(metrics)
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //create a span with the http context using the OpenTelemetry naming convention on incoming request
        .layer(OtelAxumLayer::default());
    routes_all
}

/// Confirm to the otlp backend that the programm has been shutdown sucessfuly
#[instrument(skip_all)]
pub fn graceful_shutdown() {
    info!("signal received, starting graceful shutdown");
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

    graceful_shutdown();

    opentelemetry::global::shutdown_tracer_provider();
}
