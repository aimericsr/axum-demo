use opentelemetry::{metrics::Meter, KeyValue};
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig};
use opentelemetry_sdk::Resource;
use std::time::Duration;

pub fn init_metrics() -> Meter {
    //let exporter = opentelemetry_stdout::MetricExporter::default();

    let exporter = opentelemetry_otlp::MetricExporter::builder()
        .with_tonic()
        .with_export_config(ExportConfig {
            endpoint: Some("http://localhost:4317".into()),
            timeout: Duration::from_secs(3),
            protocol: Protocol::Grpc,
        })
        .with_temporality(opentelemetry_sdk::metrics::Temporality::Cumulative)
        .build()
        .unwrap();

    let reader = opentelemetry_sdk::metrics::PeriodicReader::builder(
        exporter,
        opentelemetry_sdk::runtime::Tokio,
    )
    .with_interval(std::time::Duration::from_secs(3))
    .with_timeout(Duration::from_secs(10))
    .build();

    let provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
        .with_reader(reader)
        .with_resource(Resource::new(vec![KeyValue::new(
            "service.name",
            "axum-demo",
        )]))
        .build();

    opentelemetry::global::set_meter_provider(provider.clone());

    opentelemetry::global::meter("axum_demo")
}

pub async fn init_tokio_metrics(meter: Meter) {
    let tokio_metrics = tokio::runtime::Handle::current().metrics();

    // Stable tokio metrics
    let tokio_metrics_num_workers = tokio_metrics.clone();
    meter
        .f64_observable_gauge("tokio.runtime.worker_threads.count")
        .with_unit("threads")
        .with_description("The number of worker threads in the Tokio runtime that are actively driving the readiness of futures.")
        .with_callback(move |observer| {
            observer.observe(
                tokio_metrics_num_workers.clone().num_workers() as f64,
                &[
                    KeyValue::new("runtime", "tokio"),
                    KeyValue::new("environment", "development"),
                ],
            )
        })
        .build();

    let tokio_metrics_num_alive_tasks = tokio_metrics.clone();
    meter
        .f64_observable_gauge("tokio.runtime.tasks.active.count")
        .with_unit("tasks")
        .with_description("The number of tasks currently alive and managed by the Tokio runtime, including all tasks that are running or scheduled.")
        .with_callback(move |observer| {
            observer.observe(tokio_metrics_num_alive_tasks.num_alive_tasks() as f64, &[
                KeyValue::new("runtime", "tokio"),
                KeyValue::new("environment", "development"),
            ],)
        })
        .build();

    let tokio_metrics_global_queue_depth = tokio_metrics.clone();
    meter
        .f64_observable_gauge("tokio.runtime.queue.global.depth")
        .with_unit("tasks")
        .with_description("The number of tasks waiting in the global Tokio queue, indicating potential contention for runtime worker threads.")
        .with_callback(move |observer| {
            observer.observe(
                tokio_metrics_global_queue_depth.global_queue_depth() as f64,
                &[
                    KeyValue::new("runtime", "tokio"),
                    KeyValue::new("environment", "development"),
                ],
            )
        })
        .build();

    // Unstable tokio metrics
}
