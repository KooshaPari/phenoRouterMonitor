//! `agileplus-agent-review` — GitHub / Coderabbit review adapter.
//!
//! Implements [`ReviewPort`] by shelling out to the `gh` CLI to fetch review
//! decisions and comments, then parses the JSON output into the domain types
//! defined in `agileplus-agent-dispatch`.

use agileplus_agent_dispatch::{
    CiStatus, DomainError, ReviewComment, CommentSeverity, ReviewOutcome, ReviewPort,
};
use async_trait::async_trait;
use std::time::Duration;
use tokio::process::Command;
use tokio::time::{sleep, timeout, Instant};
use tracing::{debug, info, warn};

// ─── GhReviewAdapter ──────────────────────────────────────────────────────────

/// Live adapter that calls `gh` CLI to query GitHub review state.
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

            match query_review_decision(pr_url).await {
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
        fetch_review_comments(pr_url).await
    }

    async fn await_ci(
        &self,
        pr_url: &str,
        ci_timeout: Duration,
    ) -> Result<CiStatus, DomainError> {
        let future = async {
            let mut interval = Duration::from_secs(20);
            loop {
                match query_ci_status(pr_url).await {
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

/// Query the review decision via `gh pr view --json reviewDecision`.
async fn query_review_decision(pr_url: &str) -> Result<ReviewOutcome, DomainError> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("view")
        .arg(pr_url)
        .arg("--json")
        .arg("reviewDecision")
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .map_err(|e| DomainError::Other(format!("JSON parse failed: {e}")))?;

    let decision = json["reviewDecision"].as_str().unwrap_or("");
    Ok(match decision {
        "APPROVED" => ReviewOutcome::Approved,
        "CHANGES_REQUESTED" => ReviewOutcome::ChangesRequested,
        "DISMISSED" => ReviewOutcome::Dismissed,
        _ => ReviewOutcome::Pending,
    })
}

/// Fetch PR review comments via `gh pr view --json reviews,comments`.
async fn fetch_review_comments(pr_url: &str) -> Result<Vec<ReviewComment>, DomainError> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("view")
        .arg(pr_url)
        .arg("--json")
        .arg("reviews,comments")
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let json: serde_json::Value = serde_json::from_str(&stdout)
        .unwrap_or(serde_json::Value::Object(Default::default()));

    let mut comments = Vec::new();

    // Parse inline code-review comments.
    if let Some(reviews) = json["reviews"].as_array() {
        for review in reviews {
            if let Some(body) = review["body"].as_str() {
                if !body.is_empty() {
                    comments.push(ReviewComment {
                        file_path: "(review summary)".to_owned(),
                        line: None,
                        severity: infer_severity(body),
                        body: body.to_owned(),
                    });
                }
            }
        }
    }

    // Parse issue-style PR comments (Coderabbit posts here too).
    if let Some(pr_comments) = json["comments"].as_array() {
        for c in pr_comments {
            if let Some(body) = c["body"].as_str() {
                if !body.is_empty() && body.contains("coderabbit") || body.contains("**") {
                    comments.push(ReviewComment {
                        file_path: "(PR comment)".to_owned(),
                        line: None,
                        severity: infer_severity(body),
                        body: body.to_owned(),
                    });
                }
            }
        }
    }

    Ok(comments)
}

/// Query CI status via `gh pr checks`.
async fn query_ci_status(pr_url: &str) -> Result<CiStatus, DomainError> {
    let output = Command::new("gh")
        .arg("pr")
        .arg("checks")
        .arg(pr_url)
        .arg("--json")
        .arg("state")
        .output()
        .await
        .map_err(|e| DomainError::Other(format!("gh spawn failed: {e}")))?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    // `gh pr checks` exits non-zero when checks fail.
    if !output.status.success() {
        return Ok(CiStatus::Failing);
    }

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

/// Heuristic severity inference from comment body text.
fn infer_severity(body: &str) -> CommentSeverity {
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
    fn infer_severity_critical() {
        assert_eq!(infer_severity("This must be fixed — critical issue"), CommentSeverity::Critical);
    }

    #[test]
    fn infer_severity_major() {
        assert_eq!(infer_severity("This should be addressed"), CommentSeverity::Major);
    }

    #[test]
    fn infer_severity_minor() {
        assert_eq!(infer_severity("nit: consider renaming this"), CommentSeverity::Minor);
    }

    #[test]
    fn infer_severity_info() {
        assert_eq!(infer_severity("Looks good!"), CommentSeverity::Info);
    }
}
