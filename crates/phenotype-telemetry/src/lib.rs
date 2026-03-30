//! Canonical telemetry (metrics collection) for Phenotype services.
//!
//! Provides [`MetricsRegistry`] for registering and updating counters, gauges,
//! and histograms, plus [`SpanTimer`] for Drop-based duration measurement and
//! [`MetricsSnapshot`] for point-in-time JSON-serializable reporting.

mod registry;
mod snapshot;
mod timer;

pub use registry::{Counter, Gauge, Histogram, Metric, MetricsRegistry, TelemetryConfig};
pub use snapshot::MetricsSnapshot;
pub use timer::{timed, SpanTimer};

/// Initialise a [`MetricsRegistry`] from the given configuration.
///
/// This is the primary entry point for setting up telemetry in a service.
pub fn init_telemetry(config: TelemetryConfig) -> MetricsRegistry {
    tracing::info!(
        service = %config.service_name,
        env = %config.environment,
        "initialising telemetry"
    );
    MetricsRegistry::new(config)
}
