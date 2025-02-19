/*!
    Set different observabilty pipelines for this app.

    This application only emits OpenTelemetry (OTEL) signals: metrics and traces. These signals
    are then sent over the network to an OTEL collector for filtering, transformation, and enrichment
    before being exported to a storage backend (e.g., Prometheus, Jaeger, or an observability platform).
*/

/// Handle sending metrics to different destinations using the OTEL format (stdout and via the netowork)
pub mod metrics;
/// Handle sending traces to different destinations using the OTEL format (stdout and via the netowork)
pub mod traces;

use crate::config::Tracing;
use opentelemetry::KeyValue;
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::Resource;
use opentelemetry_semantic_conventions::SCHEMA_URL;

fn get_ressources(_otel: &Tracing) -> Resource {
    let default_ressources = Resource::builder();

    let detected_ressources = default_ressources.with_detectors(&[
        Box::new(OsResourceDetector),
        Box::new(ProcessResourceDetector),
        Box::<HostResourceDetector>::default(),
    ]);

    let attributes_ressources =
        detected_ressources.with_attributes([KeyValue::new("service.schema.url", SCHEMA_URL)]);

    attributes_ressources.build()
}
