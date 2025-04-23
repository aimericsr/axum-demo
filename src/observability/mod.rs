/*!
    Set different observabilty pipelines for this app.

    This application only emits OpenTelemetry (OTEL) signals: metrics and traces. These signals
    are then sent over the network to an OTEL collector for filtering, transformation, and enrichment
    before being exported to a storage backend (e.g., Prometheus, Jaeger, or an observability platform).

    The ObservabilityGuard return by the init_observability will be the only interaction the app has with
    the observaiblity objects. The app must ensure these object is dropped inside a select branch with
    a timeout to try to export all remaing to remote endpoint before the app is exited.
*/

/// Handle sending metrics to different destinations using the OTEL format (stdout and via the netowork)
mod metrics;
/// Handle sending traces to different destinations using the OTEL format (stdout and via the netowork)
mod traces;

use opentelemetry::{KeyValue, metrics::Meter};
use opentelemetry_resource_detectors::{
    HostResourceDetector, OsResourceDetector, ProcessResourceDetector,
};
use opentelemetry_sdk::{Resource, metrics::SdkMeterProvider, trace::SdkTracerProvider};
use opentelemetry_semantic_conventions::SCHEMA_URL;
use tracing_appender::non_blocking::WorkerGuard;

use crate::config::Config;

pub struct ObservabilityGuard {
    pub meter: Meter,
    meter_provider: SdkMeterProvider,
    tracer_provider: Option<SdkTracerProvider>,
    file_guard: Option<WorkerGuard>,
}

impl Default for ObservabilityGuard {
    fn default() -> Self {
        let exporter = opentelemetry_sdk::metrics::InMemoryMetricExporter::default();

        let meter_provider = opentelemetry_sdk::metrics::SdkMeterProvider::builder()
            .with_reader(
                opentelemetry_sdk::metrics::PeriodicReader::builder(exporter.clone()).build(),
            )
            .build();
        let meter = opentelemetry::global::meter("axum_demo_test");
        ObservabilityGuard {
            meter,
            meter_provider,
            tracer_provider: None,
            file_guard: None,
        }
    }
}

impl ObservabilityGuard {
    /// Try to export all traces/metrics before the application shutdow
    pub async fn shutdown(self) {
        let mut jhs = Vec::with_capacity(3);

        let jh = tokio::task::spawn_blocking(move || self.meter_provider.shutdown().unwrap());
        jhs.push(jh);

        if let Some(tracer_provider) = self.tracer_provider {
            let jh = tokio::task::spawn_blocking(move || tracer_provider.shutdown().unwrap());
            jhs.push(jh);
        }

        if let Some(file_guard) = self.file_guard {
            let jh = tokio::task::spawn_blocking(move || drop(file_guard));
            jhs.push(jh);
        }

        for jh in jhs {
            jh.await.unwrap();
        }
    }
}

pub fn init_observability(config: &Config) -> ObservabilityGuard {
    let (meter_provider, meter) = metrics::init_metrics(config);

    let (file_guard, tracer_provider) = traces::init_traces(config);
    ObservabilityGuard {
        meter_provider,
        meter,
        tracer_provider,
        file_guard,
    }
}

fn get_ressources() -> Resource {
    let ressources = Resource::builder();

    let detectors = ressources.with_detectors(&[
        Box::new(OsResourceDetector),
        Box::new(ProcessResourceDetector),
        Box::<HostResourceDetector>::default(),
    ]);

    let attributes = detectors.with_attributes([KeyValue::new("service.schema.url", SCHEMA_URL)]);

    attributes.build()
}
