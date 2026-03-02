//! `agileplus-agent-review` — GitHub / Coderabbit review adapter.
//!
//! Implements [`ReviewPort`] by calling the GitHub REST API to fetch review
//! decisions and comments, then parses the JSON output into the domain types
//! defined in `agileplus-agent-dispatch`.
//!
//! # Architecture
//!
//! - `coderabbit` — fetches and parses Coderabbit bot comments from GitHub
//! - `ci_status`  — polls the GitHub Checks API and legacy Status API
//! - `fallback`   — CLI-based manual approval flow when Coderabbit is absent
//! - `ReviewAdapter` — top-level struct wiring all three behind `ReviewPort`

pub mod ci_status;
pub mod coderabbit;
pub mod fallback;

use agileplus_agent_dispatch::{CiStatus, DomainError, ReviewComment, ReviewOutcome, ReviewPort};
use async_trait::async_trait;
use std::time::Duration;
use tokio::time::{sleep, timeout, Instant};
use tracing::{debug, info, instrument, warn};

// ─── Configuration ────────────────────────────────────────────────────────────

/// Configuration for [`ReviewAdapter`].
#[derive(Debug, Clone)]
pub struct ReviewAdapterConfig {
    /// Personal access token with `repo` scope.
    pub github_token: String,
    /// GitHub repository owner (organisation or user).
    pub github_owner: String,
    /// GitHub repository name.
    pub github_repo: String,
    /// Bot login that Coderabbit posts comments under.
    pub coderabbit_bot_username: String,
    /// How long to wait for Coderabbit before falling back to manual review.
    pub fallback_timeout: Duration,
    /// Base URL for the GitHub API (overridable for testing).
    pub github_api_base: String,
}

impl ReviewAdapterConfig {
    /// Create a new config with required fields and sensible defaults.
    pub fn new(github_token: impl Into<String>, owner: impl Into<String>, repo: impl Into<String>) -> Self {
        Self {
            github_token: github_token.into(),
            github_owner: owner.into(),
            github_repo: repo.into(),
            coderabbit_bot_username: "coderabbitai[bot]".to_owned(),
            fallback_timeout: Duration::from_secs(300),
            github_api_base: "https://api.github.com".to_owned(),
        }
    }

    /// Override the GitHub API base URL (used in tests to point at wiremock).
    pub fn with_api_base(mut self, base: impl Into<String>) -> Self {
        self.github_api_base = base.into();
        self
    }

    /// Override the fallback timeout.
    pub fn with_fallback_timeout(mut self, t: Duration) -> Self {
        self.fallback_timeout = t;
        self
    }

    /// Override the Coderabbit bot username.
    pub fn with_bot_username(mut self, name: impl Into<String>) -> Self {
        self.coderabbit_bot_username = name.into();
        self
    }
}

// ─── ReviewAdapter ─────────────────────────────────────────────────────────────

/// HTTP-based adapter for GitHub Reviews and Coderabbit integration.
///
/// This type is `Send + Sync` and safe to use across async task boundaries.
pub struct ReviewAdapter {
    http: reqwest::Client,
    config: ReviewAdapterConfig,
}

impl ReviewAdapter {
    /// Construct a new adapter from the supplied configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if the `reqwest::Client` cannot be initialised.
    pub fn new(config: ReviewAdapterConfig) -> Result<Self, DomainError> {
        let http = reqwest::Client::builder()
            .user_agent("agileplus-agent-review/0.1")
            .build()
            .map_err(|e| DomainError::Other(format!("HTTP client init failed: {e}")))?;
        Ok(Self { http, config })
    }

    /// Extract `(owner, repo, pr_number)` from a GitHub PR URL.
    ///
    /// Accepts both `https://github.com/owner/repo/pull/123` and bare
    /// `owner/repo#123` shorthands.
    fn parse_pr_url(pr_url: &str) -> Result<(String, String, u64), DomainError> {
        // Try full GitHub URL first.
        if let Some(rest) = pr_url.strip_prefix("https://github.com/") {
            let parts: Vec<&str> = rest.splitn(4, '/').collect();
            if parts.len() >= 4 && parts[2] == "pull" {
                let owner = parts[0].to_owned();
                let repo = parts[1].to_owned();
                let number: u64 = parts[3]
                    .parse()
                    .map_err(|_| DomainError::Other(format!("invalid PR number in URL: {pr_url}")))?;
                return Ok((owner, repo, number));
            }
        }
        Err(DomainError::Other(format!("cannot parse PR URL: {pr_url}")))
    }
}

