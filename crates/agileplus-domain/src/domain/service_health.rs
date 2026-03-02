//! Service health domain types — platform service monitoring.
//!
//! Traceability: FR-016 / WP01-T004

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Health status of a platform service.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unavailable,
}

impl std::fmt::Display for HealthStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Healthy => write!(f, "healthy"),
            Self::Degraded => write!(f, "degraded"),
            Self::Unavailable => write!(f, "unavailable"),
        }
    }
}

/// Health information for a single platform service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceHealth {
    pub service_name: String,
    pub status: HealthStatus,
    pub last_check: DateTime<Utc>,
    pub uptime_seconds: u64,
    pub connection_info: String,
    pub metadata: serde_json::Value,
}

impl ServiceHealth {
    pub fn new(service_name: impl Into<String>, connection_info: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            status: HealthStatus::Unavailable,
            last_check: Utc::now(),
            uptime_seconds: 0,
            connection_info: connection_info.into(),
            metadata: serde_json::Value::Object(serde_json::Map::new()),
        }
    }

    pub fn mark_healthy(&mut self, uptime_seconds: u64) {
        self.status = HealthStatus::Healthy;
        self.uptime_seconds = uptime_seconds;
        self.last_check = Utc::now();
    }

    pub fn mark_degraded(&mut self, reason: &str) {
        self.status = HealthStatus::Degraded;
        self.last_check = Utc::now();
        self.metadata["degraded_reason"] = serde_json::Value::String(reason.to_string());
    }

    pub fn mark_unavailable(&mut self) {
        self.status = HealthStatus::Unavailable;
        self.last_check = Utc::now();
    }
}

/// Aggregated platform health across all services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformStatus {
    pub services: Vec<ServiceHealth>,
    pub overall: HealthStatus,
    pub checked_at: DateTime<Utc>,
}

impl PlatformStatus {
    pub fn from_services(services: Vec<ServiceHealth>) -> Self {
        let overall = if services.iter().all(|s| s.status == HealthStatus::Healthy) {
            HealthStatus::Healthy
        } else if services.iter().any(|s| s.status == HealthStatus::Unavailable) {
            HealthStatus::Unavailable
        } else {
            HealthStatus::Degraded
        };
        Self {
            services,
            overall,
            checked_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_status_display() {
        assert_eq!(HealthStatus::Healthy.to_string(), "healthy");
        assert_eq!(HealthStatus::Degraded.to_string(), "degraded");
        assert_eq!(HealthStatus::Unavailable.to_string(), "unavailable");
    }

    #[test]
    fn service_health_transitions() {
        let mut h = ServiceHealth::new("nats", "localhost:4222");
        assert_eq!(h.status, HealthStatus::Unavailable);

        h.mark_healthy(120);
        assert_eq!(h.status, HealthStatus::Healthy);
        assert_eq!(h.uptime_seconds, 120);

        h.mark_degraded("high latency");
        assert_eq!(h.status, HealthStatus::Degraded);
    }

    #[test]
    fn platform_status_aggregation() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let mut dragonfly = ServiceHealth::new("dragonfly", "localhost:6379");
        dragonfly.mark_healthy(100);

        let status = PlatformStatus::from_services(vec![nats, dragonfly]);
        assert_eq!(status.overall, HealthStatus::Healthy);
    }

    #[test]
    fn platform_status_degraded() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let mut dragonfly = ServiceHealth::new("dragonfly", "localhost:6379");
        dragonfly.mark_degraded("slow");

        let status = PlatformStatus::from_services(vec![nats, dragonfly]);
        assert_eq!(status.overall, HealthStatus::Degraded);
    }

    #[test]
    fn platform_status_unavailable() {
        let mut nats = ServiceHealth::new("nats", "localhost:4222");
        nats.mark_healthy(100);
        let neo4j = ServiceHealth::new("neo4j", "localhost:7687");

        let status = PlatformStatus::from_services(vec![nats, neo4j]);
        assert_eq!(status.overall, HealthStatus::Unavailable);
    }
}
