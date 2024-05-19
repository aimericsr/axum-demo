use crate::config::Config;
pub use crate::error::{Error, Result};
use crate::model::ModelManager;
use crate::web;
use crate::web::mw_res_map::mw_res_map;
use crate::web::rest::routes_health::routes as routes_health;
use crate::web::rest::routes_hello::routes as routes_hello;
use crate::web::rest::routes_login::routes as routes_login;
use crate::web::rest::routes_static::routes as routes_static;
use crate::web::routes_docs::routes as routes_docs;
use crate::web::Error as ErrorWeb;
use axum::error_handling::HandleErrorLayer;
use axum::extract::connect_info::IntoMakeServiceWithConnectInfo;
use axum::extract::ConnectInfo;
use axum::http::HeaderValue;
use axum::http::Method;
use axum::middleware::AddExtension;
use axum::middleware::{from_fn_with_state, map_response};
use axum::serve::Serve;
use axum::BoxError;
use axum::Router;
use axum_otel_metrics::HttpMetricsLayerBuilder;
use axum_tracing_opentelemetry::middleware::OtelAxumLayer;
use axum_tracing_opentelemetry::middleware::OtelInResponseLayer;
use opentelemetry::global;
use opentelemetry::metrics::Counter;
use std::net::SocketAddr;
use std::result::Result as ResultIO;
use std::time::Duration;
use tokio::signal;
use tower::ServiceBuilder;
use tower_cookies::CookieManagerLayer;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::CorsLayer;
use tracing::info;
use tracing::instrument;
/// Type to hold the newly built server and his port
pub struct Application
// <T>
// where
//     T: Future<Output = ()>,
{
    port: u16,
    server: Serve<
        IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
        AddExtension<Router, ConnectInfo<SocketAddr>>,
        // server: WithGracefulShutdown<
        //     IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr>,
        //     AddExtension<Router, ConnectInfo<std::net::SocketAddr>>,
        //     impl std::future::Future<Output = ()>,
    >,
    // server: WithGracefulShutdown<
    //     IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr>,
    //     AddExtension<Router, ConnectInfo<std::net::SocketAddr>>,
    //     //T,
    //     Pending<()>,
    //     //Pin<Box<dyn Future<Output = ()>>>,
    //     //impl Future<Output = ()>,
    // >,
}

//server: Pin<Box<dyn Future<Output = hyper::Result<()>> + Send>>,
// server: Serve<
//     IntoMakeServiceWithConnectInfo<Router, SocketAddr>,
//     AddExtension<Router, ConnectInfo<SocketAddr>>,
// >,
// server: WithGracefulShutdown<
//     IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr>,
//     AddExtension<Router, ConnectInfo<std::net::SocketAddr>>,
//     Box<dyn std::future::Future<Output = ()> + Send>,
// >,

impl Application {
    /// build the axum server with the provided configuration without lunch it
    #[instrument(skip_all)]
    pub async fn build(config: Config) -> Result<Self> {
        let mm = setup_db_migrations().await;

        let routes = routes(mm);

        let addr = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", config.application.port))
            .await
            .unwrap();

        let server = axum::serve(
            addr,
            routes.into_make_service_with_connect_info::<SocketAddr>(),
        );
        //.with_graceful_shutdown(shutdown_signal());

        let port = config.application.port;

        //let server = server.with_graceful_shutdown(shutdown_signal());
        info!("Listening on {port:?}");

        Ok(Self {
            port,
            //server: Box::pin(server),
            server,
        })
    }

    /// Lunch the already build server to start listening to requests
    pub async fn run_until_stopped(self) -> ResultIO<(), std::io::Error> {
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

#[derive(Clone, Debug)]
pub struct SharedState {
    pub custom_prometheus_metrics: CustomPrometheusMetrics,
    pub mm: ModelManager,
}

#[derive(Clone, Debug)]
pub struct CustomPrometheusMetrics {
    pub ready_endpoint: Counter<u64>,
}

fn routes(mm: ModelManager) -> Router {
    // Build services for Rate Limit and Timeout
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
            ErrorWeb::RateLimitExceeded
        }))
        .layer(GovernorLayer {
            config: Box::leak(governor_conf),
        });

    let timeout_layer = ServiceBuilder::new()
        .layer(HandleErrorLayer::new(|_: BoxError| async {
            ErrorWeb::Timeout
        }))
        .timeout(Duration::from_secs(1));

    // Set CORS
    let cors_layer = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET]);

    let metrics = HttpMetricsLayerBuilder::new()
        .with_service_name("axum-demo".to_string())
        .with_service_version("0.0.1".to_string())
        .build();

    let prom = CustomPrometheusMetrics {
        ready_endpoint: global::meter("axum-app").u64_counter("foobar").init(),
    };
    let state = SharedState {
        custom_prometheus_metrics: prom,
        mm,
    };

    // Build the main Router
    Router::new()
        .merge(routes_health().with_state(state.clone()))
        .merge(metrics.routes::<SharedState>().with_state(state.clone()))
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
        .layer(metrics)
        // include trace context as header into the response
        .layer(OtelInResponseLayer::default())
        //create a span with the http context using the OpenTelemetry naming convention on incoming request
        .layer(OtelAxumLayer::default())
}

/// Confirm to the otlp backend that the programm has been shutdown sucessfuly
#[instrument]
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

    info!("signal received, graceful shutdown finished successfully");

    opentelemetry::global::shutdown_tracer_provider();
}

// I want to store an instance of an Axum app in a struct to be able to do tests after. I can when I am note using the graceful shutdown, the type is Serve<IntoMakeServiceWithConnectInfo<Router, SocketAddr>, AddExtension<Router, ConnectInfo<SocketAddr>>> which can be used but when I add the graceful shutdown the type is WithGracefulShutdown<IntoMakeServiceWithConnectInfo<Router, SocketAddr>, AddExtension<Router, ConnectInfo<SocketAddr>>, impl Future<Output = ()>> . But as my error says, it's not possible to use impl in fields types. Here is my code.

// ```
//  let server = axum::serve(
//             addr,
//             routes.into_make_service_with_connect_info::<SocketAddr>(),
//         )
//         .with_graceful_shutdown(shutdown_signal());

// async fn shutdown_signal() {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("failed to install Ctrl+C handler");
//     };

//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("failed to install signal handler")
//             .recv()
//             .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }

//     info!("signal received, graceful shutdown finished successfully");

//     opentelemetry::global::shutdown_tracer_provider();
// }

// pub struct Application {
//     port: u16,
//     server: WithGracefulShutdown<
//         IntoMakeServiceWithConnectInfo<Router, std::net::SocketAddr>,
//         AddExtension<Router, ConnectInfo<std::net::SocketAddr>>,
//         impl std::future::Future<Output = ()>,
//     >,
// }
// ```
