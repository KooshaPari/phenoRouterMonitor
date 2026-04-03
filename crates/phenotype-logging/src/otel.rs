//! OpenTelemetry integration.

#![cfg(feature = "otel")]

use opentelemetry::trace::TracerProvider;
use opentelemetry_sdk::{runtime, trace as sdktrace, Resource};
use opentelemetry_otlp::SpanExporter;
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Initialize tracing with OpenTelemetry OTLP exporter.
pub fn init_with_otel(service_name: &str, otlp_endpoint: &str) -> Result<(), OTelError> {
    // Create OTLP exporter
    let otlp_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(|e: opentelemetry::trace::TraceError| {
            OTelError::ExportError(e.to_string())
        })?;

    // Create tracer provider
    let tracer_provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .with_resource(Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name.to_string()),
        ]))
        .build();

    let tracer = tracer_provider.tracer(service_name);

    // Create OpenTelemetry tracing layer
    let otel_layer = OpenTelemetryLayer::new().with_tracer(tracer);

    // Initialize subscriber with OTLP
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

/// Initialize OpenTelemetry with a custom resource.
pub fn init_with_resource(
    service_name: &str,
    otlp_endpoint: &str,
    attributes: Vec<(&str, &str)>,
) -> Result<(), OTelError> {
    let resource = Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", service_name.to_string()),
    ])
    .merge(&Resource::new(
        attributes
            .into_iter()
            .map(|(k, v)| opentelemetry::KeyValue::new(k.to_string(), v.to_string()))
            .collect::<Vec<_>>(),
    ));

    let otlp_exporter = SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(|e: opentelemetry::trace::TraceError| {
            OTelError::ExportError(e.to_string())
        })?;

    let tracer_provider = sdktrace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer(service_name);

    let otel_layer = OpenTelemetryLayer::new().with_tracer(tracer);

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .with(otel_layer)
        .init();

    Ok(())
}

/// OpenTelemetry errors.
#[derive(Debug, thiserror::Error)]
pub enum OTelError {
    #[error("export error: {0}")]
    ExportError(String),

    #[error("configuration error: {0}")]
    ConfigError(String),

    #[error("runtime error: {0}")]
    RuntimeError(String),
}
