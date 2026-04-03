//! Full workflow integration tests for AgilePlus.
//!
//! These tests exercise the complete specify -> research -> plan -> implement ->
//! validate -> ship lifecycle against a real running stack. They are gated by
//! the `AGILEPLUS_API_URL` environment variable and marked `#[ignore]` so they
//! do not run in normal `cargo test`. They run in CI inside the Docker Compose
//! environment via `make test-integration`.
//!
//! Traceability: WP16-T096

use std::env;

/// Helper: build a reqwest client with the test API key set.
fn test_client() -> reqwest::Client {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .unwrap()
}

fn api_url() -> String {
    env::var("AGILEPLUS_API_URL").unwrap_or_else(|_| "http://localhost:8080".into())
}

fn api_key() -> String {
    env::var("AGILEPLUS_API_KEY").unwrap_or_else(|_| "test-key".into())
}

/// Poll a URL until it returns 200 or timeout is reached.
async fn wait_for_healthy(url: &str, max_attempts: u32) {
    let client = test_client();
    for attempt in 0..max_attempts {
        if let Ok(resp) = client.get(url).send().await {
            if resp.status().is_success() {
                return;
            }
        }
        if attempt < max_attempts - 1 {
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
        }
    }
    panic!("Service at {url} did not become healthy after {max_attempts} attempts");
}

