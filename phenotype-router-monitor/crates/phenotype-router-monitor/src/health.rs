//! Health check and status tracking for backends.

use serde::{Deserialize, Serialize};

/// Health status of a backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
}

/// Health checker trait for custom health checks.
pub trait HealthChecker: Send + Sync {
    /// Check the health of a backend.
    fn check(&self, backend_url: &str) -> HealthStatus;

    /// Get the checker name.
    fn name(&self) -> &str;
}

/// Simple HTTP-based health checker.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct HttpHealthChecker {
    timeout_ms: u64,
}

#[allow(dead_code)]
impl HttpHealthChecker {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    pub fn default_timeout() -> Self {
        Self::new(5000)
    }
}

impl HealthChecker for HttpHealthChecker {
    fn check(&self, _backend_url: &str) -> HealthStatus {
        // In a real implementation, this would make an HTTP request
        // For now, we return Healthy as a placeholder
        HealthStatus::Healthy
    }

    fn name(&self) -> &str {
        "http"
    }
}

/// TCP-based health checker.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct TcpHealthChecker {
    timeout_ms: u64,
}

#[allow(dead_code)]
impl TcpHealthChecker {
    pub fn new(timeout_ms: u64) -> Self {
        Self { timeout_ms }
    }

    pub fn default_timeout() -> Self {
        Self::new(5000)
    }
}

impl HealthChecker for TcpHealthChecker {
    fn check(&self, _backend_url: &str) -> HealthStatus {
        // In a real implementation, this would attempt a TCP connection
        // For now, we return Healthy as a placeholder
        HealthStatus::Healthy
    }

    fn name(&self) -> &str {
        "tcp"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_equality() {
        assert_eq!(HealthStatus::Healthy, HealthStatus::Healthy);
        assert_ne!(HealthStatus::Healthy, HealthStatus::Unhealthy);
    }

    #[test]
    fn test_http_health_checker_creation() {
        let checker = HttpHealthChecker::new(3000);
        assert_eq!(checker.name(), "http");
        assert_eq!(checker.timeout_ms, 3000);
    }

    #[test]
    fn test_http_health_checker_default() {
        let checker = HttpHealthChecker::default_timeout();
        assert_eq!(checker.timeout_ms, 5000);
    }

    #[test]
    fn test_http_health_checker_check() {
        let checker = HttpHealthChecker::new(5000);
        let status = checker.check("http://localhost:3000");
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_tcp_health_checker_creation() {
        let checker = TcpHealthChecker::new(3000);
        assert_eq!(checker.name(), "tcp");
        assert_eq!(checker.timeout_ms, 3000);
    }

    #[test]
    fn test_tcp_health_checker_default() {
        let checker = TcpHealthChecker::default_timeout();
        assert_eq!(checker.timeout_ms, 5000);
    }

    #[test]
    fn test_tcp_health_checker_check() {
        let checker = TcpHealthChecker::new(5000);
        let status = checker.check("tcp://localhost:3000");
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_health_status_serialization() {
        let status = HealthStatus::Healthy;
        let json = serde_json::to_string(&status).unwrap();
        let deserialized: HealthStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(status, deserialized);
    }

    #[test]
    fn test_health_checker_trait_object() {
        let checker: Box<dyn HealthChecker> = Box::new(HttpHealthChecker::new(5000));
        let status = checker.check("http://localhost:3000");
        assert_eq!(status, HealthStatus::Healthy);
    }

    #[test]
    fn test_all_health_statuses() {
        let statuses = vec![
            HealthStatus::Healthy,
            HealthStatus::Degraded,
            HealthStatus::Unhealthy,
        ];

        assert_eq!(statuses.len(), 3);
        assert!(statuses.contains(&HealthStatus::Healthy));
        assert!(statuses.contains(&HealthStatus::Degraded));
        assert!(statuses.contains(&HealthStatus::Unhealthy));
    }
}
