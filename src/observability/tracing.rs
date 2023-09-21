use core::time::Duration;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::KeyValue;
use opentelemetry::{global, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_sdk::{trace as sdktrace, Resource};
use tonic::metadata::MetadataMap;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::{layer::SubscriberExt, Registry};

use crate::config;

pub fn get_subscriber(env_filter: String) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let tracer = init_optl_tracer();

    Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .expect("Failed");
}

fn init_optl_tracer() -> sdktrace::Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());

    // Metadata
    let mut map = MetadataMap::with_capacity(1);
    map.insert("x-host", "example.com".parse().unwrap());

    opentelemetry_otlp::new_pipeline()
        .tracing()
        // where to send the traces
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                //.with_metadata(map)
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
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    &*config().jeager.tracing_service_name,
                )])),
        )
        // batch exporter instead of exporting each span synchronously on drop
        .with_batch_config(BatchConfig::default())
        .install_batch(Tokio)
        .expect("pipeline install error")
}

// fn get_jaeger_config_from_env() -> JaegerConfig {
//     JaegerConfig {
//         jaeger_agent_host: config().JAEGER_AGENT_HOST.clone(),
//         jaeger_agent_port: config().JAEGER_AGENT_PORT.clone(),
//         jaeger_tracing_service_name: config().TRACING_SERVICE_NAME.clone(),
//     }
// }
