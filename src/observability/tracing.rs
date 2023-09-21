use core::time::Duration;
use opentelemetry::sdk::trace::{self, Sampler};
use opentelemetry::trace::Tracer;
use opentelemetry::KeyValue;
use opentelemetry::{global, runtime::Tokio, sdk::propagation::TraceContextPropagator};
use opentelemetry_otlp::{ExportConfig, WithExportConfig};
use opentelemetry_sdk::logs::Config;
use opentelemetry_sdk::runtime::RuntimeChannel;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator};
use opentelemetry_sdk::{metrics::MeterProvider, runtime, trace as sdktrace, Resource};
use std::env;
use std::fmt::Display;
use tonic::metadata::MetadataMap;
use tracing::Subscriber;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::fmt::format::{Format, Pretty};
use tracing_subscriber::fmt::layer;
use tracing_subscriber::fmt::MakeWriter;
use tracing_subscriber::layer::Layered;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{layer::SubscriberExt, Registry};
use tracing_subscriber::{EnvFilter, Layer};

use crate::config;

// -- TODO: See if we can use the classic OpenTelemetry exporter insted of the Jaeger one

struct JaegerConfig {
    jaeger_agent_host: String,
    jaeger_agent_port: i64,
    jaeger_tracing_service_name: String,
}

pub fn get_subscriber(env_filter: String) {
    let env_filter =
        EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(env_filter));
    let config = get_jaeger_config_from_env();
    let tracer = init_optl_tracer(config);

    Registry::default()
        .with(env_filter)
        .with(tracing_subscriber::fmt::layer().json())
        .with(tracing_opentelemetry::layer().with_tracer(tracer))
        .try_init()
        .expect("Failed");
}

fn init_optl_tracer(config: JaegerConfig) -> sdktrace::Tracer {
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
                    config.jaeger_agent_host, config.jaeger_agent_port
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
                    config.jaeger_tracing_service_name,
                )])),
        )
        // batch exporter instead of exporting each span synchronously on drop
        .with_batch_config(BatchConfig::default())
        .install_batch(Tokio)
        .expect("pipeline install error")
}

fn get_jaeger_config_from_env() -> JaegerConfig {
    JaegerConfig {
        jaeger_agent_host: config().JAEGER_AGENT_HOST.clone(),
        jaeger_agent_port: config().JAEGER_AGENT_PORT.clone(),
        jaeger_tracing_service_name: config().TRACING_SERVICE_NAME.clone(),
    }
}
