//! T119: Dashboard service control integration test.
//!
//! Verifies persisted toggle and restart behavior of dashboard API endpoints.
//!
//! Traceability: WP19-T119

use agileplus_integration_tests::common::harness::{is_process_compose_installed, TestHarness};

#[cfg(feature = "integration")]
use std::env;

#[cfg(feature = "integration")]
use reqwest::StatusCode;

/// Helper: skip the test if services are unavailable.
#[cfg(feature = "integration")]
macro_rules! require_services {
    () => {
        if !is_process_compose_installed() {
            eprintln!(
                "SKIP: process-compose not installed -- run with --features integration and a live stack to execute this test."
            );
            return Ok(());
        }
    };
}

#[tokio::test]
#[ignore]
#[cfg(feature = "integration")]
async fn dashboard_service_control_integration() -> anyhow::Result<()> {
    require_services!();

    let _ = tracing_subscriber::fmt()
        .with_env_filter("info")
        .try_init();

    let harness = TestHarness::start().await?;
    let client = harness.client();

    // 1) Toggle service to disabled via API; assert persisted
    let toggle_resp = client
        .post(harness.url("/api/dashboard/services/NATS/toggle"))
        .json(&serde_json::json!({ "enabled": false }))
        .send()
        .await?;

    assert_eq!(toggle_resp.status(), StatusCode::OK);
    let toggle_json: serde_json::Value = toggle_resp.json().await?;
    assert_eq!(toggle_json["status"], "ok");
    assert_eq!(toggle_json["service"], "NATS");
    assert_eq!(toggle_json["enabled"], false);

    // 2) Restart service via safe command registry
    // SAFETY: set_var is unsafe in Rust due to potential undefined behavior when
    // used from multiple threads without synchronization. This call is intentionally
    // scoped to a single-threaded test context.
    unsafe { env::set_var("AGILEPLUS_SERVICE_RESTART_CMD", "echo restarted {}") };

    let restart_resp = client
        .post(harness.url("/api/dashboard/services/NATS/restart"))
        .send()
        .await?;

    assert_eq!(restart_resp.status(), StatusCode::OK);
    let restart_json: serde_json::Value = restart_resp.json().await?;
    assert_eq!(restart_json["status"], "ok");
    assert_eq!(restart_json["service"], "NATS");
    assert!(restart_json["stdout"].as_str().unwrap_or_default().contains("restarted NATS"));

    Ok(())
}

// Unit-safe compile test path
#[test]
fn service_control_module_compiles() {
    // still ensures file is included in builds when integration feature is off
    let _ = is_process_compose_installed();
}
