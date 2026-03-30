//! Health check port for service readiness and liveness probes.
//!
//! Health checks provide visibility into system component status.
//! Synchronous implementation allows for use in HTTP endpoints.

use std::fmt::Debug;

/// Health status of a component.
///
/// # Variants
///
/// - `Healthy`: Component is functioning normally.
/// - `Degraded`: Component is operational but with reduced capacity or partial failure.
/// - `Unhealthy`: Component is not functioning and should not receive requests.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HealthStatus {
    Healthy,
    Degraded(String),
    Unhealthy(String),
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded(msg) => write!(f, "degraded: {}", msg),
            HealthStatus::Unhealthy(msg) => write!(f, "unhealthy: {}", msg),
        }
    }
}

/// Health check port for synchronous status reporting.
///
/// Implementations report the health of a specific component or subsystem.
/// Multiple health checks can be aggregated to form a complete system health picture.
///
/// # Example
///
/// ```ignore
/// struct DatabaseHealthCheck {
///     pool: Arc<ConnectionPool>,
/// }
///
/// impl HealthCheck for DatabaseHealthCheck {
///     fn name(&self) -> &str {
///         "database"
///     }
///
///     fn check(&self) -> HealthStatus {
///         match self.pool.ping() {
///             Ok(_) => HealthStatus::Healthy,
///             Err(e) => HealthStatus::Unhealthy(e.to_string()),
///         }
///     }
/// }
/// ```
pub trait HealthCheck: Send + Sync + Debug {
    /// Get the name of this health check component.
    fn name(&self) -> &str;

    /// Perform the health check and return the status.
    ///
    /// Should be implemented to complete quickly (< 1 second recommended).
    fn check(&self) -> HealthStatus;
}
