use std::future::ready;

use axum::{middleware, routing::get, Router};

use crate::observability::metrics::{create_prometheus_recorder, track_metrics};

pub fn routes() -> Router {
    let prometheus_recorder = create_prometheus_recorder();

    Router::new().route(
        "/metrics",
        get(move || ready(prometheus_recorder.render()))
            .route_layer(middleware::from_fn(track_metrics)),
    )
}