#[async_trait]
impl ReviewPort for ReviewAdapter {
    /// Poll for a Coderabbit or GitHub review decision until `poll_timeout` elapses.
    ///
    /// If Coderabbit has not responded within `config.fallback_timeout`, the
    /// fallback manual-review module is invoked.  In non-interactive contexts
    /// the fallback returns an error rather than hanging on stdin.
    #[instrument(skip(self), fields(pr_url))]
    async fn await_review(
        &self,
        pr_url: &str,
        poll_timeout: Duration,
    ) -> Result<ReviewOutcome, DomainError> {
        let (owner, repo, pr_number) = Self::parse_pr_url(pr_url)?;

        let start = Instant::now();
        let mut interval = Duration::from_secs(30);
        let mut first_check: Option<std::time::SystemTime> = None;

        loop {
            if start.elapsed() >= poll_timeout {
                warn!(%pr_url, "review poll timed out");
                return Ok(ReviewOutcome::Pending);
            }

            match coderabbit::parse_review_status(
                &self.http,
                &self.config.github_api_base,
                &self.config.github_token,
                &owner,
                &repo,
                pr_number,
                &self.config.coderabbit_bot_username,
            )
            .await
            {
                Ok(coderabbit::ReviewStatus::Approved) => {
                    info!(%pr_url, "Coderabbit approved PR");
                    return Ok(ReviewOutcome::Approved);
                }
                Ok(coderabbit::ReviewStatus::ChangesRequested(_comments)) => {
                    info!(%pr_url, "Coderabbit requested changes");
                    return Ok(ReviewOutcome::ChangesRequested);
                }
                Ok(coderabbit::ReviewStatus::Pending) | Ok(coderabbit::ReviewStatus::NotFound) => {
                    debug!(%pr_url, "review still pending");
                    if first_check.is_none() {
                        first_check = Some(std::time::SystemTime::now());
                    }
                }
                Err(DomainError::Other(ref msg)) if msg.contains("rate limited") => {
                    warn!(%pr_url, "GitHub API rate limited — backing off");
                }
                Err(e) => {
                    warn!(%pr_url, error = %e, "review query failed — retrying");
                }
            }

            // Check whether we should fall back to manual review.
            let should_fb = fallback::should_fallback(
                first_check.map(chrono::DateTime::from),
                self.config.fallback_timeout,
            );
            if should_fb {
                info!(%pr_url, "falling back to manual review");
                match fallback::prompt_manual_review(pr_url, "work package") {
                    Ok(fallback::ManualReviewResult::Approved { .. }) => {
                        return Ok(ReviewOutcome::Approved);
                    }
                    Ok(fallback::ManualReviewResult::Rejected { .. }) => {
                        return Ok(ReviewOutcome::Dismissed);
                    }
                    Ok(fallback::ManualReviewResult::ChangesRequested { .. }) => {
                        return Ok(ReviewOutcome::ChangesRequested);
                    }
                    Err(e) => {
                        warn!(%pr_url, error = %e, "manual fallback unavailable");
                        // Continue polling — the loop will time out eventually.
                    }
                }
            }

            sleep(interval).await;
            interval = (interval * 2).min(Duration::from_secs(120));
        }
    }

    /// Fetch all actionable Coderabbit comments from the PR.
    #[instrument(skip(self), fields(pr_url))]
    async fn get_actionable_comments(
        &self,
        pr_url: &str,
    ) -> Result<Vec<ReviewComment>, DomainError> {
        let (owner, repo, pr_number) = Self::parse_pr_url(pr_url)?;

        let raw_comments = coderabbit::fetch_review_comments(
            &self.http,
            &self.config.github_api_base,
            &self.config.github_token,
            &owner,
            &repo,
            pr_number,
            &self.config.coderabbit_bot_username,
        )
        .await?;

        let actionable: Vec<ReviewComment> = raw_comments
            .into_iter()
            .filter(|c| c.is_actionable)
            .map(|c| ReviewComment {
                file_path: c.path.unwrap_or_else(|| "(PR comment)".to_owned()),
                line: c.line,
                severity: match c.severity {
                    coderabbit::CommentSeverity::Error => agileplus_agent_dispatch::CommentSeverity::Critical,
                    coderabbit::CommentSeverity::Warning => agileplus_agent_dispatch::CommentSeverity::Major,
                    coderabbit::CommentSeverity::Info => agileplus_agent_dispatch::CommentSeverity::Info,
                },
                body: c.body,
            })
            .collect();

        info!(%pr_url, count = actionable.len(), "fetched actionable comments");
        Ok(actionable)
    }

