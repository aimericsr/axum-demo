use super::get_ressources;
use crate::config::Tracing;
use opentelemetry::{metrics::Meter, KeyValue};
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use std::time::Duration;

pub fn init_metrics(otel: &Tracing) -> Meter {
    //let exporter = opentelemetry_stdout::MetricExporter::default();

    let exporter = opentelemetry_otlp::MetricExporter::builder()
        // .with_tonic()
        .with_tonic()
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Cumulative)
        // .with_export_config(ExportConfig {
        //     endpoint: Some("http://localhost:4317".into()),
        //     timeout: Duration::from_secs(3),
        //     protocol: Protocol::Grpc,
        // })
        .build()
        .unwrap();

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(exporter)
        .with_interval(std::time::Duration::from_secs(3))
        .build();

    let ressources = get_ressources(otel);

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader.clone())
        .with_resource(ressources)
        .build();

    opentelemetry::global::set_meter_provider(provider);

    let meter = opentelemetry::global::meter("axum_demo");

    init_tokio_metrics(&meter);

    meter
}

fn init_tokio_metrics(meter: &Meter) {
    let tokio_metrics = tokio::runtime::Handle::current().metrics();

    let attributes = [
        KeyValue::new("runtime", "tokio"),
        KeyValue::new("environment", "development"),
    ];

    // Stable tokio metrics
    let tokio_metrics_num_workers = tokio_metrics.clone();
    let attributes_num_workers = attributes.clone();
    meter
        .f64_observable_up_down_counter("tokio.runtime.worker_threads.count")
        .with_unit("threads")
        .with_description("The number of worker threads in the Tokio runtime that are actively driving the readiness of futures.")
        .with_callback(move |observer| {
            observer.observe(
                tokio_metrics_num_workers.clone().num_workers() as f64,
                &attributes_num_workers,
            )
        })
        .build();

    let tokio_metrics_num_alive_tasks = tokio_metrics.clone();
    let attributes_num_alive_tasks = attributes.clone();
    meter
        .f64_observable_up_down_counter("tokio.runtime.tasks.active.count")
        .with_unit("tasks")
        .with_description("The number of tasks currently alive and managed by the Tokio runtime, including all tasks that are running or scheduled.")
        .with_callback(move |observer| {
            observer.observe(tokio_metrics_num_alive_tasks.num_alive_tasks() as f64, &attributes_num_alive_tasks)
        })
        .build();

    let tokio_metrics_global_queue_depth = tokio_metrics.clone();
    let attributes_global_queue_depth = attributes.clone();
    meter
        .f64_observable_up_down_counter("tokio.runtime.queue.global.depth")
        .with_unit("tasks")
        .with_description("The number of tasks waiting in the global Tokio queue, indicating potential contention for runtime worker threads.")
        .with_callback(move |observer| {
            observer.observe(
                tokio_metrics_global_queue_depth.global_queue_depth() as f64,
                &attributes_global_queue_depth,
            )
        })
        .build();

    // Unstable tokio metrics
    let tokio_metrics_num_workers = tokio_metrics.clone();
    meter
        .f64_observable_up_down_counter("tokio.runtime.worker_threads.blocking.count")
        .with_unit("threads")
        .with_description("The number of blocking worker threads in the Tokio runtime, used to execute tasks that perform blocking operations. 
            This pool is dynamically managed by Tokio based on demand."
        )
        .with_callback(move |observer| {
            observer.observe(
                tokio_metrics_num_workers.clone().num_blocking_threads() as f64,
                &attributes,
            )
        })
        .build();
}
