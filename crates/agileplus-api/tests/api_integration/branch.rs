use axum::http::StatusCode;
use serde_json::Value;

use super::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn branch_endpoints_work() {
    let server = setup_test_server().await;

    let created = server
        .post("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"name":"feat/demo","base":"main"}))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body: Value = created.json();
    assert_eq!(body["message"], "Created branch feat/demo from main");

    let sync = server
        .post("/api/v1/branches/sync")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"source":"main","target":"canary"}))
        .await;
    sync.assert_status_ok();
    let body: Value = sync.json();
    assert_eq!(body["success"], true);
    assert_eq!(body["source"], "main");
    assert_eq!(body["target"], "canary");

    let listed = server
        .get("/api/v1/branches")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    listed.assert_status_ok();
}
