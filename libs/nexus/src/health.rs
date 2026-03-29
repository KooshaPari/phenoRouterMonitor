//! Health check module for service monitoring

use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

/// Health status of a service
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum HealthStatus {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Health check is pending
    #[default]
    Pending,
}

/// Health check configuration
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    /// Interval between health checks
    pub interval: Duration,
    /// Timeout for each health check
    pub timeout: Duration,
    /// Number of failures before marking unhealthy
    pub failure_threshold: u32,
    /// Number of successes before marking healthy
    pub success_threshold: u32,
}

impl Default for HealthCheckConfig {
    fn default() -> Self {
        Self {
            interval: Duration::from_secs(30),
            timeout: Duration::from_secs(5),
            failure_threshold: 3,
            success_threshold: 2,
        }
    }
}

/// Health check state for a service
#[derive(Debug, Clone)]
pub struct ServiceHealth {
    /// Service name
    pub service_name: String,
    /// Current health status
    pub status: HealthStatus,
    /// Consecutive failures
    pub consecutive_failures: u32,
    /// Consecutive successes
    pub consecutive_successes: u32,
    /// Last check timestamp
    pub last_check: Option<std::time::Instant>,
}

impl ServiceHealth {
    /// Create new service health tracker
    pub fn new(service_name: String) -> Self {
        Self {
            service_name,
            status: HealthStatus::Pending,
            consecutive_failures: 0,
            consecutive_successes: 0,
            last_check: None,
        }
    }

    /// Record a successful health check
    pub fn record_success(&mut self) {
        self.last_check = Some(std::time::Instant::now());
        self.consecutive_failures = 0;
        self.consecutive_successes += 1;

        if self.consecutive_successes >= 2 {
            self.status = HealthStatus::Healthy;
        }
    }

    /// Record a failed health check
    pub fn record_failure(&mut self) {
        self.last_check = Some(std::time::Instant::now());
        self.consecutive_successes = 0;
        self.consecutive_failures += 1;

        if self.consecutive_failures >= 3 {
            self.status = HealthStatus::Unhealthy;
        }
    }
}

/// Health monitor for managing service health checks
///
/// # Example
///
/// ```ignore
/// # async fn example() {
/// use nexus::{HealthMonitor, HealthStatus};
///
/// let monitor = HealthMonitor::new();
/// monitor.register_service("api".to_string()).await;
///
/// monitor.record_success("api").await;
/// monitor.record_success("api").await;
///
/// let status = monitor.get_health("api").await;
/// assert_eq!(status, Some(HealthStatus::Healthy));
/// # }
/// ```
pub struct HealthMonitor {
    services: Arc<RwLock<Vec<ServiceHealth>>>,
    config: HealthCheckConfig,
}

impl HealthMonitor {
    /// Create a new health monitor with default config
    pub fn new() -> Self {
        Self::with_config(HealthCheckConfig::default())
    }

    /// Create a health monitor with custom config
    pub fn with_config(config: HealthCheckConfig) -> Self {
        Self {
            services: Arc::new(RwLock::new(Vec::new())),
            config,
        }
    }

    /// Register a service for health monitoring
    pub async fn register_service(&self, service_name: String) {
        let mut services = self.services.write().await;
        if !services.iter().any(|s| s.service_name == service_name) {
            services.push(ServiceHealth::new(service_name));
        }
    }

    /// Unregister a service from health monitoring
    pub async fn unregister_service(&self, service_name: &str) {
        let mut services = self.services.write().await;
        services.retain(|s| s.service_name != service_name);
    }

    /// Record a successful health check for a service
    pub async fn record_success(&self, service_name: &str) {
        let mut services = self.services.write().await;
        if let Some(health) = services.iter_mut().find(|s| s.service_name == service_name) {
            health.record_success();
        }
    }

    /// Record a failed health check for a service
    pub async fn record_failure(&self, service_name: &str) {
        let mut services = self.services.write().await;
        if let Some(health) = services.iter_mut().find(|s| s.service_name == service_name) {
            health.record_failure();
        }
    }

    /// Get health status for a service
    pub async fn get_health(&self, service_name: &str) -> Option<HealthStatus> {
        let services = self.services.read().await;
        services.iter().find(|s| s.service_name == service_name).map(|s| s.status.clone())
    }

    /// Get all service health statuses
    pub async fn get_all_health(&self) -> Vec<ServiceHealth> {
        self.services.read().await.clone()
    }

    /// Check if all registered services are healthy
    pub async fn is_healthy(&self) -> bool {
        let services = self.services.read().await;
        services.iter().all(|s| s.status == HealthStatus::Healthy)
    }

    /// Get the health check configuration
    pub fn config(&self) -> &HealthCheckConfig {
        &self.config
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

    #[tokio::test]
    async fn test_health_monitor_register() {
        let monitor = HealthMonitor::new();
        monitor.register_service("test-svc".to_string()).await;
        let health = monitor.get_health("test-svc").await;
        assert_eq!(health, Some(HealthStatus::Pending));
    }

    #[tokio::test]
    async fn test_health_monitor_success() {
        let monitor = HealthMonitor::new();
        monitor.register_service("test-svc".to_string()).await;
        monitor.record_success("test-svc").await;
        monitor.record_success("test-svc").await;
        assert_eq!(monitor.get_health("test-svc").await, Some(HealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_health_monitor_failure() {
        let monitor = HealthMonitor::new();
        monitor.register_service("test-svc".to_string()).await;
        monitor.record_failure("test-svc").await;
        monitor.record_failure("test-svc").await;
        monitor.record_failure("test-svc").await;
        assert_eq!(monitor.get_health("test-svc").await, Some(HealthStatus::Unhealthy));
    }

    #[tokio::test]
    async fn test_health_monitor_unregister() {
        let monitor = HealthMonitor::new();
        monitor.register_service("test-svc".to_string()).await;
        monitor.unregister_service("test-svc").await;
        assert_eq!(monitor.get_health("test-svc").await, None);
    }

    #[tokio::test]
    async fn test_is_healthy() {
        let monitor = HealthMonitor::new();
        monitor.register_service("svc1".to_string()).await;
        monitor.register_service("svc2".to_string()).await;
        monitor.record_success("svc1").await;
        monitor.record_success("svc1").await;
        monitor.record_success("svc2").await;
        monitor.record_success("svc2").await;
        assert!(monitor.is_healthy().await);
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = HealthCheckConfig {
            interval: Duration::from_secs(10),
            timeout: Duration::from_secs(3),
            failure_threshold: 2,
            success_threshold: 1,
        };
        let monitor = HealthMonitor::with_config(config);
        assert_eq!(monitor.config().interval, Duration::from_secs(10));
        assert_eq!(monitor.config().failure_threshold, 2);
    }
}
