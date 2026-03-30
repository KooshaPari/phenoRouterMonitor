//! Phenotype health monitoring - Health status types and traits.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
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
}

impl HealthCheckResult {
    pub fn healthy(component: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Healthy,
            message: None,
            checked_at: Utc::now(),
            latency_ms: None,
        }
    }
    pub fn unhealthy(component: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            component: component.into(),
            status: HealthStatus::Unhealthy,
            message: Some(message.into()),
            checked_at: Utc::now(),
            latency_ms: None,
        }
    }
    pub fn with_latency(mut self, ms: u64) -> Self {
        self.latency_ms = Some(ms);
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
}

pub trait HealthChecker: Send + Sync {
    fn check(&self) -> HealthResult<HealthCheckResult>;
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
        assert_eq!(
            HealthStatus::Healthy.worst(HealthStatus::Healthy),
            HealthStatus::Healthy
        );
        assert_eq!(
            HealthStatus::Healthy.worst(HealthStatus::Degraded),
            HealthStatus::Degraded
        );
        assert_eq!(
            HealthStatus::Degraded.worst(HealthStatus::Unhealthy),
            HealthStatus::Unhealthy
        );
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

    // FR-PHENO-HLT-001: Display trait
    #[test]
    fn test_health_status_display() {
        assert_eq!(format!("{}", HealthStatus::Healthy), "healthy");
        assert_eq!(format!("{}", HealthStatus::Degraded), "degraded");
        assert_eq!(format!("{}", HealthStatus::Unhealthy), "unhealthy");
        assert_eq!(format!("{}", HealthStatus::Unknown), "unknown");
    }

    // FR-PHENO-HLT-002: is_healthy
    #[test]
    fn test_health_status_is_healthy() {
        assert!(HealthStatus::Healthy.is_healthy());
        assert!(!HealthStatus::Degraded.is_healthy());
    }

    // FR-PHENO-HLT-003: worst Unhealthy dominates
    #[test]
    fn test_health_status_worst_unhealthy_dominates() {
        assert_eq!(HealthStatus::Unhealthy.worst(HealthStatus::Healthy), HealthStatus::Unhealthy);
    }

    // FR-PHENO-HLT-004: worst Degraded precedence
    #[test]
    fn test_health_status_worst_degraded_precedence() {
        assert_eq!(HealthStatus::Healthy.worst(HealthStatus::Degraded), HealthStatus::Degraded);
    }

    // FR-PHENO-HLT-005: worst Unknown absorption
    #[test]
    fn test_health_status_worst_unknown_absorption() {
        assert_eq!(HealthStatus::Unknown.worst(HealthStatus::Healthy), HealthStatus::Healthy);
    }

    // FR-PHENO-HLT-006: unhealthy factory
    #[test]
    fn test_check_result_unhealthy_factory() {
        let result = HealthCheckResult::unhealthy("svc", "error");
        assert_eq!(result.status, HealthStatus::Unhealthy);
    }

    // FR-PHENO-HLT-007: timestamp
    #[test]
    fn test_check_result_timestamp() {
        let result = HealthCheckResult::healthy("svc");
        assert!(result.checked_at <= Utc::now());
    }

    // FR-PHENO-HLT-008: all healthy
    #[test]
    fn test_response_all_healthy() {
        let results = vec![
            HealthCheckResult::healthy("a"),
            HealthCheckResult::healthy("b"),
        ];
        let resp = HealthResponse::new(results);
        assert_eq!(resp.status, HealthStatus::Healthy);
    }

    // FR-PHENO-HLT-009: with unhealthy
    #[test]
    fn test_response_with_unhealthy() {
        let results = vec![
            HealthCheckResult::healthy("a"),
            HealthCheckResult::unhealthy("b", "error"),
        ];
        let resp = HealthResponse::new(results);
        assert_eq!(resp.status, HealthStatus::Unhealthy);
    }

    // FR-PHENO-HLT-010: response timestamp
    #[test]
    fn test_response_timestamp() {
        let resp = HealthResponse::healthy();
        assert!(resp.timestamp <= Utc::now());
    }

    // FR-PHENO-HLT-011: Send+Sync
    #[test]
    fn test_health_checker_send_sync() {
        fn check<T: Send + Sync>() {}
        struct M;
        impl HealthChecker for M {
            fn check(&self) -> HealthResult<HealthCheckResult> {
                Ok(HealthCheckResult::healthy("m"))
            }
        }
        check::<M>();
    }

    // FR-PHENO-HLT-012: mock ok
    #[test]
    fn test_mock_checker_ok() {
        struct M;
        impl HealthChecker for M {
            fn check(&self) -> HealthResult<HealthCheckResult> {
                Ok(HealthCheckResult::healthy("m").with_latency(5))
            }
        }
        let result = M.check().unwrap();
        assert_eq!(result.latency_ms, Some(5));
    }

    // FR-PHENO-HLT-013: mock error
    #[test]
    fn test_mock_checker_err() {
        struct M;
        impl HealthChecker for M {
            fn check(&self) -> HealthResult<HealthCheckResult> {
                Err(HealthError::Unavailable("x".to_string()))
            }
        }
        assert!(M.check().is_err());
    }

