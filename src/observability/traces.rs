use super::get_ressources;
use crate::config::{Config, Env, Tracing};
use opentelemetry::trace::TracerProvider as TraceProviderOtel;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::{RandomIdGenerator, Sampler};
use opentelemetry_sdk::trace::{SdkTracerProvider, SpanLimits};
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::{EnvFilter, Layer};
use tracing_subscriber::{Registry, layer::SubscriberExt};

/// Init the traces configuration for the lifetime of the applications.
/// User code should only emit spans and events with the Tracing API
/// Log records are forwared as tracing events for compatibility
pub fn init_traces(conf: &Config) -> (Option<WorkerGuard>, Option<SdkTracerProvider>) {
    let (file_guard, otel_guard) = init_subscriber(&conf.tracing, &conf.env);
    (file_guard, otel_guard)
}

/// Retreive the fully configured subscriber
fn init_subscriber(otel: &Tracing, env: &Env) -> (Option<WorkerGuard>, Option<SdkTracerProvider>) {
    // Config which trace levels to collect
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    // Configure multiples targets to send traces
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

    let file_layer = if otel.file_enabled {
        let file_appender = tracing_appender::rolling::hourly("logs", "rolling.log");
        let (non_blocking, file_guard) = tracing_appender::non_blocking(file_appender);

        let common_layer = tracing_subscriber::fmt::layer()
            .with_span_events(FmtSpan::CLOSE)
            .with_writer(non_blocking);
        let layer = match env {
            Env::Dev => common_layer.pretty().boxed(),
            _ => common_layer.json().boxed(),
        };
        Some((layer, file_guard))
    } else {
        None
    };

    let otel_layer = if otel.otel_enabled {
        opentelemetry::global::set_text_map_propagator(TraceContextPropagator::new());
        let tracer = match env {
            Env::Dev => get_stdout_tracer(),
            _ => get_otlp_tracer(),
        };
        let layer = tracing_opentelemetry::layer().with_tracer(tracer.tracer("axum-app"));
        Some((layer, tracer))
    } else {
        None
    };

    let (file_layer, file_guard) = file_layer
        .map(|(u, t)| (Some(u), Some(t)))
        .unwrap_or((None, None));

    let (otel_layer, otel_guard) = otel_layer
        .map(|(u, t)| (Some(u), Some(t)))
        .unwrap_or((None, None));

    let registry = Registry::default()
        .with(filter_layer)
        .with(stdout_layer)
        .with(file_layer)
        .with(otel_layer);

    tracing_log::LogTracer::init().expect("Failed to set log tracer");
    tracing::subscriber::set_global_default(registry).expect("Failed to set subscriber");

    (file_guard, otel_guard)
}

/// Init the stdout opentelemetry tracer
fn get_stdout_tracer() -> SdkTracerProvider {
    let ressources = get_ressources();

    SdkTracerProvider::builder()
        .with_simple_exporter(opentelemetry_stdout::SpanExporter::default())
        .with_resource(ressources)
        .build()
}

/// Init the OTLP opentelemetry tracer
fn get_otlp_tracer() -> SdkTracerProvider {
    let ressources = get_ressources();

    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    SdkTracerProvider::builder()
        .with_resource(ressources)
        .with_batch_exporter(exporter)
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
