use axum::http::StatusCode;
use serde_json::Value;

use super::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn worktree_endpoints_work() {
    let server = setup_test_server().await;

    let created = server
        .post("/api/v1/worktrees")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"feature_slug":"feat/demo","wp_id":"WP01"}))
        .await;
    created.assert_status(StatusCode::CREATED);
    let body: Value = created.json();
    assert_eq!(body["feature_slug"], "feat/demo");
    assert_eq!(body["wp_id"], "WP01");

    let removed = server
        .delete("/api/v1/worktrees")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({"path":"/tmp/worktree"}))
        .await;
    removed.assert_status_ok();
    let body: Value = removed.json();
    assert_eq!(body["message"], "Removed worktree /tmp/worktree");
}
