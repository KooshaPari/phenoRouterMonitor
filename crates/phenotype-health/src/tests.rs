use crate::{
    CacheHealthChecker, DatabaseHealthChecker, ExternalServiceHealthChecker, HealthCheckConfig,
    HealthChecker, HealthMonitor, HealthStatus, MemoryHealthChecker,
};
use std::pin::Pin;
use std::time::Duration;

// --- HealthStatus tests ---

#[test]
fn worse_returns_unhealthy_over_anything() {
    assert_eq!(
        HealthStatus::Healthy.worse(HealthStatus::Unhealthy),
        HealthStatus::Unhealthy
    );
    assert_eq!(
        HealthStatus::Unhealthy.worse(HealthStatus::Healthy),
        HealthStatus::Unhealthy
    );
}

#[test]
fn worse_returns_degraded_over_healthy() {
    assert_eq!(
        HealthStatus::Healthy.worse(HealthStatus::Degraded),
        HealthStatus::Degraded
    );
}

#[test]
fn worse_returns_unknown_over_healthy() {
    assert_eq!(
        HealthStatus::Healthy.worse(HealthStatus::Unknown),
        HealthStatus::Unknown
    );
}

#[test]
fn worse_healthy_healthy_is_healthy() {
    assert_eq!(
        HealthStatus::Healthy.worse(HealthStatus::Healthy),
        HealthStatus::Healthy
    );
}

// --- Stub checker for tests ---

struct StubChecker {
    name: &'static str,
    status: HealthStatus,
}

impl HealthChecker for StubChecker {
    fn name(&self) -> &str {
        self.name
    }

    fn check(&self) -> Pin<Box<dyn std::future::Future<Output = HealthStatus> + Send + '_>> {
        let s = self.status;
        Box::pin(async move { s })
    }
}

struct SlowChecker;

impl HealthChecker for SlowChecker {
    fn name(&self) -> &str {
        "slow"
    }

    fn check(&self) -> Pin<Box<dyn std::future::Future<Output = HealthStatus> + Send + '_>> {
        Box::pin(async {
            tokio::time::sleep(Duration::from_secs(10)).await;
            HealthStatus::Healthy
        })
    }
}

// --- HealthMonitor tests ---

#[tokio::test]
async fn monitor_no_checkers_is_healthy() {
    let monitor = HealthMonitor::new();
    assert_eq!(monitor.overall_status().await, HealthStatus::Healthy);
}

#[tokio::test]
async fn monitor_all_healthy() {
    let mut monitor = HealthMonitor::new();
    monitor.add_checker(StubChecker {
        name: "a",
        status: HealthStatus::Healthy,
    });
    monitor.add_checker(StubChecker {
        name: "b",
        status: HealthStatus::Healthy,
    });
    assert_eq!(monitor.overall_status().await, HealthStatus::Healthy);
}

#[tokio::test]
async fn monitor_one_unhealthy() {
    let mut monitor = HealthMonitor::new();
    monitor.add_checker(StubChecker {
        name: "a",
        status: HealthStatus::Healthy,
    });
    monitor.add_checker(StubChecker {
        name: "b",
        status: HealthStatus::Unhealthy,
    });
    assert_eq!(monitor.overall_status().await, HealthStatus::Unhealthy);
}

#[tokio::test]
async fn monitor_timeout_yields_unhealthy() {
    let config = HealthCheckConfig {
        timeout: Duration::from_millis(50),
        ..Default::default()
    };
    let mut monitor = HealthMonitor::with_config(config);
    monitor.add_checker(SlowChecker);
    let results = monitor.check_all().await;
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].status, HealthStatus::Unhealthy);
    assert!(results[0].details.as_ref().unwrap().contains("timeout"));
}

#[tokio::test]
async fn health_response_json_serialization() {
    let mut monitor = HealthMonitor::new();
    monitor.add_checker(StubChecker {
        name: "db",
        status: HealthStatus::Healthy,
    });
    let resp = monitor.health_response().await;
    let json = serde_json::to_string(&resp).unwrap();
    assert!(json.contains("\"status\":\"healthy\""));
    assert!(json.contains("\"service\":\"db\""));
}

// --- Checker implementation tests ---

#[tokio::test]
async fn database_checker_healthy() {
    let checker = DatabaseHealthChecker::new("pg", || Box::pin(async { true }));
    assert_eq!(checker.check().await, HealthStatus::Healthy);
}

#[tokio::test]
async fn database_checker_unhealthy() {
    let checker = DatabaseHealthChecker::new("pg", || Box::pin(async { false }));
    assert_eq!(checker.check().await, HealthStatus::Unhealthy);
}

#[tokio::test]
async fn cache_checker_degraded_on_failure() {
    let checker = CacheHealthChecker::new("redis", || Box::pin(async { false }));
    assert_eq!(checker.check().await, HealthStatus::Degraded);
}

#[tokio::test]
async fn external_service_checker() {
    let checker = ExternalServiceHealthChecker::new("api", || Box::pin(async { true }));
    assert_eq!(checker.check().await, HealthStatus::Healthy);
}

#[tokio::test]
async fn memory_checker_healthy_below_threshold() {
    let checker = MemoryHealthChecker::new(0.8, || (400, 1000));
    assert_eq!(checker.check().await, HealthStatus::Healthy);
}

#[tokio::test]
async fn memory_checker_degraded_above_threshold() {
    let checker = MemoryHealthChecker::new(0.8, || (900, 1000));
    assert_eq!(checker.check().await, HealthStatus::Degraded);
}