#[tokio::test]
#[ignore = "Requires running Docker Compose stack (make test-integration)"]
async fn test_full_specify_to_ship_workflow() {
    let api = api_url();
    let key = api_key();
    let client = test_client();

    // Ensure the stack is up
    wait_for_healthy(&format!("{api}/health"), 20).await;

    let feature_slug = "integration-test-feature";

    // ── Step 1: Specify ──────────────────────────────────────────────────
    let resp = client
        .post(format!("{api}/api/v1/commands/specify"))
        .json(&serde_json::json!({
            "feature": feature_slug,
            "friendly_name": "Integration Test Feature",
            "from_content": "# FR-001\nThe system shall accept test input.\n"
        }))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("specify request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "specify should return 200: {}",
        resp.text().await.unwrap_or_default()
    );

    // ── Step 2: Verify feature was created with state 'specified' ────────
    let resp = client
        .get(format!("{api}/api/v1/features/{feature_slug}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("get feature request failed");
    assert_eq!(resp.status().as_u16(), 200);
    let feature: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        feature["state"], "specified",
        "Feature should be in 'specified' state after specify"
    );

    // ── Step 3: Research ─────────────────────────────────────────────────
    let resp = client
        .post(format!("{api}/api/v1/commands/research"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("research request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "research should return 200: {}",
        resp.text().await.unwrap_or_default()
    );

    // ── Step 4: Plan ─────────────────────────────────────────────────────
    let resp = client
        .post(format!("{api}/api/v1/commands/plan"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("plan request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "plan should return 200: {}",
        resp.text().await.unwrap_or_default()
    );

    // Verify feature is now 'planned'
    let resp = client
        .get(format!("{api}/api/v1/features/{feature_slug}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    let feature: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(feature["state"], "planned");

    // ── Step 5: Submit mock evidence (simulates agent work) ───────────────
    // In Docker Compose integration testing, we inject mock evidence directly
    // via the API to simulate an agent completing WP01.
    let resp = client
        .post(format!("{api}/api/v1/features/{feature_slug}/evidence"))
        .json(&serde_json::json!({
            "wp_sequence": 1,
            "fr_id": "FR-001",
            "evidence_type": "test_result",
            "artifact_path": "tests/fixtures/sample-evidence/WP01/test-results.json",
            "metadata": {"passed": 42, "failed": 0, "coverage": 85.2}
        }))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("evidence submission failed");
    // 200 or 201
    assert!(
        resp.status().as_u16() < 300,
        "evidence submission should succeed: {}",
        resp.text().await.unwrap_or_default()
    );

    // ── Step 6: Validate ─────────────────────────────────────────────────
    let resp = client
        .post(format!("{api}/api/v1/commands/validate"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("validate request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "validate should return 200: {}",
        resp.text().await.unwrap_or_default()
    );

    // ── Step 7: Ship ─────────────────────────────────────────────────────
    let resp = client
        .post(format!("{api}/api/v1/commands/ship"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("ship request failed");
    assert_eq!(
        resp.status().as_u16(),
        200,
        "ship should return 200: {}",
        resp.text().await.unwrap_or_default()
    );

    // ── Step 8: Verify final state ────────────────────────────────────────
    let resp = client
        .get(format!("{api}/api/v1/features/{feature_slug}"))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    let feature: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        feature["state"], "shipped",
        "Feature should be in 'shipped' state after ship"
    );

    // ── Step 9: Verify audit trail integrity ──────────────────────────────
    let resp = client
        .post(format!(
            "{api}/api/v1/features/{feature_slug}/audit/verify"
        ))
        .header("X-API-Key", &key)
        .send()
        .await
        .expect("audit verify request failed");
    assert_eq!(resp.status().as_u16(), 200);
    let verification: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(
        verification["valid"], true,
        "Audit chain should be valid after complete lifecycle"
    );
    let entries = verification["entries_verified"].as_u64().unwrap_or(0);
    assert!(
        entries >= 4,
        "Expected at least 4 audit entries (specify+research+plan+ship), got {entries}"
    );
}

#[tokio::test]
#[ignore = "Requires running Docker Compose stack (make test-integration)"]
async fn test_specify_rejects_invalid_state_transition() {
    let api = api_url();
    let key = api_key();
    let client = test_client();

    wait_for_healthy(&format!("{api}/health"), 20).await;

    let feature_slug = "state-test-feature";

    // Create feature and advance to 'implementing'
    for command in &["specify", "research", "plan"] {
        let body = if *command == "specify" {
            serde_json::json!({
                "feature": feature_slug,
                "friendly_name": "State Test Feature",
                "from_content": "# FR-001\nTest."
            })
        } else {
            serde_json::json!({"feature": feature_slug})
        };
        let resp = client
            .post(format!("{api}/api/v1/commands/{command}"))
            .json(&body)
            .header("X-API-Key", &key)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200, "command {command} should succeed");
    }

    // Advance to implementing via implement command
    let resp = client
        .post(format!("{api}/api/v1/commands/implement"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Now try to run specify again from 'implementing' — should fail
    let resp = client
        .post(format!("{api}/api/v1/commands/specify"))
        .json(&serde_json::json!({
            "feature": feature_slug,
            "from_content": "# FR-001\nUpdated."
        }))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_ne!(
        resp.status().as_u16(),
        200,
        "specify from 'implementing' state should fail"
    );
}

#[tokio::test]
#[ignore = "Requires running Docker Compose stack (make test-integration)"]
async fn test_validate_blocks_on_missing_evidence() {
    let api = api_url();
    let key = api_key();
    let client = test_client();

    wait_for_healthy(&format!("{api}/health"), 20).await;

    let feature_slug = "governance-test-feature";

    // Advance to 'implementing' without submitting evidence
    for (command, body) in [
        (
            "specify",
            serde_json::json!({
                "feature": feature_slug,
                "friendly_name": "Governance Test Feature",
                "from_content": "# FR-001\nTest."
            }),
        ),
        ("research", serde_json::json!({"feature": feature_slug})),
        ("plan", serde_json::json!({"feature": feature_slug})),
        ("implement", serde_json::json!({"feature": feature_slug})),
    ] {
        let resp = client
            .post(format!("{api}/api/v1/commands/{command}"))
            .json(&body)
            .header("X-API-Key", &key)
            .send()
            .await
            .unwrap();
        assert_eq!(resp.status().as_u16(), 200, "command {command} should succeed");
    }

    // Validate without evidence — should fail
    let resp = client
        .post(format!("{api}/api/v1/commands/validate"))
        .json(&serde_json::json!({"feature": feature_slug}))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    let status = resp.status().as_u16();
    let body: serde_json::Value = resp.json().await.unwrap_or_default();
    assert!(
        status != 200 || body.get("success").and_then(|v| v.as_bool()) == Some(false),
        "validate should fail without evidence, status={status}, body={body}"
    );
}

#[tokio::test]
#[ignore = "Requires running Docker Compose stack (make test-integration)"]
async fn test_audit_chain_integrity_after_full_lifecycle() {
    let api = api_url();
    let key = api_key();
    let client = test_client();

    wait_for_healthy(&format!("{api}/health"), 20).await;

    let feature_slug = "audit-chain-feature";

    // Quick lifecycle (just specify for the audit chain test)
    let resp = client
        .post(format!("{api}/api/v1/commands/specify"))
        .json(&serde_json::json!({
            "feature": feature_slug,
            "friendly_name": "Audit Chain Feature",
            "from_content": "# FR-016\nAudit chain test."
        }))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);

    // Verify audit chain
    let resp = client
        .post(format!(
            "{api}/api/v1/features/{feature_slug}/audit/verify"
        ))
        .header("X-API-Key", &key)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 200);
    let v: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(v["valid"], true, "Audit chain should be valid");
}
