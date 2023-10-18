use core::time::Duration;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry::{global, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::{Compression, WithExportConfig};
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, SERVICE_NAMESPACE, SERVICE_VERSION,
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use crate::config::config;

/// Init tracing for the lifetime of the application

///  # Overview
///
/// ```
/// init_subscriber();
/// ```

/// Set the subscriber as the default for the lifetime of the applications.
pub fn init_subscriber() {
    let subscriber = get_subscriber();
    set_global_default(subscriber).expect("Failed to set subscriber");
}

/// Retreive the subscriber configured
fn get_subscriber() -> impl Subscriber + Sync + Send {
    // Config which trace levels to collect
    let env_filter = EnvFilter::builder().try_from_env().unwrap();

    // Config multiple target to send traces
    let stdout_layer = tracing_subscriber::fmt::layer().json();

    let tracer = init_otlp_traces().expect("Failed to init the otlp tracer");
    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(opentelemetry_layer)
}

/// Init the opentelemetry tracer
fn init_otlp_traces() -> Result<sdktrace::Tracer, TraceError> {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(&config().otel.endpoint)
                .with_timeout(Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_attributes_per_event(128)
                .with_max_attributes_per_link(128)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.schema.url", SCHEMA_URL),
                    SERVICE_NAME.string(&*config().otel.service_name),
                    SERVICE_VERSION.string(&*config().otel.service_version),
                    SERVICE_NAMESPACE.string(&*config().otel.service_namespace),
                ]))
                .with_sampler(Sampler::AlwaysOn),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(Tokio)
}
