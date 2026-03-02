//! T111: Service failure and recovery integration test.
//!
//! Verifies that the platform degrades gracefully when the cache (Dragonfly)
//! is unavailable, and recovers correctly when it restarts.
//!
//! Traceability: WP19-T111

use std::time::{Duration, Instant};

use agileplus_integration_tests::common::{
    fixtures::feature_create_payload,
    harness::{TestHarness, is_process_compose_installed, project_root},
};

/// Helper: skip the test if services are unavailable.
macro_rules! require_services {
    () => {
        if !is_process_compose_installed() {
            eprintln!(
                "SKIP: process-compose not installed — \
                 run with --features integration and a live stack to execute this test."
            );
            return Ok(());
        }
    };
}

/// Cache (Dragonfly) failure and recovery.
///
/// Kills the Dragonfly process, verifies degraded-mode responses from
/// the health endpoint, creates a feature (SQLite fallback), restarts
/// Dragonfly via process-compose, and confirms recovery.
#[tokio::test]
#[ignore]
#[cfg(feature = "integration")]
async fn service_failure_recovery_integration() -> anyhow::Result<()> {
    require_services!();

    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();

    let harness = TestHarness::start().await?;

    // -----------------------------------------------------------------------
    // Step 1: Confirm all services are healthy at the start
    // -----------------------------------------------------------------------
    let health_resp = harness
        .client()
        .get(harness.url("/health"))
        .send()
        .await?;

    assert_eq!(health_resp.status().as_u16(), 200);
    let health: serde_json::Value = health_resp.json().await?;
    assert_eq!(
        health["status"].as_str(),
        Some("healthy"),
        "initial state should be healthy"
    );

    // -----------------------------------------------------------------------
    // Step 2: Kill the Dragonfly cache process
    // -----------------------------------------------------------------------
    let kill_output = tokio::process::Command::new("pkill")
        .arg("-f")
        .arg("dragonfly")
        .output()
        .await?;

    // pkill exits 0 if it found and killed the process; 1 if no match.
    // We proceed either way — if dragonfly isn't running the test will still
    // verify the degraded-mode path as long as the cache is unreachable.
    let killed = kill_output.status.success();
    if !killed {
        eprintln!("WARNING: pkill dragonfly returned non-zero; process may not have been running");
    }

    // Brief settle time.
    tokio::time::sleep(Duration::from_secs(2)).await;

    // -----------------------------------------------------------------------
    // Step 3: Verify health reports degraded
    // -----------------------------------------------------------------------
    let degraded_resp = harness
        .client()
        .get(harness.url("/health"))
        .send()
        .await?;

    // The API should still respond (degraded, not down).
    assert!(
        degraded_resp.status().is_success() || degraded_resp.status().as_u16() == 503,
        "health endpoint should respond even when cache is down"
    );
    let degraded: serde_json::Value = degraded_resp.json().await?;
    assert_ne!(
        degraded["status"].as_str(),
        Some("healthy"),
        "status should not be healthy while cache is offline"
    );
    assert_eq!(
        degraded["checks"]["cache"].as_str(),
        Some("unhealthy"),
        "cache check should report unhealthy"
    );

    // -----------------------------------------------------------------------
    // Step 4: Verify core write operations still succeed (SQLite fallback)
    // -----------------------------------------------------------------------
    let payload = feature_create_payload("Created during outage", "SQLite fallback test");
    let create_resp = harness
        .client()
        .post(harness.url("/api/features"))
        .json(&payload)
        .send()
        .await?;

    assert_eq!(
        create_resp.status().as_u16(),
        201,
        "feature creation should succeed even without cache"
    );

    let created: serde_json::Value = create_resp.json().await?;
    let feature_id = created["id"].as_i64().expect("numeric feature id");

    // Verify via a direct DB-backed GET (bypassing cache).
    let get_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}")))
        .send()
        .await?;

    assert_eq!(
        get_resp.status().as_u16(),
        200,
        "feature should be readable from SQLite during outage"
    );

    // -----------------------------------------------------------------------
    // Step 5: Restart Dragonfly via process-compose
    // -----------------------------------------------------------------------
    let restart_output = tokio::process::Command::new("process-compose")
        .arg("restart")
        .arg("dragonfly")
        .current_dir(project_root())
        .output()
        .await?;

    assert!(
        restart_output.status.success(),
        "process-compose restart dragonfly should succeed"
    );

    // -----------------------------------------------------------------------
    // Step 6: Wait for service to recover
    // -----------------------------------------------------------------------
    let start = Instant::now();
    let recovery_timeout = Duration::from_secs(30);
    let mut recovered = false;

    loop {
        tokio::time::sleep(Duration::from_millis(500)).await;

        if let Ok(r) = harness.client().get(harness.url("/health")).send().await {
            if let Ok(body) = r.json::<serde_json::Value>().await {
                if body["status"].as_str() == Some("healthy") {
                    recovered = true;
                    break;
                }
            }
        }

        if start.elapsed() > recovery_timeout {
            break;
        }
    }

    assert!(recovered, "service should recover to healthy within 30 seconds");

    // -----------------------------------------------------------------------
    // Step 7: Verify cache is warm again (feature readable quickly)
    // -----------------------------------------------------------------------
    let cached_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}")))
        .send()
        .await?;

    assert_eq!(
        cached_resp.status().as_u16(),
        200,
        "feature should be accessible after cache recovery"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit-safe stub tests (always run)
// ---------------------------------------------------------------------------

#[test]
fn service_failure_module_compiles() {
    // Verify key imports and types resolve at compile time.
    let _ = project_root();
    let _ = feature_create_payload("outage-test", "payload check");
}
