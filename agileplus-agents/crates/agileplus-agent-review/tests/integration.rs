//! End-to-end integration tests for `ReviewAdapter`.
//!
//! These tests spin up a wiremock server and configure `ReviewAdapter` to call
//! it, exercising the full `ReviewPort` trait methods.

use agileplus_agent_dispatch::ReviewPort;
use agileplus_agent_review::{ReviewAdapter, ReviewAdapterConfig};
use std::time::Duration;
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/mock_responses/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("fixture {name} not found: {e}"))
}

fn make_adapter(api_base: &str) -> ReviewAdapter {
    let config = ReviewAdapterConfig::new("fake-token", "acme", "repo")
        .with_api_base(api_base)
        .with_fallback_timeout(Duration::from_millis(100)); // short timeout for tests
    ReviewAdapter::new(config).unwrap()
}

async fn mock_pr_sha(server: &MockServer, pr_number: u64, sha: &str) {
    let body = serde_json::json!({ "head": { "sha": sha } });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/pulls/{pr_number}")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&body))
        .mount(server)
        .await;
}

// ─── await_review ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn integration_await_review_approved() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/7/reviews"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("pr_reviews.json")),
        )
        .mount(&server)
        .await;

    let adapter = make_adapter(&server.uri());
    let outcome = adapter
        .await_review(
            "https://github.com/acme/repo/pull/7",
            Duration::from_secs(5),
        )
        .await
        .unwrap();

    assert_eq!(
        outcome,
        agileplus_agent_dispatch::ReviewOutcome::Approved,
        "adapter should return Approved"
    );
}

#[tokio::test]
async fn integration_await_review_changes_requested() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/8/reviews"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(fixture("pr_reviews_changes_requested.json")),
        )
        .mount(&server)
        .await;

    let adapter = make_adapter(&server.uri());
    let outcome = adapter
        .await_review(
            "https://github.com/acme/repo/pull/8",
            Duration::from_secs(5),
        )
        .await
        .unwrap();

    assert_eq!(outcome, agileplus_agent_dispatch::ReviewOutcome::ChangesRequested);
}

#[tokio::test]
async fn integration_fallback_triggers_in_non_interactive_mode() {
    let server = MockServer::start().await;

    // Always return NotFound so the adapter tries the fallback.
    let empty_reviews: serde_json::Value = serde_json::json!([]);
    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/9/reviews"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&empty_reviews))
        .mount(&server)
        .await;

    // Short fallback timeout so fallback triggers quickly.
    let config = ReviewAdapterConfig::new("fake-token", "acme", "repo")
        .with_api_base(&server.uri())
        .with_fallback_timeout(Duration::from_millis(10));
    let adapter = ReviewAdapter::new(config).unwrap();

    // With a very short poll_timeout, it will time out returning Pending
    // (fallback errors in non-interactive mode are swallowed and polling continues).
    let outcome = adapter
        .await_review(
            "https://github.com/acme/repo/pull/9",
            Duration::from_millis(200),
        )
        .await
        .unwrap();

    // In non-interactive mode the fallback fails silently; poll eventually
    // times out returning Pending.
    assert_eq!(outcome, agileplus_agent_dispatch::ReviewOutcome::Pending);
}

// ─── get_actionable_comments ──────────────────────────────────────────────────

#[tokio::test]
async fn integration_get_actionable_comments() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/10/comments"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("pr_comments_coderabbit.json")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/issues/10/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    let adapter = make_adapter(&server.uri());
    let comments = adapter
        .get_actionable_comments("https://github.com/acme/repo/pull/10")
        .await
        .unwrap();

    // Only actionable comments are returned (first two of three).
    assert_eq!(comments.len(), 2);
    assert_eq!(comments[0].file_path, "src/main.rs");
}

// ─── await_ci ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn integration_await_ci_passing() {
    let server = MockServer::start().await;
    let sha = "cipasssha";

    mock_pr_sha(&server, 11, sha).await;

    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/check-runs")))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("check_runs_passing.json")),
        )
        .mount(&server)
        .await;

    let empty_status = serde_json::json!({ "state": "success", "statuses": [] });
    Mock::given(method("GET"))
        .and(path(format!("/repos/acme/repo/commits/{sha}/status")))
        .respond_with(ResponseTemplate::new(200).set_body_json(&empty_status))
        .mount(&server)
        .await;

    let adapter = make_adapter(&server.uri());
    let ci = adapter
        .await_ci(
            "https://github.com/acme/repo/pull/11",
            Duration::from_secs(5),
        )
        .await
        .unwrap();

    assert_eq!(ci, agileplus_agent_dispatch::CiStatus::Passing);
}
