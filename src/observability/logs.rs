use crate::config::Config;

use super::get_ressources;
use opentelemetry_appender_tracing::layer;
use tracing::Subscriber;
use tracing_subscriber::{EnvFilter, Layer, registry::LookupSpan};

pub fn logs_layer<S>(_conf: &Config) -> impl Layer<S>
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    let exporter = opentelemetry_otlp::LogExporter::builder()
        .with_tonic()
        .build()
        .unwrap();

    let ressources = get_ressources();

    let provider = opentelemetry_sdk::logs::SdkLoggerProvider::builder()
        .with_resource(ressources)
        .with_simple_exporter(exporter)
        .build();

    let filter_otel = EnvFilter::new("info")
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap());

    layer::OpenTelemetryTracingBridge::new(&provider)
}
