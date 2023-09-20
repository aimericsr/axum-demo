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

// -- TODO: See if we can use the classic OpenTelemetry exporter insted of the Jaeger one

struct JaegerConfig {
    jaeger_agent_host: String,
    jaeger_agent_port: String,
    jaeger_tracing_service_name: String,
}

pub fn create_tracer_from_env() -> Option<sdktrace::Tracer> {
    let jaeger_enabled: bool = env::var("JAEGER_ENABLED")
        .unwrap_or_else(|_| "false".into())
        .parse()
        .unwrap();

    if jaeger_enabled {
        let config = get_jaeger_config_from_env();
        Some(init_tracer(config))
    } else {
        None
    }
}

fn init_tracer(config: JaegerConfig) -> sdktrace::Tracer {
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
        jaeger_agent_host: env::var("JAEGER_AGENT_HOST").unwrap_or_else(|_| "localhost".into()),
        jaeger_agent_port: env::var("JAEGER_AGENT_PORT").unwrap_or_else(|_| "4317".into()),
        jaeger_tracing_service_name: env::var("TRACING_SERVICE_NAME")
            .unwrap_or_else(|_| "axum-graphql".into()),
    }
}
