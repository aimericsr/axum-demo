use axum::{extract::MatchedPath, http::Request, middleware::Next, response::IntoResponse};
use metrics::{gauge, histogram, increment_counter};
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use opentelemetry::metrics::ObservableGauge;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::util::tokio_interval_stream;
use std::time::{Duration, Instant};
use sysinfo::{CpuExt, System, SystemExt};

use crate::config::config;

const REQUEST_DURATION_METRIC_NAME: &str = "http_requests_duration_seconds";
const MEMORY_USAGE_METRIC_NAME: &str = "memory_usage";
const MEMORY_TOTAL_METRIC_NAME: &str = "memory_total";
const MEMORY_FREE_METRIC_NAME: &str = "memory_free";
const MEMORY_SWAP_METRIC_NAME: &str = "memory_swap";
const CPU_USAGE_METRIC_NAME: &str = "cpu_usage";

pub(crate) fn create_prometheus_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full(REQUEST_DURATION_METRIC_NAME.to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap_or_else(|_| {
            panic!(
                "Could not initialize the bucket for '{}'",
                REQUEST_DURATION_METRIC_NAME
            )
        })
        .install_recorder()
        .expect("Could not install the Prometheus recorder")
}

pub(crate) async fn track_metrics<B>(req: Request<B>, next: Next<B>) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let path_clone = path.clone();
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    // App metrics
    increment_counter!("http_requests_total", &labels);
    histogram!(REQUEST_DURATION_METRIC_NAME, latency, &labels);

    if path_clone.eq("/metrics") {
        let mut sys = System::new();
        sys.refresh_memory();
        sys.refresh_cpu();

        // System metrics
        gauge!(MEMORY_TOTAL_METRIC_NAME, sys.total_memory() as f64);
        gauge!(MEMORY_USAGE_METRIC_NAME, sys.used_memory() as f64);
        gauge!(MEMORY_FREE_METRIC_NAME, sys.free_memory() as f64);
        gauge!(MEMORY_SWAP_METRIC_NAME, sys.used_swap() as f64);
        gauge!(
            CPU_USAGE_METRIC_NAME,
            sys.global_cpu_info().cpu_usage() as f64
        );
    }

    response
}

// fn init_otlp_metrics() {
//     let meter = opentelemetry_otlp::new_pipeline()
//         .metrics()
//         .with_exporter(
//             opentelemetry_otlp::new_exporter()
//                 .tonic()
//                 .with_endpoint(&config().otel.endpoint)
//                 .with_timeout(Duration::from_secs(3)),
//         )
//         .with_stateful(true)
//         .with_period(Duration::from_secs(3))
//         .with_timeout(Duration::from_secs(10))
//         .with_aggregator_selector(selectors::simple::Selector::Exact)
//         .build();
// }
