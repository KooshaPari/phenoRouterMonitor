//! Observability port — telemetry abstraction (traces, metrics, logs).
//!
//! All methods are synchronous for fire-and-forget semantics.
//!
//! Traceability: FR-OBSERVE-* / WP05-T029

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Context identifying a span within a distributed trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanContext {
    pub trace_id: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
}

/// A metric data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetricValue {
    Counter(u64),
    Histogram(f64),
    Gauge(f64),
}

/// Severity level for log entries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

/// A structured log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub fields: HashMap<String, String>,
    pub span_context: Option<SpanContext>,
}

/// Port for observability operations (traces, metrics, logs).
///
/// All methods are synchronous — telemetry should never block business logic.
/// The Telemetry adapter (WP10) implements this with OpenTelemetry.
pub trait ObservabilityPort: Send + Sync {
    // -- Tracing --

    /// Start a new span, optionally parented to an existing span.
    fn start_span(&self, name: &str, parent: Option<&SpanContext>) -> SpanContext;

    /// End a span.
    fn end_span(&self, ctx: &SpanContext);

    /// Add an event to a span with key-value attributes.
    fn add_span_event(&self, ctx: &SpanContext, name: &str, attributes: &[(&str, &str)]);

    /// Mark a span as errored.
    fn set_span_error(&self, ctx: &SpanContext, error: &str);

    // -- Metrics --

    /// Increment a counter metric.
    fn record_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Record a histogram observation.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Set a gauge value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    // -- Logging --

    /// Emit a structured log entry.
    fn log(&self, entry: &LogEntry);

    /// Convenience: log at INFO level.
    fn log_info(&self, message: &str);

    /// Convenience: log at WARN level.
    fn log_warn(&self, message: &str);

    /// Convenience: log at ERROR level.
    fn log_error(&self, message: &str);
}
