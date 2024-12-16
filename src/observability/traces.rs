use super::get_ressources;
use crate::config::Otel;
use core::time::Duration;
use opentelemetry::trace::TracerProvider as TraceProviderOtel;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler};
use opentelemetry_sdk::trace::{SpanLimits, TracerProvider};
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Init the traces configuration for the lifetime of the applications.
/// User code should only emit spans and events with the Tracing API
pub fn init_traces(otel: &Otel) {
    tracing_log::LogTracer::init().expect("Failed to set log tracer");
    let subscriber = get_subscriber(otel);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Retreive the fully configured subscriber
fn get_subscriber(otel: &Otel) -> impl Subscriber + Sync + Send {
    // Config which trace levels to collect
    let env_filter = EnvFilter::builder().try_from_env().unwrap();

    // Configure multiples targets to send traces to
    let stdout_json_layer = if otel.stdout_enabled {
        Some(tracing_subscriber::fmt::layer().json())
    } else {
        None
    };

    let otel_stdout_layer = if otel.stdout_enabled {
        let provider = TracerProvider::builder()
            .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
            .build();
        let tracer = provider.tracer("axum-app");
        let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Some(opentelemetry_layer)
    } else {
        None
    };

    let otel_layer = if otel.otel_enabled {
        let provider = get_tracer_provider(otel);
        let tracer = provider.tracer("axum-app2");
        let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Some(opentelemetry_layer)
    } else {
        None
    };

    Registry::default()
        .with(env_filter)
        .with(stdout_json_layer)
        .with(otel_stdout_layer)
        .with(otel_layer)
}

/// Init the opentelemetry tracer
fn get_tracer_provider(otel: &Otel) -> TracerProvider {
    // For the moment, user code only interact with the Tracing API so the propagation
    // is done throught this API and not via opentelemetry
    //global::set_text_map_propagator(TraceContextPropagator::new());

    let ressources = get_ressources(otel);

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_export_config(ExportConfig {
            endpoint: Some("http://localhost:4317".into()),
            protocol: Protocol::Grpc,
            timeout: Duration::from_secs(3),
        })
        .with_compression(opentelemetry_otlp::Compression::Zstd)
        .build()
        .unwrap();

    TracerProvider::builder()
        .with_resource(ressources)
        .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
        .with_sampler(Sampler::AlwaysOn)
        .with_id_generator(RandomIdGenerator::default())
        .with_span_limits(SpanLimits {
            max_events_per_span: 64,
            max_attributes_per_span: 64,
            max_links_per_span: 64,
            max_attributes_per_event: 64,
            max_attributes_per_link: 64,
        })
        .build()
}
