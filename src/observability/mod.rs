/// Create metrics for the application and then expose to the /metrics endpoint.
pub mod metrics;
/// Handle sending traces to the destinations (stdout and to an endpoint with the opentelemetry format).
pub mod tracing;
