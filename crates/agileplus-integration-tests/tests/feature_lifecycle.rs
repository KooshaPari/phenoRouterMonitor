//! T108: Feature lifecycle end-to-end integration test.
//!
//! Verifies the complete path from API creation through state transitions
//! and event hash-chain integrity.
//!
//! Traceability: WP19-T108

use agileplus_integration_tests::common::{
    fixtures::{feature_create_payload, transition_payload},
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

/// Full feature lifecycle: create → transitions → event integrity.
///
/// Requires a running service stack. Marked `#[ignore]` so it is skipped in
/// normal `cargo test` runs; enable with `cargo test -- --include-ignored`
/// together with `--features integration`.
#[tokio::test]
#[ignore]
#[cfg(feature = "integration")]
async fn feature_lifecycle_integration() -> anyhow::Result<()> {
    require_services!();

    let _ = tracing_subscriber::fmt()
        .with_env_filter("debug")
        .try_init();

    let harness = TestHarness::start().await?;

    // -----------------------------------------------------------------------
    // Step 1: Create feature via API
    // -----------------------------------------------------------------------
    let payload = feature_create_payload("E2E lifecycle feature", "Test lifecycle end-to-end");
    let create_resp = harness
        .client()
        .post(harness.url("/api/features"))
        .json(&payload)
        .send()
        .await?;

    assert_eq!(
        create_resp.status().as_u16(),
        201,
        "feature creation should return 201 Created"
    );

    let feature: serde_json::Value = create_resp.json().await?;
    let feature_id = feature["id"]
        .as_i64()
        .expect("response should contain numeric id");

    // -----------------------------------------------------------------------
    // Step 2: Verify feature readable via GET
    // -----------------------------------------------------------------------
    let get_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}")))
        .send()
        .await?;

    assert_eq!(get_resp.status().as_u16(), 200, "GET feature should succeed");
    let fetched: serde_json::Value = get_resp.json().await?;
    assert_eq!(fetched["friendly_name"], "E2E lifecycle feature");

    // -----------------------------------------------------------------------
    // Step 3: Transition through states
    // -----------------------------------------------------------------------
    let states = ["specified", "researched", "planned", "implementing", "validated", "shipped"];
    for target in states {
        let tr = harness
            .client()
            .post(harness.url(&format!("/api/features/{feature_id}/transition")))
            .json(&transition_payload(target))
            .send()
            .await?;

        assert_eq!(
            tr.status().as_u16(),
            200,
            "transition to '{target}' should return 200"
        );

        let tr_body: serde_json::Value = tr.json().await?;
        assert_eq!(
            tr_body["state"].as_str().unwrap_or(""),
            target,
            "feature state should be '{target}' after transition"
        );
    }

    // -----------------------------------------------------------------------
    // Step 4: Verify events endpoint and hash-chain integrity
    // -----------------------------------------------------------------------
    let events_resp = harness
        .client()
        .get(harness.url(&format!("/api/features/{feature_id}/events")))
        .send()
        .await?;

    assert_eq!(events_resp.status().as_u16(), 200, "events list should succeed");
    let events: Vec<serde_json::Value> = events_resp.json().await?;
    assert!(
        !events.is_empty(),
        "there should be at least one event for the feature"
    );

    // Verify hash chain: each event's prev_hash should match the previous hash.
    let mut prev_hash = "0000000000000000000000000000000000000000000000000000000000000000"
        .to_string();
    for event in &events {
        let event_prev = event["prev_hash"].as_str().unwrap_or("");
        assert_eq!(
            event_prev, prev_hash,
            "hash chain broken at event seq {}",
            event["sequence"]
        );
        prev_hash = event["hash"].as_str().unwrap_or("").to_string();
    }

    Ok(())
}

// ---------------------------------------------------------------------------
// Unit-safe stub tests (always run, no services needed)
// ---------------------------------------------------------------------------

#[test]
fn feature_lifecycle_module_compiles() {
    // Verifying imports and macro expansion compiles correctly.
    let _ = feature_create_payload("test", "desc");
    let _ = transition_payload("specified");
}