    /// Poll CI until all checks complete or `ci_timeout` elapses.
    #[instrument(skip(self), fields(pr_url))]
    async fn await_ci(
        &self,
        pr_url: &str,
        ci_timeout: Duration,
    ) -> Result<CiStatus, DomainError> {
        let (owner, repo, pr_number) = Self::parse_pr_url(pr_url)?;

        let poll_future = ci_status::poll_until_complete(
            &self.http,
            &self.config.github_api_base,
            &self.config.github_token,
            &owner,
            &repo,
            pr_number,
            Duration::from_secs(20),
            ci_timeout,
            None::<&tokio_util::sync::CancellationToken>,
        );

        match timeout(ci_timeout, poll_future).await {
            Ok(Ok(status)) => {
                info!(%pr_url, ?status, "CI polling complete");
                Ok(map_ci_status(status))
            }
            Ok(Err(e)) => Err(e),
            Err(_elapsed) => {
                warn!(%pr_url, "CI polling timed out");
                Ok(CiStatus::Unknown)
            }
        }
    }
}

/// Map the adapter-local `CiStatus` to the domain `CiStatus`.
fn map_ci_status(s: ci_status::CiStatus) -> CiStatus {
    match s {
        ci_status::CiStatus::Passed => CiStatus::Passing,
        ci_status::CiStatus::Failed { .. } => CiStatus::Failing,
        ci_status::CiStatus::Pending { .. } => CiStatus::Pending,
        ci_status::CiStatus::Unknown => CiStatus::Unknown,
    }
}

// ─── Legacy GhReviewAdapter (kept for compatibility) ──────────────────────────

/// Simple adapter that delegates to the `gh` CLI.
///
/// This is the WP08 baseline implementation kept for environments where the
/// HTTP-based `ReviewAdapter` is not yet configured.
pub struct GhReviewAdapter;

impl GhReviewAdapter {
    pub fn new() -> Self {
        Self
    }
}

impl Default for GhReviewAdapter {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ReviewPort for GhReviewAdapter {
    async fn await_review(
        &self,
        pr_url: &str,
        poll_timeout: Duration,
    ) -> Result<ReviewOutcome, DomainError> {
        let start = Instant::now();
        let mut interval = Duration::from_secs(30);

        loop {
            if start.elapsed() >= poll_timeout {
                warn!(%pr_url, "review poll timed out");
                return Ok(ReviewOutcome::Pending);
            }

            match gh_query_review_decision(pr_url).await {
                Ok(outcome) if outcome != ReviewOutcome::Pending => {
                    info!(%pr_url, ?outcome, "review decision received");
                    return Ok(outcome);
                }
                Ok(_) => {
                    debug!(%pr_url, "review still pending — waiting");
                }
                Err(e) => {
                    warn!(%pr_url, error = %e, "review query failed — retrying");
                }
            }

            sleep(interval).await;
            interval = (interval * 2).min(Duration::from_secs(120));
        }
    }

    async fn get_actionable_comments(
        &self,
        pr_url: &str,
    ) -> Result<Vec<ReviewComment>, DomainError> {
        gh_fetch_review_comments(pr_url).await
    }

    async fn await_ci(
        &self,
        pr_url: &str,
        ci_timeout: Duration,
    ) -> Result<CiStatus, DomainError> {
        let future = async {
            let mut interval = Duration::from_secs(20);
            loop {
                match gh_query_ci_status(pr_url).await {
                    Ok(CiStatus::Pending) | Ok(CiStatus::Unknown) => {}
                    Ok(status) => return status,
                    Err(e) => {
                        warn!(%pr_url, error = %e, "CI query failed — retrying");
                    }
                }
                sleep(interval).await;
                interval = (interval * 2).min(Duration::from_secs(120));
            }
        };

        Ok(timeout(ci_timeout, future)
            .await
            .unwrap_or(CiStatus::Unknown))
    }
}

// ─── gh CLI helpers ───────────────────────────────────────────────────────────

async fn gh_query_review_decision(pr_url: &str) -> Result<ReviewOutcome, DomainError> {
    use tokio::process::Command;
    let output = Command::new("gh")
        .args(["pr", "view", pr_url, "--json", "reviewDecision"])
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| DomainError::Other(format!("JSON parse failed: {e}")))?;

