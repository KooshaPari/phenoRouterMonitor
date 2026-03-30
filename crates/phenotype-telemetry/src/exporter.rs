//! Telemetry exporters for sending metrics and traces to backends.

use serde::{Deserialize, Serialize};

/// Exporter configuration for telemetry backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExporterConfig {
    pub kind: ExporterKind,
    pub endpoint: Option<String>,
    pub timeout_secs: u64,
}

/// Supported telemetry export backends.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ExporterKind {
    /// OpenTelemetry Protocol (OTLP) gRPC exporter
    OtlpGrpc,
    /// OpenTelemetry Protocol (OTLP) HTTP exporter
    OtlpHttp,
    /// Jaeger exporter
    Jaeger,
    /// Prometheus exporter
    Prometheus,
    /// No-op exporter for testing
    Noop,
}

/// Result of an export operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportResult {
    pub success: bool,
    pub messages_sent: usize,
    pub error: Option<String>,
}

/// Trait for telemetry exporters.
pub trait Exporter: Send + Sync {
    /// Export telemetry data to the backend.
    fn export(&self, data: &[u8]) -> Result<ExportResult, String>;

    /// Get the exporter kind.
    fn kind(&self) -> ExporterKind;

    /// Check if the exporter is ready.
    fn is_ready(&self) -> bool;
}

/// No-op exporter for testing and development.
#[derive(Debug, Clone)]
pub struct NoopExporter;

impl Exporter for NoopExporter {
    fn export(&self, _data: &[u8]) -> Result<ExportResult, String> {
        Ok(ExportResult {
            success: true,
            messages_sent: 0,
            error: None,
        })
    }

    fn kind(&self) -> ExporterKind {
        ExporterKind::Noop
    }

    fn is_ready(&self) -> bool {
        true
    }
}

/// HTTP exporter for OTLP and other backends.
#[derive(Debug, Clone)]
pub struct HttpExporter {
    endpoint: String,
    timeout_secs: u64,
}

impl HttpExporter {
    /// Create a new HTTP exporter.
    pub fn new(endpoint: impl Into<String>, timeout_secs: u64) -> Self {
        Self {
            endpoint: endpoint.into(),
            timeout_secs,
        }
    }
}

impl Exporter for HttpExporter {
    fn export(&self, _data: &[u8]) -> Result<ExportResult, String> {
        // Placeholder: In production, this would use reqwest or similar to POST to the endpoint
        Ok(ExportResult {
            success: true,
            messages_sent: 1,
            error: None,
        })
    }

    fn kind(&self) -> ExporterKind {
        ExporterKind::OtlpHttp
    }

    fn is_ready(&self) -> bool {
        // Placeholder: Would check connectivity to endpoint
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exporter_config() {
        let config = ExporterConfig {
            kind: ExporterKind::OtlpHttp,
            endpoint: Some("http://localhost:4318".to_string()),
            timeout_secs: 10,
        };

        assert_eq!(config.timeout_secs, 10);
        assert!(config.endpoint.is_some());
    }

    #[test]
    fn noop_exporter() {
        let exporter = NoopExporter;
        assert!(exporter.is_ready());

        let result = exporter.export(b"test").unwrap();
        assert!(result.success);
    }

    #[test]
    fn http_exporter() {
        let exporter = HttpExporter::new("http://localhost:4318", 30);
        assert!(exporter.is_ready());

        let result = exporter.export(b"data").unwrap();
        assert!(result.success);
    }

    #[test]
    fn export_result_serialization() {
        let result = ExportResult {
            success: true,
            messages_sent: 5,
            error: None,
        };

        let json = serde_json::to_string(&result).unwrap();
        let deserialized: ExportResult = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.messages_sent, 5);
    }
}
