//! Core metric types and data structures

use serde::{Deserialize, Serialize};

/// Metric name/identifier
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MetricName(pub String);

impl MetricName {
    /// Create a new metric name
    pub fn new(name: impl Into<String>) -> Self {
        Self(name.into())
    }

    /// Get metric name as string
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for MetricName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Metric value
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MetricValue {
    /// Counter (monotonically increasing)
    Counter(u64),

    /// Gauge (can increase or decrease)
    Gauge(f64),

    /// Histogram bucket (with count)
    Histogram { sum: f64, count: u64 },
}

/// Request-specific metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestMetrics {
    /// Request path
    pub path: String,

    /// Response status code
    pub status: u16,

    /// Response time in milliseconds
    pub latency_ms: u64,

    /// Service name
    pub service: String,

    /// Backend URL
    pub backend: String,

    /// Request size in bytes
    pub request_size: u64,

    /// Response size in bytes
    pub response_size: u64,

    /// Timestamp (Unix milliseconds)
    pub timestamp_ms: u64,
}

impl RequestMetrics {
    /// Create new request metrics
    pub fn new(
        path: impl Into<String>,
        status: u16,
        latency_ms: u64,
        service: impl Into<String>,
        backend: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            status,
            latency_ms,
            service: service.into(),
            backend: backend.into(),
            request_size: 0,
            response_size: 0,
            timestamp_ms: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
        }
    }

    /// Set request size
    pub fn with_request_size(mut self, size: u64) -> Self {
        self.request_size = size;
        self
    }

    /// Set response size
    pub fn with_response_size(mut self, size: u64) -> Self {
        self.response_size = size;
        self
    }

    /// Check if response is 2xx
    pub fn is_success(&self) -> bool {
        (200..=299).contains(&self.status)
    }

    /// Check if response is 4xx
    pub fn is_client_error(&self) -> bool {
        (400..=499).contains(&self.status)
    }

    /// Check if response is 5xx
    pub fn is_server_error(&self) -> bool {
        (500..=599).contains(&self.status)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-009 (Request metrics)
    #[test]
    fn test_metric_name() {
        let name = MetricName::new("http_requests_total");
        assert_eq!(name.as_str(), "http_requests_total");
    }

    // Traces to: FR-ROUTER-009
    #[test]
    fn test_metric_value_counter() {
        let value = MetricValue::Counter(42);
        assert!(matches!(value, MetricValue::Counter(42)));
    }

    // Traces to: FR-ROUTER-009
    #[test]
    fn test_metric_value_gauge() {
        let value = MetricValue::Gauge(3.14);
        assert!(matches!(value, MetricValue::Gauge(v) if (v - 3.14).abs() < 0.01));
    }

    // Traces to: FR-ROUTER-009
    #[test]
    fn test_request_metrics_creation() {
        let metrics = RequestMetrics::new("/api/users", 200, 150, "api", "http://localhost:3000");
        assert_eq!(metrics.path, "/api/users");
        assert_eq!(metrics.status, 200);
        assert_eq!(metrics.latency_ms, 150);
    }

    // Traces to: FR-ROUTER-009
    #[test]
    fn test_request_metrics_with_sizes() {
        let metrics = RequestMetrics::new("/api/users", 200, 150, "api", "http://localhost:3000")
            .with_request_size(1024)
            .with_response_size(2048);
        assert_eq!(metrics.request_size, 1024);
        assert_eq!(metrics.response_size, 2048);
    }

    // Traces to: FR-ROUTER-009
    #[test]
    fn test_request_metrics_status_classification() {
        let success = RequestMetrics::new("/api/users", 200, 150, "api", "http://localhost:3000");
        assert!(success.is_success());
        assert!(!success.is_client_error());
        assert!(!success.is_server_error());

        let client_err =
            RequestMetrics::new("/api/users", 404, 150, "api", "http://localhost:3000");
        assert!(!client_err.is_success());
        assert!(client_err.is_client_error());
        assert!(!client_err.is_server_error());

        let server_err =
            RequestMetrics::new("/api/users", 500, 150, "api", "http://localhost:3000");
        assert!(!server_err.is_success());
        assert!(!server_err.is_client_error());
        assert!(server_err.is_server_error());
    }
}
