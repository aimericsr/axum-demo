use std::future::ready;

use axum::{routing::get, Router};

use crate::observability::metrics::create_prometheus_recorder;

// use crate::observability::metrics::{create_prometheus_recorder, track_metrics};

pub fn routes() -> Router {
    // let (prometheus_layer, metric_handle) = PrometheusMetricLayer::pair();

    // Router::new().route(
    //     "/metrics",
    //     get(|| async move { metric_handle.render() }).layer(prometheus_layer),
    // )

    let prometheus_recorder = create_prometheus_recorder();

    Router::new().route("/metrics", get(move || ready(prometheus_recorder.render())))
}
