//! Health and observability ports: HealthChecker, Auditor.
//!
//! Consolidates `harness_teammates::ports::HealthCheckPort` and adds
//! a general-purpose auditing trait.

use async_trait::async_trait;

use crate::error::PortError;

/// Component health status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    /// Component is operating normally.
    Healthy,
    /// Component is partially degraded.
    Degraded(String),
    /// Component is unavailable.
    Unhealthy(String),
}

/// Health check port for any subsystem.
///
/// Implementations should be cheap and non-blocking where possible.
#[async_trait]
pub trait HealthChecker: Send + Sync {
    /// Human-readable component name.
    fn name(&self) -> &str;

    /// Perform a health check.
    async fn check(&self) -> HealthStatus;
}

/// Audit trail port for recording significant domain actions.
#[async_trait]
pub trait Auditor: Send + Sync {
    /// Record an audit entry.
    async fn record(
        &self,
        action: &str,
        actor: &str,
        details: &str,
    ) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct AlwaysHealthy;

    #[async_trait]
    impl HealthChecker for AlwaysHealthy {
        fn name(&self) -> &str { "test" }
        async fn check(&self) -> HealthStatus { HealthStatus::Healthy }
    }

    #[tokio::test]
    async fn health_checker_healthy() {
        let hc = AlwaysHealthy;
        assert_eq!(hc.check().await, HealthStatus::Healthy);
        assert_eq!(hc.name(), "test");
    }
}
