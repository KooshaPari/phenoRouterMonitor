//! Health check port and related types.
//!
//! Provides a unified health check abstraction for verifying service dependencies.

use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Overall status of a health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum HealthStatus {
    /// Service is fully operational.
    Healthy,
    /// Service is operational but experiencing degraded performance.
    Degraded,
    /// Service is not operational.
    Unhealthy,
    /// Health status is unknown (not yet checked).
    #[default]
    Unknown,
}

impl HealthStatus {
    /// Returns the worse of two statuses.
    ///
    /// Priority: Unhealthy > Degraded > Unknown > Healthy
    pub fn worse(self, other: Self) -> Self {
        use HealthStatus::*;
        match (self, other) {
            (Unhealthy, _) | (_, Unhealthy) => Unhealthy,
            (Degraded, _) | (_, Degraded) => Degraded,
            (Unknown, _) | (_, Unknown) => Unknown,
            _ => Healthy,
        }
    }

    /// Check if this status is considered "up" (Healthy or Degraded).
    pub fn is_up(&self) -> bool {
        matches!(self, Self::Healthy | Self::Degraded)
    }

    /// Check if this status is considered "down" (Unhealthy).
    pub fn is_down(&self) -> bool {
        matches!(self, Self::Unhealthy)
    }
}

/// A single health check probe.
///
/// Implement this trait for each dependency your service needs to verify
/// (database, cache, external API, etc.).
pub trait HealthChecker: Send + Sync {
    /// Human-readable name for this checker (e.g., "postgres", "redis").
    fn name(&self) -> &str;

    /// Run the check and return the status.
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;
}

/// Result of a single health check execution.
#[derive(Debug, Clone)]
pub struct HealthCheckResult {
    /// Name of the service checked.
    pub service: String,
    /// Status of the check.
    pub status: HealthStatus,
    /// Duration of the check.
    pub duration: Duration,
    /// Optional details about the check.
    pub details: Option<String>,
}

/// Configuration for health checking behavior.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Whether health checks are enabled.
    pub enabled: bool,
    /// Interval between checks.
    pub interval: Duration,
    /// Timeout for each check.
    pub timeout: Duration,
    /// Number of consecutive successes before marking as healthy.
    pub success_threshold: u32,
    /// Number of consecutive failures before marking as unhealthy.
    pub failure_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            success_threshold: 1,
            failure_threshold: 3,
        }
    }
}

/// Aggregate health response suitable for JSON serialization.
#[derive(Debug, Clone)]
pub struct HealthResponse {
    /// Overall status across all checks.
    pub status: HealthStatus,
    /// Individual check results.
    pub checks: Vec<HealthCheckResult>,
}

/// Manages multiple health checkers and aggregates results.
pub struct HealthMonitor {
    checkers: Vec<Box<dyn HealthChecker>>,
    config: HealthCheckConfig,
}

impl HealthMonitor {
    /// Create a new health monitor.
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
            config: HealthCheckConfig::default(),
        }
    }

    /// Create a new health monitor with custom config.
    pub fn with_config(config: HealthCheckConfig) -> Self {
        Self {
            checkers: Vec::new(),
            config,
        }
    }

    /// Add a health checker.
    pub fn add_checker(&mut self, checker: impl HealthChecker + 'static) {
        self.checkers.push(Box::new(checker));
    }

    /// Run all registered checkers and return individual results.
    pub async fn check_all(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::with_capacity(self.checkers.len());

        for checker in &self.checkers {
            let start = Instant::now();

            // Use tokio timeout if available, otherwise just run
            let status = tokio::time::timeout(self.config.timeout, checker.check())
                .await
                .unwrap_or(HealthStatus::Unhealthy);

            let duration = start.elapsed();

            let (status, details) = match status {
                HealthStatus::Unhealthy => (
                    HealthStatus::Unhealthy,
                    Some(format!("timeout after {:?}", self.config.timeout)),
                ),
                s => (s, None),
            };

            results.push(HealthCheckResult {
                service: checker.name().to_owned(),
                status,
                duration,
                details,
            });
        }

        results
    }

    /// Compute the aggregate status across all checkers.
    pub async fn overall_status(&self) -> HealthStatus {
        let results = self.check_all().await;
        results
            .iter()
            .fold(HealthStatus::Healthy, |acc, r| acc.worse(r.status))
    }

    /// Produce a full health response.
    pub async fn health_response(&self) -> HealthResponse {
        let checks = self.check_all().await;
        let status = checks
            .iter()
            .fold(HealthStatus::Healthy, |acc, r| acc.worse(r.status));

        HealthResponse { status, checks }
    }
}

impl Default for HealthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockChecker {
        name: String,
        status: HealthStatus,
    }

    impl HealthChecker for MockChecker {
        fn name(&self) -> &str {
            &self.name
        }

        fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
            let status = self.status;
            Box::pin(async move { status })
        }
    }

    #[test]
    fn health_status_worse() {
        assert_eq!(
            HealthStatus::Healthy.worse(HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthStatus::Healthy.worse(HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            HealthStatus::Degraded.worse(HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
        assert_eq!(
            HealthStatus::Healthy.worse(HealthStatus::Healthy),
            HealthStatus::Healthy
        );
    }

    #[test]
    fn health_status_is_up() {
        assert!(HealthStatus::Healthy.is_up());
        assert!(HealthStatus::Degraded.is_up());
        assert!(!HealthStatus::Unhealthy.is_up());
        assert!(!HealthStatus::Unknown.is_up());
    }

    #[tokio::test]
    async fn health_monitor_check_all() {
        let mut monitor = HealthMonitor::new();
        monitor.add_checker(MockChecker {
            name: "db".into(),
            status: HealthStatus::Healthy,
        });
        monitor.add_checker(MockChecker {
            name: "cache".into(),
            status: HealthStatus::Healthy,
        });

        let results = monitor.check_all().await;
        assert_eq!(results.len(), 2);
        assert!(results.iter().all(|r| r.status == HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn health_monitor_overall_status() {
        let mut monitor = HealthMonitor::new();
        monitor.add_checker(MockChecker {
            name: "db".into(),
            status: HealthStatus::Healthy,
        });
        monitor.add_checker(MockChecker {
            name: "cache".into(),
            status: HealthStatus::Unhealthy,
        });

        let status = monitor.overall_status().await;
        assert_eq!(status, HealthStatus::Unhealthy);
    }
}
