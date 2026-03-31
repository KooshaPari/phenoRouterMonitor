//! Common health checker implementations.

use crate::{HealthChecker, HealthStatus};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

/// Health checker that probes a database connection via a user-supplied closure.
pub struct DatabaseHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    name: String,
    probe: F,
}

impl<F> DatabaseHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    pub fn new(name: impl Into<String>, probe: F) -> Self {
        Self {
            name: name.into(),
            probe,
        }
    }
}

impl<F> HealthChecker for DatabaseHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            if (self.probe)().await {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            }
        })
    }
}

/// Health checker that probes a cache connection via a user-supplied closure.
pub struct CacheHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    name: String,
    probe: F,
}

impl<F> CacheHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    pub fn new(name: impl Into<String>, probe: F) -> Self {
        Self {
            name: name.into(),
            probe,
        }
    }
}

impl<F> HealthChecker for CacheHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            if (self.probe)().await {
                HealthStatus::Healthy
            } else {
                HealthStatus::Degraded
            }
        })
    }
}

/// Health checker that hits an external HTTP-like service via a user-supplied closure.
pub struct ExternalServiceHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    name: String,
    probe: F,
}

impl<F> ExternalServiceHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    pub fn new(name: impl Into<String>, probe: F) -> Self {
        Self {
            name: name.into(),
            probe,
        }
    }
}

impl<F> HealthChecker for ExternalServiceHealthChecker<F>
where
    F: Fn() -> Pin<Box<dyn Future<Output = bool> + Send>> + Send + Sync,
{
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            if (self.probe)().await {
                HealthStatus::Healthy
            } else {
                HealthStatus::Unhealthy
            }
        })
    }
}

/// Health checker that reports memory usage. Returns `Degraded` when usage
/// exceeds the configured threshold percentage (0.0..1.0).
pub struct MemoryHealthChecker {
    threshold: f64,
    /// Returns (used_bytes, total_bytes).
    probe: Arc<dyn Fn() -> (u64, u64) + Send + Sync>,
}

impl MemoryHealthChecker {
    pub fn new(threshold: f64, probe: impl Fn() -> (u64, u64) + Send + Sync + 'static) -> Self {
        Self {
            threshold,
            probe: Arc::new(probe),
        }
    }
}

impl HealthChecker for MemoryHealthChecker {
    fn name(&self) -> &str {
        "memory"
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = HealthStatus> + Send + '_>> {
        let (used, total) = (self.probe)();
        let ratio = if total == 0 {
            0.0
        } else {
            used as f64 / total as f64
        };
        let status = if ratio > self.threshold {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };
        Box::pin(async move { status })
    }
}
