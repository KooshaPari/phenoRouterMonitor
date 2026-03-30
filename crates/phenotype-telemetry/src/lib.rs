//! Canonical telemetry (tracing + metrics + logs) for Phenotype services.
//!
//! This crate provides comprehensive OpenTelemetry integration including:
//! - Structured tracing with JSON/console output and OTLP export
//! - Metrics collection (counters and histograms)
//! - Health checks and status reporting
//! - Span helpers and timed operation utilities

pub mod error;
pub mod health;
pub mod metrics;
pub mod tracing;

pub use error::{TelemetryError, TelemetryResult};
pub use health::{HealthCheckResult, HealthStatus, TelemetryHealth};
pub use metrics::{init_metrics, Counter, Histogram, Metrics};
pub use tracing::{create_span, init_tracing, timed_operation, TracingConfig};
