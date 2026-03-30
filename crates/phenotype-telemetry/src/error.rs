//! Error types for telemetry operations.

use thiserror::Error;

/// Telemetry operation errors.
#[derive(Error, Debug)]
pub enum TelemetryError {
    #[error("tracing initialization failed: {0}")]
    TracingInit(String),

    #[error("metrics initialization failed: {0}")]
    MetricsInit(String),

    #[error("OTLP export failed: {0}")]
    OtlpExport(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("health check failed: {0}")]
    HealthCheckFailed(String),
}

pub type TelemetryResult<T> = Result<T, TelemetryError>;
