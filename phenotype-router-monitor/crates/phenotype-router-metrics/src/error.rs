//! Error types for metrics

use thiserror::Error;

/// Result type for metrics operations
pub type Result<T> = std::result::Result<T, MetricsError>;

/// Metrics error type
#[derive(Debug, Error)]
pub enum MetricsError {
    #[error("Failed to export metrics: {0}")]
    ExportError(String),

    #[error("Invalid metric name: {0}")]
    InvalidMetricName(String),

    #[error("Invalid metric value: {0}")]
    InvalidMetricValue(String),

    #[error("Prometheus error: {0}")]
    PrometheusError(String),

    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    #[error("Internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    // Traces to: FR-ROUTER-008 (Metrics error handling)
    #[test]
    fn test_export_error() {
        let err = MetricsError::ExportError("connection failed".to_string());
        assert!(err.to_string().contains("connection failed"));
    }

    // Traces to: FR-ROUTER-008
    #[test]
    fn test_invalid_metric_name() {
        let err = MetricsError::InvalidMetricName("bad-name".to_string());
        assert!(err.to_string().contains("bad-name"));
    }
}
