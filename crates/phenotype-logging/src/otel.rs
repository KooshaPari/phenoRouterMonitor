//! OpenTelemetry integration.

use opentelemetry_sdk::{runtime, trace as sdktrace};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Initialize tracing with OpenTelemetry OTLP exporter.
pub fn init_with_otel(service_name: &str, otlp_endpoint: &str) -> Result<(), OTelError> {
    // Create OTLP exporter
    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(|e| OTelError::ExportError(e.to_string()))?;

    // Create tracer provider
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .with_resource(opentelemetry_sdk::Resource::new(vec![
            opentelemetry::KeyValue::new("service.name", service_name),
        ]))
        .build();

    let tracer = tracer_provider.tracer(service_name);

    // Create OpenTelemetry tracing layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

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
    let resource = opentelemetry_sdk::Resource::new(vec![
        opentelemetry::KeyValue::new("service.name", service_name),
    ])
    .merge(&opentelemetry_sdk::Resource::new(
        attributes
            .into_iter()
            .map(|(k, v)| opentelemetry::KeyValue::new(k, v))
            .collect::<Vec<_>>(),
    ));

    let otlp_exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(otlp_endpoint)
        .build()
        .map_err(|e| OTelError::ExportError(e.to_string()))?;

    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(otlp_exporter, runtime::Tokio)
        .with_resource(resource)
        .build();

    let tracer = tracer_provider.tracer(service_name);

    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

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
