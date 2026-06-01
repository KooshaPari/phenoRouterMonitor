//! Health check framework for phenotype infrastructure.
//!
//! Provides a trait for implementing health checks that can be queried
//! by monitoring systems.

use serde::Serialize;

/// Health check status.
#[derive(Debug, Clone, Serialize)]
pub enum HealthStatus {
    /// The component is healthy.
    Healthy,
    /// The component is degraded but functional.
    Degraded(String),
    /// The component is unhealthy.
    Unhealthy(String),
}

impl HealthStatus {
    /// Returns true if the status indicates health.
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
}

/// Result of a health check.
#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub name: String,
    pub status: HealthStatus,
}

impl HealthReport {
    /// Create a new healthy report.
    pub fn healthy(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Healthy,
        }
    }

    /// Create a new degraded report.
    pub fn degraded(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Degraded(reason.into()),
        }
    }

    /// Create a new unhealthy report.
    pub fn unhealthy(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: HealthStatus::Unhealthy(reason.into()),
        }
    }
}

/// Trait for implementing health checks.
///
/// Implement this trait to provide health check functionality for components.
pub trait HealthCheck: Send + Sync {
    /// Perform the health check.
    fn check(&self) -> HealthReport;
}