    // FR-PHENO-HLT-014: CheckFailed
    #[test]
    fn test_health_error_check_failed() {
        let err = HealthError::CheckFailed("msg".to_string());
        assert_eq!(format!("{}", err), "check failed: msg");
    }

    // FR-PHENO-HLT-015: Timeout
    #[test]
    fn test_health_error_timeout() {
        let err = HealthError::Timeout("msg".to_string());
        assert_eq!(format!("{}", err), "timeout: msg");
    }

    // FR-PHENO-HLT-016: Unavailable
    #[test]
    fn test_health_error_unavailable() {
        let err = HealthError::Unavailable("msg".to_string());
        assert_eq!(format!("{}", err), "unavailable: msg");
    }

    // FR-PHENO-HLT-017: CheckResult serde
    #[test]
    fn test_check_result_serde() {
        let r = HealthCheckResult::healthy("api").with_latency(25);
        let json = serde_json::to_string(&r).unwrap();
        let d: HealthCheckResult = serde_json::from_str(&json).unwrap();
        assert_eq!(d.latency_ms, Some(25));
    }

    // FR-PHENO-HLT-018: Response serde
    #[test]
    fn test_response_serde() {
        let results = vec![HealthCheckResult::healthy("db")];
        let resp = HealthResponse::new(results);
        let json = serde_json::to_string(&resp).unwrap();
        let d: HealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(d.components.len(), 1);
    }

    // FR-PHENO-HLT-019: Status serde healthy
    #[test]
    fn test_status_serde_healthy() {
        let s = HealthStatus::Healthy;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"healthy\"");
    }

    // FR-PHENO-HLT-020: Status serde degraded
    #[test]
    fn test_status_serde_degraded() {
        let s = HealthStatus::Degraded;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"degraded\"");
    }

    // FR-PHENO-HLT-021: Status serde unhealthy
    #[test]
    fn test_status_serde_unhealthy() {
        let s = HealthStatus::Unhealthy;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"unhealthy\"");
    }

    // FR-PHENO-HLT-022: Status serde unknown
    #[test]
    fn test_status_serde_unknown() {
        let s = HealthStatus::Unknown;
        let json = serde_json::to_string(&s).unwrap();
        assert_eq!(json, "\"unknown\"");
    }

    // FR-PHENO-HLT-023: Empty unknown
    #[test]
    fn test_response_empty_unknown() {
        let resp = HealthResponse::new(vec![]);
        assert_eq!(resp.status, HealthStatus::Unknown);
    }

    // FR-PHENO-HLT-024: Message preservation
    #[test]
    fn test_check_result_message() {
        let result = HealthCheckResult::unhealthy("disk", "95% usage");
        assert_eq!(result.message, Some("95% usage".to_string()));
    }

    // FR-PHENO-HLT-025: Preserves order
    #[test]
    fn test_response_preserves_order() {
        let results = vec![
            HealthCheckResult::healthy("a"),
            HealthCheckResult::healthy("b"),
            HealthCheckResult::healthy("c"),
        ];
        let resp = HealthResponse::new(results);
        assert_eq!(resp.components[0].component, "a");
        assert_eq!(resp.components[1].component, "b");
        assert_eq!(resp.components[2].component, "c");
    }

    // FR-PHENO-HLT-026: Chaining
    #[test]
    fn test_check_result_chaining() {
        let result = HealthCheckResult::healthy("db").with_latency(100);
        assert_eq!(result.latency_ms, Some(100));
    }

    // FR-PHENO-HLT-027: Into<String>
    #[test]
    fn test_check_result_into_string() {
        let r1 = HealthCheckResult::healthy("static");
        assert_eq!(r1.component, "static");
        let r2 = HealthCheckResult::healthy("owned".to_string());
        assert_eq!(r2.component, "owned");
    }

    // FR-PHENO-HLT-028: Version
    #[test]
    fn test_response_version() {
        let mut resp = HealthResponse::healthy();
        resp.version = Some("1.0.0".to_string());
        assert_eq!(resp.version, Some("1.0.0".to_string()));
    }

    // FR-PHENO-HLT-029: Multiple with latency
    #[test]
    fn test_multiple_results_with_latency() {
        let results = vec![
            HealthCheckResult::healthy("a").with_latency(10),
            HealthCheckResult::healthy("b").with_latency(20),
        ];
        let resp = HealthResponse::new(results);
        assert_eq!(resp.components[0].latency_ms, Some(10));
        assert_eq!(resp.components[1].latency_ms, Some(20));
    }

    // FR-PHENO-HLT-030: Component count
    #[test]
    fn test_response_component_count() {
        let results = vec![
            HealthCheckResult::healthy("a"),
            HealthCheckResult::healthy("b"),
            HealthCheckResult::unhealthy("c", "err"),
            HealthCheckResult::healthy("d"),
        ];
        let resp = HealthResponse::new(results);
        assert_eq!(resp.components.len(), 4);
    }
}
