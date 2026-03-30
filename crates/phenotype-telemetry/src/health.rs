//! Health check integration for telemetry.

use serde::{Deserialize, Serialize};

/// Health status enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// System is healthy and operational.
    Healthy,
    /// System is degraded but operational.
    Degraded,
    /// System is unhealthy and non-operational.
    Unhealthy,
}

/// Health check result with status and optional details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    /// Overall health status.
    pub status: HealthStatus,
    /// Optional message describing the health state.
    pub message: Option<String>,
    /// Timestamp of the health check.
    pub timestamp: String,
}

impl HealthCheckResult {
    /// Create a new healthy result.
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            message: Some("System is operational".to_string()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create a degraded result with a message.
    pub fn degraded(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Degraded,
            message: Some(message.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }

    /// Create an unhealthy result with a message.
    pub fn unhealthy(message: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            timestamp: chrono::Utc::now().to_rfc3339(),
        }
    }
}

/// Telemetry health check provider.
pub struct TelemetryHealth {
    status: std::sync::Arc<std::sync::RwLock<HealthCheckResult>>,
}

impl TelemetryHealth {
    /// Create a new telemetry health checker.
    pub fn new() -> Self {
        Self {
            status: std::sync::Arc::new(std::sync::RwLock::new(HealthCheckResult::healthy())),
        }
    }

    /// Check if telemetry is healthy.
    pub fn is_healthy(&self) -> bool {
        self.status
            .read()
            .ok()
            .map(|s| s.status == HealthStatus::Healthy)
            .unwrap_or(false)
    }

    /// Get the current health status.
    pub fn status(&self) -> Option<HealthCheckResult> {
        self.status.read().ok().map(|s| s.clone())
    }

    /// Update the health status.
    pub fn set_status(&self, status: HealthCheckResult) {
        if let Ok(mut current) = self.status.write() {
            *current = status;
        }
    }

    /// Export health status as JSON.
    pub fn status_json(&self) -> serde_json::Value {
        match self.status.read() {
            Ok(s) => {
                serde_json::json!({
                    "status": s.status,
                    "message": s.message,
                    "timestamp": s.timestamp,
                })
            }
            Err(_) => {
                serde_json::json!({
                    "status": "unhealthy",
                    "message": "Failed to read health status",
                })
            }
        }
    }
}

impl Default for TelemetryHealth {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_status_healthy() {
        let result = HealthCheckResult::healthy();
        assert_eq!(result.status, HealthStatus::Healthy);
        assert!(result.message.is_some());
    }

    #[test]
    fn test_health_status_degraded() {
        let result = HealthCheckResult::degraded("Memory usage high");
        assert_eq!(result.status, HealthStatus::Degraded);
        assert_eq!(
            result.message,
            Some("Memory usage high".to_string())
        );
    }

    #[test]
    fn test_health_status_unhealthy() {
        let result = HealthCheckResult::unhealthy("Database connection lost");
        assert_eq!(result.status, HealthStatus::Unhealthy);
        assert_eq!(
            result.message,
            Some("Database connection lost".to_string())
        );
    }

    #[test]
    fn test_telemetry_health_is_healthy() {
        let health = TelemetryHealth::new();
        assert!(health.is_healthy());
    }

    #[test]
    fn test_telemetry_health_set_degraded() {
        let health = TelemetryHealth::new();
        health.set_status(HealthCheckResult::degraded("Test degradation"));
        assert!(!health.is_healthy());

        let status = health.status().unwrap();
        assert_eq!(status.status, HealthStatus::Degraded);
    }

    #[test]
    fn test_telemetry_health_status_json() {
        let health = TelemetryHealth::new();
        let json = health.status_json();

        assert_eq!(json["status"], "healthy");
        assert!(json["message"].is_string());
        assert!(json["timestamp"].is_string());
    }
}