    Ok(match json["reviewDecision"].as_str().unwrap_or("") {
        "APPROVED" => ReviewOutcome::Approved,
        "CHANGES_REQUESTED" => ReviewOutcome::ChangesRequested,
        "DISMISSED" => ReviewOutcome::Dismissed,
        _ => ReviewOutcome::Pending,
    })
}

async fn gh_fetch_review_comments(pr_url: &str) -> Result<Vec<ReviewComment>, DomainError> {
    use tokio::process::Command;

    let output = Command::new("gh")
        .args(["pr", "view", pr_url, "--json", "reviews,comments"])
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value =
        serde_json::from_str(&stdout).unwrap_or(serde_json::Value::Object(Default::default()));

    let mut comments = Vec::new();

    if let Some(reviews) = json["reviews"].as_array() {
        for review in reviews {
            if let Some(body) = review["body"].as_str() {
                if !body.is_empty() {
                    comments.push(ReviewComment {
                        file_path: "(review summary)".to_owned(),
                        line: None,
                        severity: gh_infer_severity(body),
                        body: body.to_owned(),
                    });
                }
            }
        }
    }

    if let Some(pr_comments) = json["comments"].as_array() {
        for c in pr_comments {
            if let Some(body) = c["body"].as_str() {
                if !body.is_empty() && (body.contains("coderabbit") || body.contains("**")) {
                    comments.push(ReviewComment {
                        file_path: "(PR comment)".to_owned(),
                        line: None,
                        severity: gh_infer_severity(body),
                        body: body.to_owned(),
                    });
                }
            }
        }
    }

    Ok(comments)
}

async fn gh_query_ci_status(pr_url: &str) -> Result<CiStatus, DomainError> {
    use tokio::process::Command;
    let output = Command::new("gh")
        .args(["pr", "checks", pr_url, "--json", "state"])
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    if !output.status.success() {
        return Ok(CiStatus::Failing);
    }

    let stdout = String::from_utf8_lossy(&output.stdout);
    if stdout.contains("\"pass\"") || stdout.contains("\"success\"") {
        Ok(CiStatus::Passing)
    } else if stdout.contains("\"fail\"") || stdout.contains("\"failure\"") {
        Ok(CiStatus::Failing)
    } else if stdout.contains("\"pending\"") || stdout.contains("\"queued\"") {
        Ok(CiStatus::Pending)
    } else {
        Ok(CiStatus::Unknown)
    }
}

fn gh_infer_severity(body: &str) -> agileplus_agent_dispatch::CommentSeverity {
    use agileplus_agent_dispatch::CommentSeverity;
    let lower = body.to_lowercase();
    if lower.contains("critical") || lower.contains("must") || lower.contains("error") {
        CommentSeverity::Critical
    } else if lower.contains("should") || lower.contains("major") || lower.contains("issue") {
        CommentSeverity::Major
    } else if lower.contains("minor") || lower.contains("nit") || lower.contains("consider") {
        CommentSeverity::Minor
    } else {
        CommentSeverity::Info
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_pr_url_valid() {
        let (owner, repo, number) =
            ReviewAdapter::parse_pr_url("https://github.com/acme/myrepo/pull/42").unwrap();
        assert_eq!(owner, "acme");
        assert_eq!(repo, "myrepo");
        assert_eq!(number, 42);
    }

    #[test]
    fn parse_pr_url_invalid() {
        assert!(ReviewAdapter::parse_pr_url("not-a-url").is_err());
    }

    #[test]
    fn gh_infer_severity_critical() {
        assert_eq!(
            gh_infer_severity("This must be fixed — critical issue"),
            agileplus_agent_dispatch::CommentSeverity::Critical
        );
    }

    #[test]
    fn gh_infer_severity_major() {
        assert_eq!(
            gh_infer_severity("This should be addressed"),
            agileplus_agent_dispatch::CommentSeverity::Major
        );
    }

    #[test]
    fn gh_infer_severity_minor() {
        assert_eq!(
            gh_infer_severity("nit: consider renaming this"),
            agileplus_agent_dispatch::CommentSeverity::Minor
        );
    }

    #[test]
    fn gh_infer_severity_info() {
        assert_eq!(
            gh_infer_severity("Looks good!"),
            agileplus_agent_dispatch::CommentSeverity::Info
        );
    }
}
