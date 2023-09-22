use core::time::Duration;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::KeyValue;
use opentelemetry::{global, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use crate::config::config;

/// Init tracing for the lifetime of the application
pub fn get_subscriber() {
    let env_filter = EnvFilter::builder().try_from_env().unwrap();
    let tracer = init_optl_tracer();

    Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .expect("Failed to init the registry");
}

/// Init the opentelemetry tracer
fn init_optl_tracer() -> sdktrace::Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        // where to send the traces
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(format!(
                    "http://{}:{}",
                    &config().jeager.agent_host,
                    &config().jeager.agent_port
                ))
                .with_timeout(Duration::from_secs(3)),
        )
        .with_trace_config(
            trace::config()
                .with_sampler(Sampler::AlwaysOn)
                .with_id_generator(RandomIdGenerator::default())
                .with_max_events_per_span(64)
                .with_max_attributes_per_span(16)
                .with_max_events_per_span(16)
                .with_resource(Resource::new(vec![
                    KeyValue::new("service.name", &*config().jeager.tracing_service_name),
                    KeyValue::new("service.version", "v1"),
                ])),
        )
        // batch exporter instead of exporting each span synchronously on drop
        .with_batch_config(BatchConfig::default())
        .install_batch(Tokio)
        .expect("Opentelemetry pipeline install error")
}
