//! Integration tests for the `coderabbit` module using wiremock.

use agileplus_agent_review::coderabbit::{fetch_review_comments, parse_review_status, ReviewStatus};
use wiremock::matchers::{method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .user_agent("agileplus-test/0.1")
        .build()
        .unwrap()
}

/// Load a JSON fixture from the `mock_responses` directory.
fn fixture(name: &str) -> String {
    std::fs::read_to_string(format!(
        "{}/tests/mock_responses/{name}",
        env!("CARGO_MANIFEST_DIR")
    ))
    .unwrap_or_else(|e| panic!("fixture {name} not found: {e}"))
}

// ─── fetch_review_comments ────────────────────────────────────────────────────

#[tokio::test]
async fn test_fetch_comments_coderabbit_only() {
    let server = MockServer::start().await;

    // Inline PR review comments — Coderabbit + one non-bot comment filtered below.
    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("pr_comments_coderabbit.json")),
        )
        .mount(&server)
        .await;

    // Top-level issue comments — empty for this test.
    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    let comments = fetch_review_comments(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    // All 3 comments are from the bot.
    assert_eq!(comments.len(), 3);

    // First two are actionable (suggestion block and warning prefix).
    assert!(comments[0].is_actionable, "suggestion block should be actionable");
    assert!(comments[1].is_actionable, "warning prefix should be actionable");
    // Third is praise — informational.
    assert!(!comments[2].is_actionable, "praise should not be actionable");
}

#[tokio::test]
async fn test_fetch_comments_mixed_filters_non_bot() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("pr_comments_mixed.json")),
        )
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    let comments = fetch_review_comments(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    // Only the one coderabbit comment should survive the filter.
    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].path, Some("src/main.rs".to_owned()));
}

#[tokio::test]
async fn test_fetch_comments_empty_pr() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    let comments = fetch_review_comments(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    assert!(comments.is_empty());
}

#[tokio::test]
async fn test_fetch_comments_pagination() {
    let server = MockServer::start().await;

    let page1 = serde_json::json!([{
        "id": 9001,
        "body": "```suggestion\nlet x = 1;\n```",
        "path": "src/a.rs",
        "line": 1,
        "user": { "login": "coderabbitai[bot]" },
        "created_at": "2026-03-01T10:00:00Z"
    }]);

    let page2 = serde_json::json!([{
        "id": 9002,
        "body": "warning: another issue",
        "path": "src/b.rs",
        "line": 5,
        "user": { "login": "coderabbitai[bot]" },
        "created_at": "2026-03-01T10:01:00Z"
    }]);

    // Page 2 URL that the Link header will point to.
    let page2_url = format!("{}/repos/acme/repo/pulls/1/comments?per_page=100&page=2", server.uri());
    let link_header = format!(r#"<{page2_url}>; rel="next""#);

    // First request (with per_page=100 query param).
    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .and(wiremock::matchers::query_param("per_page", "100"))
        .respond_with(
            ResponseTemplate::new(200)
                .append_header("link", link_header.as_str())
                .set_body_json(&page1),
        )
        .up_to_n_times(1)
        .mount(&server)
        .await;

    // Second request (full URL from Link header, matched by page=2 param).
    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .and(wiremock::matchers::query_param("page", "2"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&page2))
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/issues/1/comments"))
        .respond_with(ResponseTemplate::new(200).set_body_string("[]"))
        .mount(&server)
        .await;

    let comments = fetch_review_comments(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    assert_eq!(comments.len(), 2, "should collect both pages");
    assert_eq!(comments[0].id, 9001);
    assert_eq!(comments[1].id, 9002);
}

#[tokio::test]
async fn test_fetch_comments_rate_limited() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/comments"))
        .respond_with(
            ResponseTemplate::new(403)
                .append_header("x-ratelimit-remaining", "0")
                .append_header("x-ratelimit-reset", "1746000000")
                .set_body_string(fixture("rate_limited.json")),
        )
        .mount(&server)
        .await;

    let result = fetch_review_comments(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("rate limited"), "expected rate limit error, got: {msg}");
    assert!(msg.contains("1746000000"), "should include reset timestamp");
}

// ─── parse_review_status ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_parse_review_status_approved() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/reviews"))
        .respond_with(
            ResponseTemplate::new(200).set_body_string(fixture("pr_reviews.json")),
        )
        .mount(&server)
        .await;

    let status = parse_review_status(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    assert_eq!(status, ReviewStatus::Approved);
}

#[tokio::test]
async fn test_parse_review_status_changes_requested() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/reviews"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_string(fixture("pr_reviews_changes_requested.json")),
        )
        .mount(&server)
        .await;

    let status = parse_review_status(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    assert!(
        matches!(status, ReviewStatus::ChangesRequested(_)),
        "expected ChangesRequested, got {status:?}"
    );
}

#[tokio::test]
async fn test_parse_review_status_not_found() {
    let server = MockServer::start().await;

    // No Coderabbit review, only a human.
    let reviews = serde_json::json!([{
        "id": 9999,
        "state": "APPROVED",
        "user": { "login": "human-dev" },
        "body": "LGTM"
    }]);

    Mock::given(method("GET"))
        .and(path("/repos/acme/repo/pulls/1/reviews"))
        .respond_with(ResponseTemplate::new(200).set_body_json(&reviews))
        .mount(&server)
        .await;

    let status = parse_review_status(
        &client(),
        &server.uri(),
        "token",
        "acme",
        "repo",
        1,
        "coderabbitai[bot]",
    )
    .await
    .unwrap();

    assert_eq!(status, ReviewStatus::NotFound);
}
