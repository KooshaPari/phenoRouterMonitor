use crate::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn get_audit_trail() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/audit")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("audit trail response should be an array");
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["actor"], "system");
}

#[tokio::test]
async fn verify_audit_chain_valid() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/audit/verify")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["chain_valid"], true);
    assert_eq!(body["entries_verified"], 2);
}

#[tokio::test]
async fn get_governance() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature/governance")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["version"], 1);
    assert_eq!(body["feature_id"], 1);
}

#[tokio::test]
async fn trigger_validate() {
    let server = setup_test_server().await;
    let resp = server
        .post("/api/v1/features/test-feature/validate")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["feature_slug"], "test-feature");
    assert_eq!(body["compliant"], true); // no rules -> all satisfied
}
