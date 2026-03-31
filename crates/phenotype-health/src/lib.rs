//! # Phenotype Health
//!
//! Shared health check abstraction for Phenotype services.
//!
//! Provides a unified `HealthChecker` trait, a `HealthMonitor` for managing
//! multiple checks, and common checker implementations (database, cache,
//! external service, memory).

mod checkers;
#[cfg(test)]
mod tests;

pub use checkers::{
    CacheHealthChecker, DatabaseHealthChecker, ExternalServiceHealthChecker, MemoryHealthChecker,
};

use serde::Serialize;
use std::future::Future;
use std::pin::Pin;
use std::time::{Duration, Instant};

/// Overall status of a health check.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

impl HealthStatus {
    /// Returns the worse of two statuses (Unhealthy > Degraded > Unknown > Healthy).
    #[must_use]
    pub fn worse(self, other: Self) -> Self {
        use HealthStatus::*;
        match (self, other) {
            (Unhealthy, _) | (_, Unhealthy) => Unhealthy,
            (Degraded, _) | (_, Degraded) => Degraded,
            (Unknown, _) | (_, Unknown) => Unknown,
            _ => Healthy,
        }
    }
}

/// A single health check probe. Implement this for each dependency your service
/// needs to verify (database, cache, external API, etc.).
pub trait HealthChecker: Send + Sync {
    /// Human-readable name for this checker (e.g. "postgres", "redis").
    fn name(&self) -> &str;

    /// Run the check and return the status.
    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>>;
}

/// Result of a single health check execution.
#[derive(Debug, Clone, Serialize)]
pub struct HealthCheckResult {
    pub service: String,
    pub status: HealthStatus,
    #[serde(serialize_with = "serialize_duration_ms")]
    pub duration: Duration,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

fn serialize_duration_ms<S: serde::Serializer>(d: &Duration, s: S) -> Result<S::Ok, S::Error> {
    s.serialize_u64(d.as_millis() as u64)
}

/// Configuration for health checking behavior.
#[derive(Debug, Clone)]
pub struct HealthCheckConfig {
    pub enabled: bool,
    pub interval: Duration,
    pub timeout: Duration,
    pub success_threshold: u32,
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

/// Aggregate health response suitable for JSON serialization on `/health`.
#[derive(Debug, Serialize)]
pub struct HealthResponse {
    pub status: HealthStatus,
    pub checks: Vec<HealthCheckResult>,
}

/// Manages multiple `HealthChecker` instances, runs them with timeouts, and
/// aggregates results.
pub struct HealthMonitor {
    checkers: Vec<Box<dyn HealthChecker>>,
    config: HealthCheckConfig,
}

impl HealthMonitor {
    #[must_use]
    pub fn new() -> Self {
        Self {
            checkers: Vec::new(),
            config: HealthCheckConfig::default(),
        }
    }

    #[must_use]
    pub fn with_config(config: HealthCheckConfig) -> Self {
        Self {
            checkers: Vec::new(),
            config,
        }
    }

    pub fn add_checker(&mut self, checker: impl HealthChecker + 'static) {
        self.checkers.push(Box::new(checker));
    }

    /// Run all registered checkers and return individual results.
    pub async fn check_all(&self) -> Vec<HealthCheckResult> {
        let mut results = Vec::with_capacity(self.checkers.len());
        for checker in &self.checkers {
            let start = Instant::now();
            let status = tokio::time::timeout(self.config.timeout, checker.check()).await;
            let duration = start.elapsed();

            let (status, details) = match status {
                Ok(s) => (s, None),
                Err(_) => (
                    HealthStatus::Unhealthy,
                    Some(format!("timeout after {}ms", self.config.timeout.as_millis())),
                ),
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

    /// Produce a full `HealthResponse` for `/health` endpoints.
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
