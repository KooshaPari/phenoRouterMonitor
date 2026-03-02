//! Integration tests for the `ci_status` module using wiremock.

use agileplus_agent_review::ci_status::{check_ci_status, CiStatus};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("agileplus-test/0.1")
        .build()
        .unwrap()
}

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/mock_responses/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("fixture {name} not found: {e}"))
}

/// Wire up a mock PR endpoint that returns a fixed head SHA.
async fn mock_pr(server: &MockServer, pr_number: u64, sha: &str) {
    let body = serde_json::json!({ "head": { "sha": sha } });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/pulls/{pr_number}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(server)
        .await;
}

/// Wire up an empty legacy status response.
async fn mock_empty_status(server: &MockServer, sha: &str) {
    let body = serde_json::json!({ "state": "pending", "statuses": [] });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(server)
        .await;
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_ci_all_passing() {
    let server = MockServer::start().await;
    let sha = "deadbeef1234";

    mock_pr(&server, 1, sha).await;

    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("check_runs_passing.json")),
        )
        .mount(&server)
        .await;

    mock_empty_status(&server, sha).await;

    let status = check_ci_status(&client(), &server.uri(), "token", "acme", "repo", 1)
        .await
        .unwrap();

    assert_eq!(status, CiStatus::Passed);
}

#[tokio::test]
async fn test_ci_one_failed() {
    let server = MockServer::start().await;
    let sha = "failsha";

    mock_pr(&server, 1, sha).await;

    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("check_runs_failing.json")),
        )
        .mount(&server)
        .await;

    mock_empty_status(&server, sha).await;

    let status = check_ci_status(&client(), &server.uri(), "token", "acme", "repo", 1)
        .await
        .unwrap();

    assert!(
        matches!(&status, CiStatus::Failed { failed_checks } if failed_checks.contains(&"tests".to_owned())),
        "expected Failed with 'tests', got {status:?}"
    );
}

#[tokio::test]
async fn test_ci_pending() {
    let server = MockServer::start().await;
    let sha = "pendingsha";

    mock_pr(&server, 1, sha).await;

    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("check_runs_pending.json")),
        )
        .mount(&server)
        .await;

    mock_empty_status(&server, sha).await;

    let status = check_ci_status(&client(), &server.uri(), "token", "acme", "repo", 1)
        .await
        .unwrap();

    assert!(
        matches!(&status, CiStatus::Pending { .. }),
        "expected Pending, got {status:?}"
    );
}

#[tokio::test]
async fn test_ci_no_checks_returns_unknown() {
    let server = MockServer::start().await;
    let sha = "emptysha";

    mock_pr(&server, 1, sha).await;

    let empty_runs = serde_json::json!({ "total_count": 0, "check_runs": [] });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&empty_runs))
        .mount(&server)
        .await;

    // Empty legacy status too.
    let empty_status = serde_json::json!({ "state": "pending", "statuses": [] });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&empty_status))
        .mount(&server)
        .await;

    let status = check_ci_status(&client(), &server.uri(), "token", "acme", "repo", 1)
        .await
        .unwrap();

    assert_eq!(status, CiStatus::Unknown);
}

#[tokio::test]
async fn test_ci_combined_checks_and_legacy_status() {
    let server = MockServer::start().await;
    let sha = "combinedsha";

    mock_pr(&server, 1, sha).await;

    // Check runs: lint passes.
    let runs = serde_json::json!({
        "total_count": 1,
        "check_runs": [{
            "id": 7001,
            "name": "lint",
            "status": "completed",
            "conclusion": "success",
            "details_url": null,
            "started_at": null,
            "completed_at": null
        }]
    });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&runs))
        .mount(&server)
        .await;

    // Legacy status: travis also passes.
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/status")))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("commit_status.json")),
        )
        .mount(&server)
        .await;

    let status = check_ci_status(&client(), &server.uri(), "token", "acme", "repo", 1)
        .await
        .unwrap();

    assert_eq!(status, CiStatus::Passed, "both checks and legacy status pass");
}
