use crate::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn list_features_with_valid_key() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    let arr = body
        .as_array()
        .expect("features response should be an array");
    assert!(!arr.is_empty());
    assert_eq!(arr[0]["slug"], "test-feature");
}

#[tokio::test]
async fn get_feature_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["slug"], "test-feature");
    assert_eq!(body["name"], "Test Feature");
}

#[tokio::test]
async fn get_feature_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features/nonexistent")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_feature_persists_mutations() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "title": "Renamed Feature",
            "target_branch": "release/stable"
        }))
        .await;
    resp.assert_status_ok();

    let reread = server
        .get("/api/v1/features/test-feature")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    let body: serde_json::Value = reread.json();
    assert_eq!(body["name"], "Renamed Feature");
    assert_eq!(body["target_branch"], "release/stable");
}

#[tokio::test]
async fn get_work_package_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();
    assert_eq!(body["id"], 1);
    assert_eq!(body["title"], "WP01");
}

#[tokio::test]
async fn get_work_package_not_found() {
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/work-packages/999")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    resp.assert_status(axum::http::StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn patch_work_package_persists_mutations() {
    let server = setup_test_server().await;
    let resp = server
        .patch("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .json(&serde_json::json!({
            "title": "Updated WP",
            "acceptance_criteria": "Updated criteria",
            "pr_url": "https://example.com/pr/42"
        }))
        .await;
    resp.assert_status_ok();

    let reread = server
        .get("/api/v1/work-packages/1")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    reread.assert_status_ok();
    let body: serde_json::Value = reread.json();
    assert_eq!(body["title"], "Updated WP");
    assert_eq!(body["acceptance_criteria"], "Updated criteria");
    assert_eq!(body["pr_url"], "https://example.com/pr/42");
}
