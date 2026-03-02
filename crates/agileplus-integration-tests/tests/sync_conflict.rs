//! T110: Sync conflict detection and resolution integration test.
//!
//! Verifies that concurrent local and remote modifications to a feature are
//! detected as conflicts, and that the chosen resolution strategy is applied.
//!
//! Traceability: WP19-T110

use agileplus_integration_tests::common::{
    fixtures::{feature_create_payload, plane_webhook_payload},
    harness::{TestHarness, is_process_compose_installed},
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

/// Conflict detection and local-wins resolution.
///
/// Requires a running service stack with the Plane.so mock webhook endpoint.
#[tokio::test]
#[ignore]
#[cfg(feature = "integration")]
async fn sync_conflict_integration() -> anyhow::Result<()> {
    require_services!();

    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();

    let harness = TestHarness::start().await?;

    // -----------------------------------------------------------------------
    // Step 1: Create a feature locally
    // -----------------------------------------------------------------------
    let payload = feature_create_payload("Conflict test feature", "Initial description");
    let create_resp = harness
        .client()
        .post(harness.url("/api/features"))
        .json(&payload)
        .send()
        .await?;

    assert_eq!(create_resp.status().as_u16(), 201);
    let feature: serde_json::Value = create_resp.json().await?;
    let feature_id = feature["id"].as_i64().expect("numeric feature id");

    // -----------------------------------------------------------------------
    // Step 2: Trigger initial sync to Plane.so (mark as synced)
    // -----------------------------------------------------------------------
    let sync_resp = harness
        .client()
        .post(harness.url(&format!("/api/features/{feature_id}/sync")))
        .send()
        .await?;

    assert!(
        sync_resp.status().is_success(),
        "initial sync should succeed"
    );

    // -----------------------------------------------------------------------
    // Step 3: Modify feature locally
    // -----------------------------------------------------------------------
    let local_patch = serde_json::json!({ "title": "Conflict test (modified locally)" });
    let patch_resp = harness
        .client()
        .patch(harness.url(&format!("/api/features/{feature_id}")))
        .json(&local_patch)
        .send()
        .await?;

    assert!(patch_resp.status().is_success(), "local patch should succeed");

    // -----------------------------------------------------------------------
    // Step 4: Simulate a competing Plane.so webhook modification
    // -----------------------------------------------------------------------
    let webhook = plane_webhook_payload(
        feature_id,
        "Conflict test (modified remotely)",
        "Different change from Plane.so",
    );
    let webhook_resp = harness
        .client()
        .post(harness.url("/api/webhooks/plane"))
        .json(&webhook)
        .send()
        .await?;

    assert!(
        webhook_resp.status().is_success(),
        "webhook ingestion should succeed"
    );

    // -----------------------------------------------------------------------
    // Step 5: Verify conflict detected
    // -----------------------------------------------------------------------
    let status_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}/sync-status")))
        .send()
        .await?;

    assert_eq!(status_resp.status().as_u16(), 200);
    let sync_status: serde_json::Value = status_resp.json().await?;

    assert_eq!(
        sync_status["conflict_detected"].as_bool(),
        Some(true),
        "conflict should be detected"
    );
    assert!(
        sync_status["conflict_type"].is_string(),
        "conflict_type should be present"
    );

    // -----------------------------------------------------------------------
    // Step 6: Resolve conflict with local-wins strategy
    // -----------------------------------------------------------------------
    let resolve_resp = harness
        .client()
        .post(harness.url(&format!("/api/features/{feature_id}/resolve-conflict")))
        .json(&serde_json::json!({ "resolution": "local-wins" }))
        .send()
        .await?;

    assert!(
        resolve_resp.status().is_success(),
        "conflict resolution should succeed"
    );

    // -----------------------------------------------------------------------
    // Step 7: Verify resolved feature retains local change
    // -----------------------------------------------------------------------
    let final_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}")))
        .send()
        .await?;

    assert_eq!(final_resp.status().as_u16(), 200);
    let resolved: serde_json::Value = final_resp.json().await?;
    assert_eq!(
        resolved["friendly_name"],
        "Conflict test (modified locally)",
        "local change should win"
    );

    // -----------------------------------------------------------------------
    // Step 8: Verify sync status is clean after resolution
    // -----------------------------------------------------------------------
    let final_status_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}/sync-status")))
        .send()
        .await?;

    let final_status: serde_json::Value = final_status_resp.json().await?;
    assert_eq!(
        final_status["conflict_detected"].as_bool(),
        Some(false),
        "conflict should be cleared after resolution"
    );
    assert_eq!(
        final_status["last_resolved"].as_str(),
        Some("local-wins"),
        "last_resolved should record the strategy"
    );

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit-safe stub tests (always run)
// ---------------------------------------------------------------------------

#[test]
fn sync_conflict_module_compiles() {
    let p = plane_webhook_payload(1, "title", "desc");
    assert_eq!(p["event"], "issue.updated");
}
