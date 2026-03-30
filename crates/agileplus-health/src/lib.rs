//! Standardized health check abstraction for AgilePlus.
//!
//! Replaces duplicate health check implementations across AgilePlus:
//! - `GraphHealth` in agileplus-graph
//! - `CacheHealth` in agileplus-cache  
//! - `BusHealth` in agileplus-nats
//! - `HealthStatus` in agileplus-domain

use async_trait::async_trait;

/// Standardized health status for all AgilePlus services.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthStatus {
    /// Service is fully operational
    Healthy,
    /// Service is degraded but operational
    Degraded,
    /// Service is unavailable
    Unavailable,
}

impl HealthStatus {
    /// Returns true if the service is operational
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }

    /// Returns true if the service needs attention
    pub fn needs_attention(&self) -> bool {
        !matches!(self, HealthStatus::Healthy)
    }
}

/// Errors during health checks
#[derive(Debug, thiserror::Error)]
pub enum HealthError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Timeout: {0}")]
    Timeout(String),
    #[error("Check failed: {0}")]
    CheckFailed(String),
}

/// Trait for health check implementations
#[async_trait]
pub trait HealthCheck: Send + Sync {
    async fn check_health(&self) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockHealthy;
    struct MockUnhealthy;

    #[async_trait::async_trait]
    impl HealthCheck for MockHealthy {
        async fn check_health(&self) -> HealthStatus {
            HealthStatus::Healthy
        }
    }

    #[async_trait::async_trait]
    impl HealthCheck for MockUnhealthy {
        async fn check_health(&self) -> HealthStatus {
            HealthStatus::Unavailable
        }
    }

    #[tokio::test]
    async fn test_health_status() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(!HealthStatus::Unavailable.is_operational());
    }
}
