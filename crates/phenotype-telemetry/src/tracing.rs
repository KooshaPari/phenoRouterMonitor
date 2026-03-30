//! Tracing setup for Phenotype services.

use crate::error::{TelemetryError, TelemetryResult};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use tracing::{Level, Span};
use tracing_subscriber::filter::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

/// Tracing configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Service name for tracing identification.
    pub service_name: String,
    /// Log level: "debug", "info", "warn", "error".
    pub log_level: String,
    /// Enable JSON output formatting.
    pub json_output: bool,
    /// Optional OTLP collector endpoint.
    pub otlp_endpoint: Option<String>,
}

impl TracingConfig {
    /// Create a new tracing configuration.
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            log_level: "info".to_string(),
            json_output: true,
            otlp_endpoint: None,
        }
    }

    /// Set the log level.
    pub fn with_log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }

    /// Enable or disable JSON output.
    pub fn with_json_output(mut self, enabled: bool) -> Self {
        self.json_output = enabled;
        self
    }

    /// Set OTLP collector endpoint.
    pub fn with_otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }
}

/// Initialize tracing with the given configuration.
pub fn init_tracing(config: &TracingConfig) -> TelemetryResult<()> {
    // Parse log level
    let level = Level::from_str(&config.log_level)
        .map_err(|_| TelemetryError::InvalidConfig(
            format!("invalid log level: {}", config.log_level)
        ))?;

    // Create env filter
    let env_filter = EnvFilter::new(level.to_string());

    // Set up OTLP if configured
    if let Some(endpoint) = &config.otlp_endpoint {
        _setup_otlp_tracing(endpoint, &config.service_name)
            .map_err(|e| TelemetryError::OtlpExport(e.to_string()))?;
    }

    // Initialize with fmt layer based on json_output setting
    if config.json_output {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer().json())
            .init();
    } else {
        tracing_subscriber::registry()
            .with(env_filter)
            .with(tracing_subscriber::fmt::layer())
            .init();
    }

    Ok(())
}

/// Set up OTLP tracing export (placeholder for actual implementation).
fn _setup_otlp_tracing(_endpoint: &str, _service_name: &str) -> TelemetryResult<()> {
    // This is a placeholder; full OTLP setup would require additional dependencies
    // and async runtime configuration. Actual implementation depends on deployment model.
    Ok(())
}

/// Create a named span for a new async operation.
pub fn create_span(_name: &str) -> Span {
    // Note: The tracing macro requires compile-time constant names.
    // This creates a generic span; for dynamic span names, use tracing::span!
    tracing::info_span!("operation")
}

/// Execute a closure with timing and span tracking.
pub fn timed_operation<F, T>(name: &str, f: F) -> T
where
    F: FnOnce() -> T,
{
    let span = tracing::info_span!("timed_operation", name = %name);
    let _guard = span.enter();
    let start = std::time::Instant::now();
    let result = f();
    let elapsed = start.elapsed();
    tracing::debug!(
        operation = name,
        duration_ms = elapsed.as_millis(),
        "operation completed"
    );
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_builder() {
        let config = TracingConfig::new("test-service")
            .with_log_level("debug")
            .with_json_output(false)
            .with_otlp_endpoint("http://localhost:4317");

        assert_eq!(config.service_name, "test-service");
        assert_eq!(config.log_level, "debug");
        assert!(!config.json_output);
        assert_eq!(config.otlp_endpoint, Some("http://localhost:4317".to_string()));
    }

    #[test]
    fn test_invalid_log_level() {
        let config = TracingConfig::new("test").with_log_level("invalid");
        let result = init_tracing(&config);
        assert!(result.is_err());
    }

    #[test]
    fn test_timed_operation() {
        let result = timed_operation("test_op", || 42);
        assert_eq!(result, 42);
    }

    #[test]
    fn test_create_span() {
        let _span = create_span("test_span");
        // Span is created successfully; actual span behavior verified by tracing infrastructure
    }
}
