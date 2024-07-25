use crate::config::Otel;
use core::time::Duration;
use opentelemetry::global;
use opentelemetry::trace::TracerProvider;
use opentelemetry::KeyValue;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::resource::{EnvResourceDetector, TelemetryResourceDetector};
use opentelemetry_sdk::trace::Config;
use opentelemetry_sdk::trace::{BatchConfig, RandomIdGenerator, Sampler};
use opentelemetry_sdk::{trace::Tracer, Resource};
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
        let tracer = init_otlp_traces(otel);
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
fn init_otlp_traces(otel: &Otel) -> Tracer {
    global::set_text_map_propagator(TraceContextPropagator::new());

    let detectors_ressources = Resource::from_detectors(
        Duration::from_millis(10),
        vec![
            Box::<EnvResourceDetector>::default(),
            Box::new(TelemetryResourceDetector),
            Box::new(OsResourceDetector),
            Box::new(ProcessResourceDetector),
            Box::<HostResourceDetector>::default(),
        ],
    );

    let default_ressources = Resource::new(vec![
        KeyValue::new("service.schema.url", SCHEMA_URL),
        KeyValue::new(SERVICE_NAME, otel.service_name.clone()),
        KeyValue::new(SERVICE_VERSION, otel.service_version.clone()),
        KeyValue::new(SERVICE_NAMESPACE, otel.service_namespace.clone()),
    ]);

    let ressources = detectors_ressources.merge(&default_ressources);

    let provider = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(otel.endpoint.clone())
                .with_timeout(Duration::from_secs(5)),
            //.with_compression(opentelemetry_otlp::Compression::Gzip),
        )
        .with_trace_config(
            Config::default()
                .with_id_generator(RandomIdGenerator::default())
                .with_max_attributes_per_event(128)
                .with_max_attributes_per_link(128)
                .with_resource(ressources)
                .with_sampler(Sampler::AlwaysOn),
        )
        .with_batch_config(BatchConfig::default())
        .install_batch(opentelemetry_sdk::runtime::Tokio)
        .unwrap();

    let tracer = provider
        .tracer_builder("opentelemetry-otlp")
        .with_version(env!("CARGO_PKG_VERSION"))
        .build();
    let _ = global::set_tracer_provider(provider);

    tracer
}
