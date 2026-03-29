use axum::http::StatusCode;

use crate::support::{TEST_API_KEY, setup_test_server};

#[tokio::test]
async fn health_no_auth_required() {
    // Traces to: FR-API-005, FR-DOMAIN-014
    // Verify that /health endpoint returns service health without authentication
    let server = setup_test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    let body: serde_json::Value = resp.json();

    // Health endpoint returns "healthy" or "degraded" (not "ok") as of WP11-T070.
    let status = body["status"].as_str().expect("status field present");
    assert!(
        status == "healthy" || status == "degraded",
        "unexpected health status: {status}"
    );

    // Timestamp and services must be present.
    assert!(body["timestamp"].is_string());
    assert!(body["services"].is_object());
}

#[tokio::test]
async fn info_no_auth_required() {
    // Traces to: FR-API-005
    // Verify that /info endpoint is accessible without authentication
    let server = setup_test_server().await;
    let resp = server.get("/info").await;
    resp.assert_status_ok();
}

#[tokio::test]
async fn list_features_requires_auth() {
    // Traces to: FR-API-007
    // Verify that unauthenticated requests are rejected with 401
    let server = setup_test_server().await;
    let resp = server.get("/api/v1/features").await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn list_features_invalid_key_returns_401() {
    // Traces to: FR-API-007
    // Verify that invalid API key is rejected with 401
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", "wrong-key")
        .await;
    resp.assert_status(StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn response_content_type_is_json() {
    // Traces to: FR-API-001
    // Verify that API responses include correct JSON content-type header
    let server = setup_test_server().await;
    let resp = server
        .get("/api/v1/features")
        .add_header("X-API-Key", TEST_API_KEY)
        .await;
    let ct = resp
        .headers()
        .get("content-type")
        .expect("content-type header present")
        .to_str()
        .expect("content-type is valid utf8");
    assert!(
        ct.contains("application/json"),
        "Expected application/json, got: {ct}"
    );
}
