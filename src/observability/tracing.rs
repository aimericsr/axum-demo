use core::time::Duration;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::trace::TraceError;
use opentelemetry::KeyValue;
use opentelemetry::{global, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tracing::subscriber::set_global_default;
use tracing::Subscriber;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use crate::config::config;

/// Init tracing for the lifetime of the application
pub fn init_subscriber() {
    let subscriber = get_subscriber();
    set_global_default(subscriber).expect("Failed to set subscriber");
}

fn get_subscriber() -> impl Subscriber + Sync + Send {
    // Config which trace levels to collect
    let env_filter = EnvFilter::builder().try_from_env().unwrap();

    // Config multiple target to send traces
    let stdout_layer = tracing_subscriber::fmt::layer().json();

    let tracer = init_optl_tracer().expect("Failed to init the otlp tracer");
    let opentelemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // let file = File::create("debug.log").expect("Failed to create log file");
    // let file_layer = tracing_subscriber::fmt::layer().with_writer(Arc::new(file));

    Registry::default()
        .with(env_filter)
        .with(stdout_layer)
        .with(opentelemetry_layer)
}

/// Init the opentelemetry tracer
fn init_optl_tracer() -> Result<sdktrace::Tracer, TraceError> {
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
}
