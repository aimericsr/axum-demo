use crate::config::Otel;
use core::time::Duration;
use opentelemetry::global;
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::trace::config;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator, Sampler};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, SERVICE_NAMESPACE, SERVICE_VERSION,
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

/// Set the subscriber as the default for the lifetime of the applications.
pub fn init_subscriber(otel: &Otel) {
    let subscriber = get_subscriber(otel);
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Retreive the subscriber configured
fn get_subscriber(otel: &Otel) -> impl Subscriber + Sync + Send {
    // Config which trace levels to collect
    let env_filter = EnvFilter::builder().try_from_env().unwrap();

    // Config multiple target to send traces
    let stdout_layer = if otel.stdout_enabled {
        Some(tracing_subscriber::fmt::layer().json())
    } else {
        None
    };

    let otel_layer = if otel.enabled {
        let tracer = init_otlp_traces(otel).expect("Failed to init the otlp tracer");
        let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        Some(opentelemetry_layer)
    } else {
        None
    };

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(otel_layer)
}

/// Init the opentelemetry tracer
fn init_otlp_traces(otel: &Otel) -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otel.endpoint.clone())
                .with_timeout(Duration::from_secs(3)),
            //.with_compression(Compression::Gzip),
        )
        .with_trace_config(
            config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_attributes_per_event(128)
                .with_max_attributes_per_link(128)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.schema.url", SCHEMA_URL),
                    KeyValue::new(SERVICE_NAME, otel.service_name.clone()),
                    KeyValue::new(SERVICE_VERSION, otel.service_version.clone()),
                    KeyValue::new(SERVICE_NAMESPACE, otel.service_namespace.clone()),
                ]))
                .with_sampler(Sampler::AlwaysOn),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
}
