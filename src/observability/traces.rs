use super::get_ressources;
use crate::config::{Env, Tracing};
use core::time::Duration;
use opentelemetry::trace::TracerProvider as TraceProviderOtel;
use opentelemetry_otlp::{ExportConfig, Protocol, WithExportConfig, WithTonicConfig};
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler, Tracer};
use opentelemetry_sdk::trace::{SpanLimits, TracerProvider};
use tracing::Subscriber;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_subscriber::{EnvFilter, Layer};

/// Init the traces configuration for the lifetime of the applications.
/// User code should only emit spans and events with the Tracing API
pub fn init_traces(otel: &Tracing, env: &Env) {
    tracing_log::LogTracer::init().expect("Failed to set log tracer");
    let subscriber = init_subscriber(otel, env);
    tracing::subscriber::set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Retreive the fully configured subscriber
fn init_subscriber(otel: &Tracing, env: &Env) -> impl Subscriber + Sync + Send {
    // Config which trace levels to collect
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Configure multiples targets to send traces
    let file_layer = if otel.file_enabled {
        let file_appender = tracing_appender::rolling::hourly("", "rolling.log");
        let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);
        let common_layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking);
        let layer = match env {
            Env::Dev => common_layer.pretty().boxed(),
            _ => common_layer.json().boxed(),
        };
        Some(layer)
    } else {
        None
    };

    let stdout_layer = if otel.stdout_enabled {
        let common_layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(std::io::stdout);

        let layer = match env {
            Env::Dev => common_layer.pretty().boxed(),
            _ => common_layer.json().boxed(),
        };
        Some(layer)
    } else {
        None
    };

    let otel_layer = if otel.otel_enabled {
        let tracer = match env {
            Env::Dev => get_stdout_tracer(otel),
            _ => get_otlp_tracer(otel),
        };
        let layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Some(layer)
    } else {
        None
    };

    Registry::default()
        .with(filter_layer)
        .with(file_layer)
        .with(stdout_layer)
        .with(otel_layer)
}

/// Init the stdout opentelemetry tracer
fn get_stdout_tracer(otel: &Tracing) -> Tracer {
    let ressources = get_ressources(otel);

    let provider = TracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .with_resource(ressources)
        .build();

    provider.tracer("axum-app")
}

/// Init the OTLP opentelemetry tracer
fn get_otlp_tracer(otel: &Tracing) -> Tracer {
    // For the moment, user code only interact with the Tracing API so the propagation
    // is done throught this API and not via opentelemetry
    //opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());

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

    let provider = TracerProvider::builder()
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
        .build();

    provider.tracer("axum-app")
}
