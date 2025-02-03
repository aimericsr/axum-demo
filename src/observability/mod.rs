use crate::config::Tracing;
use opentelemetry::KeyValue;
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::resource::{
    SERVICE_NAME, SERVICE_NAMESPACE, SERVICE_VERSION,
};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use std::time::Duration;

pub(crate) fn get_ressources(otel: &Tracing) -> Resource {
    let detected_ressources = Resource::from_detectors(
        Duration::from_millis(10),
        vec![
            Box::<EnvResourceDetector>::default(),
            Box::new(SdkProvidedResourceDetector),
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

    detected_ressources.merge(&default_ressources)
}

/// Handle sending metrics to different destinations using the OTEL format (stdout and via the netowork)
pub mod metrics;
/// Handle sending traces to different destinations using the OTEL format (stdout and via the netowork)
pub mod traces;
