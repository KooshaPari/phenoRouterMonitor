//! Phenotype health monitoring - Health status types and traits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HealthError {
    #[error("check failed: {0}")]
    CheckFailed(String),
    #[error("timeout: {0}")]
    Timeout(String),
    #[error("unavailable: {0}")]
    Unavailable(String),
}

pub type HealthResult<T> = Result<T, HealthError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    pub fn is_operational(&self) -> bool {
        matches!(self, HealthStatus::Healthy | HealthStatus::Degraded)
    }
    pub fn is_healthy(&self) -> bool {
        matches!(self, HealthStatus::Healthy)
    }
    pub fn worst(self, other: HealthStatus) -> HealthStatus {
        match (self, other) {
            (HealthStatus::Unhealthy, _) | (_, HealthStatus::Unhealthy) => HealthStatus::Unhealthy,
            (HealthStatus::Degraded, _) | (_, HealthStatus::Degraded) => HealthStatus::Degraded,
            (HealthStatus::Unknown, other) | (other, HealthStatus::Unknown) => other,
            (HealthStatus::Healthy, HealthStatus::Healthy) => HealthStatus::Healthy,
        }
    }
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthStatus::Healthy => write!(f, "healthy"),
            HealthStatus::Degraded => write!(f, "degraded"),
            HealthStatus::Unhealthy => write!(f, "unhealthy"),
            HealthStatus::Unknown => write!(f, "unknown"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckResult {
    pub component: String,
    pub status: HealthStatus,
    pub message: Option<String>,
    pub checked_at: DateTime<Utc>,
    pub latency_ms: Option<u64>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl HealthCheckResult {
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            message: None,
            checked_at: Utc::now(),
            latency_ms: None,
            metadata: HashMap::new(),
        }
    }
    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            checked_at: Utc::now(),
            latency_ms: None,
            metadata: HashMap::new(),
        }
    }
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = Some(ms);
        self
    }
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub components: Vec<HealthCheckResult>,
    pub timestamp: DateTime<Utc>,
    pub version: Option<String>,
}

impl HealthResponse {
    pub fn new(results: Vec<HealthCheckResult>) -> Self {
        let status = results
            .iter()
            .map(|r| r.status)
            .fold(HealthStatus::Unknown, HealthStatus::worst);
        Self {
            status,
            components: results,
            timestamp: Utc::now(),
            version: None,
        }
    }
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            components: Vec::new(),
            timestamp: Utc::now(),
            version: None,
        }
    }
    pub fn unhealthy(msg: impl Into<String>) -> Self {
        Self {
            status: HealthStatus::Unhealthy,
            components: vec![HealthCheckResult::unhealthy("system", msg)],
            timestamp: Utc::now(),
            version: None,
        }
    }
}

pub trait HealthChecker: Send + Sync {
    fn check(&self) -> HealthResult<HealthCheckResult>;
}

#[async_trait::async_trait]
pub trait AsyncHealthChecker: Send + Sync {
    async fn check_async(&self) -> HealthResult<HealthCheckResult>;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_status_operational() {
        assert!(HealthStatus::Healthy.is_operational());
        assert!(HealthStatus::Degraded.is_operational());
        assert!(!HealthStatus::Unhealthy.is_operational());
    }
    #[test]
    fn test_status_worst() {
        assert_eq!(HealthStatus::Healthy.worst(HealthStatus::Healthy), HealthStatus::Healthy);
        assert_eq!(HealthStatus::Healthy.worst(HealthStatus::Degraded), HealthStatus::Degraded);
        assert_eq!(HealthStatus::Degraded.worst(HealthStatus::Unhealthy), HealthStatus::Unhealthy);
    }
    #[test]
    fn test_check_result() {
        let r = HealthCheckResult::healthy("db").with_latency(10);
        assert_eq!(r.component, "db");
        assert_eq!(r.status, HealthStatus::Healthy);
        assert_eq!(r.latency_ms, Some(10));
    }
    #[test]
    fn test_response() {
        let resp = HealthResponse::new(vec![HealthCheckResult::healthy("svc")]);
        assert_eq!(resp.status, HealthStatus::Healthy);
    }
}
