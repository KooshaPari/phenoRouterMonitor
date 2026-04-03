//! # Phenotype Observability
//!
//! Unified observability facade providing:
//! - Distributed tracing initialization
//! - Metrics collection (counters, timers, histograms)
//! - Structured logging integration
//!
//! This crate serves as a lightweight abstraction that can be backed by
//! OpenTelemetry, Prometheus, or other observability backends.

pub mod tracer;
pub mod metrics;

// Re-export tracer components
pub use tracer::{init_tracer, TracerHandle};

// Re-export metrics components
pub use metrics::{Counter, Timer, MetricsCollector};

/// Record a counter value
pub fn record_counter(name: impl AsRef<str>, value: i64) {
    let metric_name = name.as_ref();
    println!("[METRIC] counter {} = {}", metric_name, value);
}

/// Increment a counter by 1
pub fn increment_counter(name: impl AsRef<str>) {
    record_counter(name, 1);
}

/// Record a gauge value
pub fn record_gauge(name: impl AsRef<str>, value: f64) {
    let metric_name = name.as_ref();
    println!("[METRIC] gauge {} = {}", metric_name, value);
}

/// Record a timing in milliseconds
pub fn record_timing(name: impl AsRef<str>, ms: u64) {
    let metric_name = name.as_ref();
    println!("[METRIC] timing {} = {}ms", metric_name, ms);
}

/// Create a span for distributed tracing
#[macro_export]
macro_rules! span {
    ($name:expr) => {
        #[cfg(feature = "tracing")]
        {
            tracing::span!(tracing::Level::INFO, $name)
        }
        #[cfg(not(feature = "tracing"))]
        {
            ()
        }
    };
}

/// Execute code within a span
#[macro_export]
macro_rules! in_span {
    ($name:expr, $code:block) => {{
        #[cfg(feature = "tracing")]
        {
            let _span = $crate::span!($name);
            let _enter = _span.enter();
            $code
        }
        #[cfg(not(feature = "tracing"))]
        {
            $code
        }
    }};
}
