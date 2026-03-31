//! # phenotype-telemetry
//!
//! Lightweight telemetry abstractions for Phenotype ecosystem.
//!
//! Provides traits for metrics, tracing, and structured logging that can be
//! implemented by any backend (e.g., OpenTelemetry, Prometheus, etc.).
//!
//! # Example
//!
//! ```rust
//! use phenotype_telemetry::{MetricsRecorder, SpanContext, TelemetryError};
//!
//! fn record_metrics(recorder: &dyn MetricsRecorder) {
//!     recorder.increment_counter("requests_total", 1, &[]);
//!     recorder.record_histogram("request_duration_ms", 42.0, &[]);
//! }
//! ```

use serde::{Deserialize, Serialize};
use thiserror::Error;

// ---------------------------------------------------------------------------
// Error
// ---------------------------------------------------------------------------

/// Errors that can occur during telemetry operations.
#[derive(Debug, Error)]
#[error(transparent)]
pub struct TelemetryError {
    #[from]
    inner: Box<dyn std::error::Error + Send + Sync>,
}

impl TelemetryError {
    /// Create a new telemetry error with a message.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            inner: msg.into().into(),
        }
    }
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Trait for recording metrics.
///
/// Implementors can forward to Prometheus, OpenTelemetry, StatsD, etc.
pub trait MetricsRecorder: Send + Sync {
    /// Increment a counter by `value`.
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]);

    /// Record a histogram value.
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Record a gauge value.
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// No-op metrics recorder for testing or disabled telemetry.
pub struct NoopMetrics;

impl NoopMetrics {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopMetrics {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRecorder for NoopMetrics {
    fn increment_counter(&self, _name: &str, _value: u64, _labels: &[(&str, &str)]) {}
    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

// ---------------------------------------------------------------------------
// Tracing
// ---------------------------------------------------------------------------

/// Context for a distributed trace span.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SpanContext {
    /// Trace identifier (128-bit, hex-encoded).
    pub trace_id: String,
    /// Span identifier (64-bit, hex-encoded).
    pub span_id: String,
    /// Parent span identifier, if any.
    pub parent_span_id: Option<String>,
}

impl SpanContext {
    /// Create a new root span context with generated IDs.
    pub fn new(trace_id: impl Into<String>, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: trace_id.into(),
            span_id: span_id.into(),
            parent_span_id: None,
        }
    }

    /// Create a child span context.
    pub fn child(&self, span_id: impl Into<String>) -> Self {
        Self {
            trace_id: self.trace_id.clone(),
            span_id: span_id.into(),
            parent_span_id: Some(self.span_id.clone()),
        }
    }

    /// Check if this is a no-op/empty context.
    pub fn is_noop(&self) -> bool {
        self.trace_id.is_empty() || self.trace_id == "00000000000000000000000000000000"
    }
}

/// Trait for distributed tracing.
pub trait Tracer: Send + Sync {
    /// Start a new span.
    fn start_span(&self, name: &str, parent: Option<&SpanContext>) -> SpanContext;

    /// End a span.
    fn end_span(&self, ctx: &SpanContext);

    /// Add an event to a span.
    fn add_span_event(&self, ctx: &SpanContext, name: &str, attrs: &[(&str, &str)]);

    /// Record an error in a span.
    fn set_span_error(&self, ctx: &SpanContext, error: &str);
}

/// No-op tracer for testing or disabled telemetry.
pub struct NoopTracer;

impl NoopTracer {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopTracer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tracer for NoopTracer {
    fn start_span(&self, _name: &str, _parent: Option<&SpanContext>) -> SpanContext {
        SpanContext::default()
    }

    fn end_span(&self, _ctx: &SpanContext) {}
    fn add_span_event(&self, _ctx: &SpanContext, _name: &str, _attrs: &[(&str, &str)]) {}
    fn set_span_error(&self, _ctx: &SpanContext, _error: &str) {}
}

// ---------------------------------------------------------------------------
// Logging
// ---------------------------------------------------------------------------

/// Log severity levels.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::str::FromStr for LogLevel {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(match s.to_lowercase().as_str() {
            "trace" => Self::Trace,
            "debug" => Self::Debug,
            "info" => Self::Info,
            "warn" | "warning" => Self::Warn,
            "error" | "err" => Self::Error,
            _ => Self::Info,
        })
    }
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Trace => write!(f, "trace"),
            Self::Debug => write!(f, "debug"),
            Self::Info => write!(f, "info"),
            Self::Warn => write!(f, "warn"),
            Self::Error => write!(f, "error"),
        }
    }
}

/// A structured log entry.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub fields: std::collections::HashMap<String, serde_json::Value>,
    pub span_context: Option<SpanContext>,
}

impl LogEntry {
    /// Create a new log entry.
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
            timestamp: chrono::Utc::now(),
            fields: std::collections::HashMap::new(),
            span_context: None,
        }
    }

    /// Add a field to the entry.
    pub fn with_field(mut self, key: impl Into<String>, value: impl Serialize) -> Self {
        if let Ok(v) = serde_json::to_value(value) {
            self.fields.insert(key.into(), v);
        }
        self
    }

    /// Attach a span context.
    pub fn with_span(mut self, ctx: SpanContext) -> Self {
        self.span_context = Some(ctx);
        self
    }
}

