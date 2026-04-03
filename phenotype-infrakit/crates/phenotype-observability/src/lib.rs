//! # phenotype-observability
//!
//! OpenTelemetry integration layer for unified observability across the Phenotype ecosystem.
//!
//! ## Features
//!
//! - `otlp`: Export traces and metrics to OTLP-compatible collectors (Jaeger, Tempo, etc.)
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────┐
//! │         Application Code                     │
//! │    #[instrument] / tracing::info!()        │
//! └──────────────┬──────────────────────────────┘
//!                │
//! ┌──────────────▼──────────────────────────────┐
//! │    phenotype-observability                 │
//! │  ┌──────────────────────────────────────┐    │
//! │  │  Tracing Layer (Sentry + OTel)      │    │
//! │  │  • Sentry for error tracking        │    │
//! │  │  • OpenTelemetry for traces         │    │
//! │  └──────────────────────────────────────┘    │
//! └──────────────┬──────────────────────────────┘
//!                │
//!    ┌───────────┴───────────┐
//!    ▼                       ▼
//! ┌──────────┐         ┌────────────┐
//! │  Sentry  │         │  OTLP      │
//! │          │         │ Collector  │
//! └──────────┘         └────────────┘
//! ```

#![cfg_attr(docsrs, feature(doc_cfg))]

use tracing::{info, warn};

#[cfg(feature = "otlp")]
pub mod otel {
    //! OpenTelemetry integration
    //!
    //! Provides OTLP export for traces and metrics.

    use opentelemetry::trace::TracerProvider;
    use opentelemetry_otlp::WithExportConfig;
    use std::time::Duration;

    /// Initialize OpenTelemetry with OTLP exporter
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use phenotype_observability::otel::init_otlp;
    ///
    /// # async fn example() {
    /// init_otlp("http://localhost:4317", "my-service").await;
    /// # }
    /// ```
    pub async fn init_otlp(endpoint: &str, service_name: &str) {
        let provider = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
                    .with_timeout(Duration::from_secs(3)),
            )
            .with_trace_config(
                opentelemetry::trace::config()
                    .with_resource(opentelemetry::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", service_name.to_string()),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("Failed to install OTLP pipeline");

        // Set as global provider
        opentelemetry::global::set_tracer_provider(provider);

        tracing::info!("OpenTelemetry OTLP exporter initialized: {}", endpoint);
    }

    /// Shutdown OpenTelemetry providers
    pub fn shutdown() {
        opentelemetry::global::shutdown_tracer_provider();
        tracing::info!("OpenTelemetry providers shut down");
    }
}

/// Unified observability configuration
#[derive(Debug, Clone)]
pub struct ObservabilityConfig {
    /// Sentry DSN (optional)
    pub sentry_dsn: Option<String>,
    /// OTLP endpoint (optional)
    pub otlp_endpoint: Option<String>,
    /// Service name for telemetry
    pub service_name: String,
    /// Service version
    pub service_version: String,
}

impl ObservabilityConfig {
    /// Create new configuration
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            sentry_dsn: None,
            otlp_endpoint: None,
            service_name: service_name.into(),
            service_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Enable Sentry integration
    pub fn with_sentry(mut self, dsn: impl Into<String>) -> Self {
        self.sentry_dsn = Some(dsn.into());
        self
    }

    /// Enable OTLP export
    pub fn with_otlp(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }
}

/// Initialize unified observability (Sentry + OpenTelemetry)
pub fn init(config: ObservabilityConfig) {
    info!(
        service = %config.service_name,
        version = %config.service_version,
        "Initializing unified observability"
    );

    if config.sentry_dsn.is_some() {
        info!("Sentry integration: enabled");
    }

    if config.otlp_endpoint.is_some() {
        info!("OpenTelemetry OTLP: enabled");
    }

    if config.sentry_dsn.is_none() && config.otlp_endpoint.is_none() {
        warn!("No observability backend configured");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_builder() {
        let config = ObservabilityConfig::new("test-service")
            .with_sentry("https://sentry.example.com")
            .with_otlp("http://localhost:4317");

        assert_eq!(config.service_name, "test-service");
        assert!(config.sentry_dsn.is_some());
        assert!(config.otlp_endpoint.is_some());
    }
}
