//! Coderabbit integration: fetches and parses Coderabbit review comments from
//! GitHub pull requests via the GitHub REST API.

use agileplus_agent_dispatch::DomainError;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use tracing::{debug, warn};

// ─── Public types ─────────────────────────────────────────────────────────────

/// Severity level of a single Coderabbit comment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommentSeverity {
    Info,
    Warning,
    Error,
}

/// A single parsed Coderabbit comment.
#[derive(Debug, Clone)]
pub struct CoderabbitComment {
    /// GitHub comment ID.
    pub id: u64,
    /// Full comment body text.
    pub body: String,
    /// File path for inline comments, `None` for top-level PR comments.
    pub path: Option<String>,
    /// Line number for inline comments.
    pub line: Option<u32>,
    /// Whether this comment requires a code change.
    pub is_actionable: bool,
    /// Inferred severity.
    pub severity: CommentSeverity,
    /// UTC timestamp when the comment was created.
    pub created_at: DateTime<Utc>,
}

/// Overall review status as determined from Coderabbit's GitHub review.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReviewStatus {
    Approved,
    ChangesRequested(Vec<String>),
    Pending,
    NotFound,
}

// ─── GitHub API response shapes ───────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GhUser {
    login: String,
}

#[derive(Debug, Deserialize)]
struct GhPullComment {
    id: u64,
    body: String,
    path: Option<String>,
    line: Option<u32>,
    user: GhUser,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GhIssueComment {
    id: u64,
    body: String,
    user: GhUser,
    created_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
struct GhReview {
    state: String,
    user: GhUser,
    body: Option<String>,
}

// ─── HTTP helpers ─────────────────────────────────────────────────────────────

/// Build a GitHub API request with auth and accept headers.
fn gh_request(
    client: &reqwest::Client,
    token: &str,
    url: &str,
) -> reqwest::RequestBuilder {
    client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
}

/// Collect all pages from a paginated GitHub API endpoint.
///
/// Follows the `Link: <url>; rel="next"` header until exhausted.
async fn fetch_all_pages<T>(
    client: &reqwest::Client,
    token: &str,
    first_url: &str,
) -> Result<Vec<T>, DomainError>
where
    T: serde::de::DeserializeOwned,
{
    let mut results: Vec<T> = Vec::new();
    let mut url = format!("{first_url}?per_page=100");

    loop {
        debug!("fetching page: {url}");
        let resp = gh_request(client, token, &url)
            .send()
            .await
            .map_err(|e| DomainError::Other(format!("HTTP request failed: {e}")))?;

        check_rate_limit(&resp)?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(DomainError::Other(format!(
                "GitHub API error {status}: {body}"
            )));
        }

        let next_link = extract_next_link(resp.headers());
        let page: Vec<T> = resp
            .json()
            .await
            .map_err(|e| DomainError::Other(format!("JSON decode failed: {e}")))?;

        results.extend(page);

        match next_link {
            Some(next) => url = next,
            None => break,
        }
    }

    Ok(results)
}

/// Return the next-page URL from the GitHub `Link` response header, if any.
fn extract_next_link(headers: &reqwest::header::HeaderMap) -> Option<String> {
    let link = headers.get("link")?.to_str().ok()?;
    // Format: `<url>; rel="next", <url>; rel="last"`
    for part in link.split(',') {
        let part = part.trim();
        if part.contains(r#"rel="next""#) {
            if let Some(start) = part.find('<') {
                if let Some(end) = part.find('>') {
                    return Some(part[start + 1..end].to_owned());
                }
            }
        }
    }
    None
}

/// Check whether the response indicates API rate limiting.
///
/// Returns `Err` with a human-readable message including the reset timestamp.
fn check_rate_limit(resp: &reqwest::Response) -> Result<(), DomainError> {
    if resp.status().as_u16() == 403 {
        let remaining = resp
            .headers()
            .get("x-ratelimit-remaining")
            .and_then(|v| v.to_str().ok())
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(1);

        if remaining == 0 {
            let reset = resp
                .headers()
                .get("x-ratelimit-reset")
                .and_then(|v| v.to_str().ok())
                .unwrap_or("unknown");
            return Err(DomainError::Other(format!(
                "rate limited by GitHub API; quota resets at {reset}"
            )));
        }
    }
    Ok(())
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Fetch all Coderabbit comments (inline + top-level) from a pull request.
///
/// Calls two GitHub endpoints:
/// - `GET /repos/{owner}/{repo}/pulls/{pr}/comments` (inline review comments)
/// - `GET /repos/{owner}/{repo}/issues/{pr}/comments` (top-level PR comments)
///
/// Filters to only comments posted by `bot_username`, parses actionability and
/// severity, then returns the list sorted by creation time ascending.
pub async fn fetch_review_comments(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
    bot_username: &str,
) -> Result<Vec<CoderabbitComment>, DomainError> {
    // Inline review comments (code diffs).
    let inline_url = format!("{api_base}/repos/{owner}/{repo}/pulls/{pr_number}/comments");
    let inline_raw: Vec<GhPullComment> =
        fetch_all_pages(client, token, &inline_url).await?;

    // Top-level issue comments.
    let top_url = format!("{api_base}/repos/{owner}/{repo}/issues/{pr_number}/comments");
    let top_raw: Vec<GhIssueComment> = fetch_all_pages(client, token, &top_url).await?;

    let mut comments: Vec<CoderabbitComment> = Vec::new();

    for c in inline_raw {
        if c.user.login != bot_username {
            continue;
        }
        debug!(id = c.id, "inline coderabbit comment");
        let (is_actionable, severity) = classify_comment(&c.body);
        comments.push(CoderabbitComment {
            id: c.id,
            body: c.body,
            path: c.path,
            line: c.line,
            is_actionable,
            severity,
            created_at: c.created_at,
        });
    }

    for c in top_raw {
        if c.user.login != bot_username {
            continue;
        }
        debug!(id = c.id, "top-level coderabbit comment");
        let (is_actionable, severity) = classify_comment(&c.body);
        comments.push(CoderabbitComment {
            id: c.id,
            body: c.body,
            path: None,
            line: None,
            is_actionable,
            severity,
            created_at: c.created_at,
        });
    }

    // Sort ascending by creation time.
    comments.sort_by_key(|c| c.created_at);

    Ok(comments)
}

/// Fetch Coderabbit's formal review state from the PR reviews endpoint.
///
/// Returns [`ReviewStatus::Approved`] if Coderabbit submitted an `APPROVED`
/// review, [`ReviewStatus::ChangesRequested`] if it requested changes, or
/// [`ReviewStatus::Pending`] / [`ReviewStatus::NotFound`] otherwise.
pub async fn parse_review_status(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
    bot_username: &str,
) -> Result<ReviewStatus, DomainError> {
    let url = format!("{api_base}/repos/{owner}/{repo}/pulls/{pr_number}/reviews?per_page=100");
    let resp = gh_request(client, token, &url)
        .send()
        .await
        .map_err(|e| DomainError::Other(format!("HTTP request failed: {e}")))?;

    check_rate_limit(&resp)?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(DomainError::Other(format!(
            "GitHub reviews API error {status}: {body}"
        )));
    }

    let reviews: Vec<GhReview> = resp
        .json()
        .await
        .map_err(|e| DomainError::Other(format!("JSON decode error: {e}")))?;

    let bot_reviews: Vec<&GhReview> = reviews
        .iter()
        .filter(|r| r.user.login == bot_username)
        .collect();

    if bot_reviews.is_empty() {
        return Ok(ReviewStatus::NotFound);
    }

    // Use the last review state from the bot.
    let last = bot_reviews.last().expect("non-empty");
    match last.state.as_str() {
        "APPROVED" => Ok(ReviewStatus::Approved),
        "CHANGES_REQUESTED" => {
            let reasons: Vec<String> = bot_reviews
                .iter()
                .filter_map(|r| r.body.clone())
                .filter(|b| !b.is_empty())
                .collect();
            Ok(ReviewStatus::ChangesRequested(reasons))
        }
        "COMMENTED" => Ok(ReviewStatus::Pending),
        other => {
            warn!("unrecognised Coderabbit review state: {other}");
            Ok(ReviewStatus::Pending)
        }
    }
}

// ─── Comment classification ───────────────────────────────────────────────────

/// Determine whether a comment is actionable and its severity.
///
/// Actionable comments are ones that require a code change:
/// - Lines starting with `suggestion:`, `fix:`, `error:`, `warning:`
/// - Code block suggestions (` ```suggestion `)
/// - Comments with words like "must", "required", "fix", "error"
///
/// Summary and praise comments are informational.
fn classify_comment(body: &str) -> (bool, CommentSeverity) {
    let lower = body.to_lowercase();

    // Code block suggestion is always actionable.
    if lower.contains("```suggestion") {
        return (true, CommentSeverity::Warning);
    }

    // Keyword-prefixed lines.
    for line in body.lines() {
        let trimmed = line.trim().to_lowercase();
        if trimmed.starts_with("suggestion:")
            || trimmed.starts_with("fix:")
            || trimmed.starts_with("error:")
            || trimmed.starts_with("warning:")
        {
            let sev = if trimmed.starts_with("error:") {
                CommentSeverity::Error
            } else if trimmed.starts_with("warning:") {
                CommentSeverity::Warning
            } else {
                CommentSeverity::Warning
            };
            return (true, sev);
        }
    }

    // Body-level keyword detection.
    let is_actionable = lower.contains("must ")
        || lower.contains("must be")
        || lower.contains("required")
        || lower.contains("should fix")
        || lower.contains("needs to be fixed")
        || lower.contains("issue:")
        || lower.contains("actionable comment");

    let severity = if lower.contains("error") || lower.contains("critical") || lower.contains("must") {
        CommentSeverity::Error
    } else if lower.contains("warning") || lower.contains("should") || lower.contains("issue") {
        CommentSeverity::Warning
    } else {
        CommentSeverity::Info
    };

    (is_actionable, severity)
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classify_suggestion_block() {
        let body = "```suggestion\nlet x = 1;\n```";
        let (actionable, sev) = classify_comment(body);
        assert!(actionable);
        assert_eq!(sev, CommentSeverity::Warning);
    }

    #[test]
    fn classify_error_prefix() {
        let body = "Error: missing semicolon on line 42";
        let (actionable, sev) = classify_comment(body);
        assert!(actionable);
        assert_eq!(sev, CommentSeverity::Error);
    }

    #[test]
    fn classify_warning_prefix() {
        let body = "Warning: this function is too long";
        let (actionable, _sev) = classify_comment(body);
        assert!(actionable);
    }

    #[test]
    fn classify_praise_not_actionable() {
        let body = "Looks great! Good job overall.";
        let (actionable, sev) = classify_comment(body);
        assert!(!actionable);
        assert_eq!(sev, CommentSeverity::Info);
    }

    #[test]
    fn classify_must_is_actionable() {
        let body = "This must be addressed before merging.";
        let (actionable, _) = classify_comment(body);
        assert!(actionable);
    }

    #[test]
    fn extract_next_link_parses() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "link",
            r#"<https://api.github.com/page2>; rel="next", <https://api.github.com/page5>; rel="last""#
                .parse()
                .unwrap(),
        );
        let next = extract_next_link(&headers);
        assert_eq!(next, Some("https://api.github.com/page2".to_owned()));
    }

    #[test]
    fn extract_next_link_missing() {
        let headers = reqwest::header::HeaderMap::new();
        assert_eq!(extract_next_link(&headers), None);
    }

    #[test]
    fn extract_next_link_no_next_rel() {
        let mut headers = reqwest::header::HeaderMap::new();
        headers.insert(
            "link",
            r#"<https://api.github.com/page5>; rel="last""#.parse().unwrap(),
        );
        assert_eq!(extract_next_link(&headers), None);
    }
}