/// Convenience constructors.
impl LogEntry {
    pub fn trace(msg: impl Into<String>) -> Self {
        Self::new(LogLevel::Trace, msg)
    }
    pub fn debug(msg: impl Into<String>) -> Self {
        Self::new(LogLevel::Debug, msg)
    }
    pub fn info(msg: impl Into<String>) -> Self {
        Self::new(LogLevel::Info, msg)
    }
    pub fn warn(msg: impl Into<String>) -> Self {
        Self::new(LogLevel::Warn, msg)
    }
    pub fn error(msg: impl Into<String>) -> Self {
        Self::new(LogLevel::Error, msg)
    }
}

/// Trait for structured logging.
pub trait Logger: Send + Sync {
    /// Log a structured entry.
    fn log(&self, entry: &LogEntry);

    /// Log at info level.
    fn log_info(&self, msg: &str);

    /// Log at warn level.
    fn log_warn(&self, msg: &str);

    /// Log at error level.
    fn log_error(&self, msg: &str);
}

/// No-op logger for testing or disabled telemetry.
pub struct NoopLogger;

impl NoopLogger {
    pub fn new() -> Self {
        Self
    }
}

impl Default for NoopLogger {
    fn default() -> Self {
        Self::new()
    }
}

impl Logger for NoopLogger {
    fn log(&self, _entry: &LogEntry) {}
    fn log_info(&self, _msg: &str) {}
    fn log_warn(&self, _msg: &str) {}
    fn log_error(&self, _msg: &str) {}
}

// ---------------------------------------------------------------------------
// Telemetry trait (combined interface)
// ---------------------------------------------------------------------------

/// Combined telemetry interface.
pub trait Telemetry: MetricsRecorder + Tracer + Logger {}

impl<T: MetricsRecorder + Tracer + Logger> Telemetry for T {}

/// No-op telemetry implementation.
pub struct NoopTelemetry {
    metrics: NoopMetrics,
    tracer: NoopTracer,
    logger: NoopLogger,
}

impl NoopTelemetry {
    pub fn new() -> Self {
        Self {
            metrics: NoopMetrics,
            tracer: NoopTracer,
            logger: NoopLogger,
        }
    }
}

impl Default for NoopTelemetry {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsRecorder for NoopTelemetry {
    fn increment_counter(&self, name: &str, value: u64, labels: &[(&str, &str)]) {
        self.metrics.increment_counter(name, value, labels)
    }
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        self.metrics.record_histogram(name, value, labels)
    }
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
        self.metrics.record_gauge(name, value, labels)
    }
}

impl Tracer for NoopTelemetry {
    fn start_span(&self, name: &str, parent: Option<&SpanContext>) -> SpanContext {
        self.tracer.start_span(name, parent)
    }
    fn end_span(&self, ctx: &SpanContext) {
        self.tracer.end_span(ctx)
    }
    fn add_span_event(&self, ctx: &SpanContext, name: &str, attrs: &[(&str, &str)]) {
        self.tracer.add_span_event(ctx, name, attrs)
    }
    fn set_span_error(&self, ctx: &SpanContext, error: &str) {
        self.tracer.set_span_error(ctx, error)
    }
}

impl Logger for NoopTelemetry {
    fn log(&self, entry: &LogEntry) {
        self.logger.log(entry)
    }
    fn log_info(&self, msg: &str) {
        self.logger.log_info(msg)
    }
    fn log_warn(&self, msg: &str) {
        self.logger.log_warn(msg)
    }
    fn log_error(&self, msg: &str) {
        self.logger.log_error(msg)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_level_from_str() {
        assert_eq!(LogLevel::from_str("debug"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("DEBUG"), Ok(LogLevel::Debug));
        assert_eq!(LogLevel::from_str("warning"), Ok(LogLevel::Warn));
        assert_eq!(LogLevel::from_str("err"), Ok(LogLevel::Error));
        assert_eq!(LogLevel::from_str("unknown"), Ok(LogLevel::Info));
    }

    #[test]
    fn test_log_entry_fluent() {
        let entry = LogEntry::info("test message")
            .with_field("user_id", "42")
            .with_field("count", 5);

        assert_eq!(entry.level, LogLevel::Info);
        assert_eq!(entry.message, "test message");
        assert_eq!(entry.fields.get("user_id").unwrap(), "42");
        assert_eq!(entry.fields.get("count").unwrap(), &5);
    }

    #[test]
    fn test_span_context_hierarchy() {
        let root = SpanContext::new("trace-123", "span-1");
        assert!(root.parent_span_id.is_none());

        let child = root.child("span-2");
        assert_eq!(child.trace_id, "trace-123");
        assert_eq!(child.span_id, "span-2");
        assert_eq!(child.parent_span_id, Some("span-1".to_string()));
    }

    #[test]
    fn test_span_context_is_noop() {
        let noop = SpanContext::default();
        assert!(noop.is_noop());

        let valid = SpanContext::new("abc", "def");
        assert!(!valid.is_noop());
    }

    #[test]
    fn test_noop_telemetry() {
        let telemetry = NoopTelemetry::new();

        telemetry.increment_counter("test", 1, &[]);
        telemetry.record_histogram("test", 42.0, &[]);
        telemetry.record_gauge("test", 1.0, &[]);

        let ctx = telemetry.start_span("test", None);
        telemetry.add_span_event(&ctx, "event", &[("k", "v")]);
        telemetry.set_span_error(&ctx, "oops");
        telemetry.end_span(&ctx);

        telemetry.log_info("hello");
        telemetry.log_warn("warn");
        telemetry.log_error("error");
        telemetry.log(&LogEntry::info("struct"));
    }

    #[test]
    fn test_telemetry_error() {
        let err = TelemetryError::new("test error");
        assert!(err.to_string().contains("test error"));
    }
}
