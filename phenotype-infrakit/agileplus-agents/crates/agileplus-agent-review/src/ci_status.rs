//! CI status checking via the GitHub Checks API and the legacy Status API.
//!
//! Both APIs are queried and results merged: all checks must be `completed`
//! with `conclusion: success` (or `neutral`) for `Passed`; any failure gives
//! `Failed`; in-progress checks yield `Pending`.

use agileplus_agent_dispatch::DomainError;
use chrono::{DateTime, Utc};
use serde::Deserialize;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tokio::time::sleep;
use tracing::{debug, info, warn};

// ─── Public types ─────────────────────────────────────────────────────────────

/// CI pipeline check result.
#[derive(Debug, Clone)]
pub struct CheckResult {
    pub name: String,
    /// `"completed"`, `"in_progress"`, or `"queued"`.
    pub status: String,
    /// `"success"`, `"failure"`, `"neutral"`, `"cancelled"`, etc.
    pub conclusion: Option<String>,
    pub details_url: Option<String>,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
}

/// Aggregate CI status for a PR.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CiStatus {
    Passed,
    Failed { failed_checks: Vec<String> },
    Pending { pending_checks: Vec<String> },
    Unknown,
}

// ─── GitHub API shapes ────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct GhPr {
    head: GhHead,
}

#[derive(Debug, Deserialize)]
struct GhHead {
    sha: String,
}

#[derive(Debug, Deserialize)]
struct GhCheckRunsPage {
    check_runs: Vec<GhCheckRun>,
}

#[derive(Debug, Deserialize)]
struct GhCheckRun {
    name: String,
    status: String,
    conclusion: Option<String>,
    details_url: Option<String>,
    started_at: Option<DateTime<Utc>>,
    completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
struct GhCombinedStatus {
    state: String,
    statuses: Vec<GhStatus>,
}

#[derive(Debug, Deserialize)]
struct GhStatus {
    context: String,
    state: String,
    target_url: Option<String>,
}

// ─── HTTP helper ──────────────────────────────────────────────────────────────

fn gh_get(client: &reqwest::Client, token: &str, url: &str) -> reqwest::RequestBuilder {
    client
        .get(url)
        .header("Authorization", format!("Bearer {token}"))
        .header("Accept", "application/vnd.github+json")
        .header("X-GitHub-Api-Version", "2022-11-28")
}

async fn json_get<T>(
    client: &reqwest::Client,
    token: &str,
    url: &str,
) -> Result<T, DomainError>
where
    T: serde::de::DeserializeOwned,
{
    debug!("GET {url}");
    let resp = gh_get(client, token, url)
        .send()
        .await
        .map_err(|e| DomainError::Other(format!("HTTP request failed: {e}")))?;

    // Rate limit check.
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

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().await.unwrap_or_default();
        return Err(DomainError::Other(format!(
            "GitHub API error {status}: {body}"
        )));
    }

    resp.json::<T>()
        .await
        .map_err(|e| DomainError::Other(format!("JSON decode failed: {e}")))
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// Resolve the head SHA for the given PR number.
async fn get_pr_head_sha(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<String, DomainError> {
    let url = format!("{api_base}/repos/{owner}/{repo}/pulls/{pr_number}");
    let pr: GhPr = json_get(client, token, &url).await?;
    Ok(pr.head.sha)
}

/// Fetch check runs for a given commit SHA.
async fn fetch_check_runs(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    sha: &str,
) -> Result<Vec<CheckResult>, DomainError> {
    let url = format!(
        "{api_base}/repos/{owner}/{repo}/commits/{sha}/check-runs?per_page=100"
    );
    let page: GhCheckRunsPage = json_get(client, token, &url).await?;
    Ok(page
        .check_runs
        .into_iter()
        .map(|r| CheckResult {
            name: r.name,
            status: r.status,
            conclusion: r.conclusion,
            details_url: r.details_url,
            started_at: r.started_at,
            completed_at: r.completed_at,
        })
        .collect())
}

/// Fetch the legacy combined commit status.
async fn fetch_commit_status(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    sha: &str,
) -> Result<Vec<CheckResult>, DomainError> {
    let url = format!("{api_base}/repos/{owner}/{repo}/commits/{sha}/status");
    let combined: GhCombinedStatus = json_get(client, token, &url).await?;

    if combined.statuses.is_empty() {
        return Ok(vec![]);
    }

    let results = combined
        .statuses
        .into_iter()
        .map(|s| {
            let conclusion = match s.state.as_str() {
                "success" => Some("success".to_owned()),
                "failure" | "error" => Some("failure".to_owned()),
                "pending" => None,
                other => Some(other.to_owned()),
            };
            let status = if conclusion.is_none() {
                "in_progress".to_owned()
            } else {
                "completed".to_owned()
            };
            CheckResult {
                name: s.context,
                status,
                conclusion,
                details_url: s.target_url,
                started_at: None,
                completed_at: None,
            }
        })
        .collect();

    Ok(results)
}

/// Aggregate a list of check results into a single `CiStatus`.
fn aggregate_checks(checks: &[CheckResult]) -> CiStatus {
    if checks.is_empty() {
        return CiStatus::Unknown;
    }

    let mut failed: Vec<String> = Vec::new();
    let mut pending: Vec<String> = Vec::new();

    for check in checks {
        match check.status.as_str() {
            "completed" => {
                let conclusion = check.conclusion.as_deref().unwrap_or("success");
                match conclusion {
                    "success" | "neutral" | "skipped" => {}
                    "failure" | "cancelled" | "timed_out" | "action_required" => {
                        failed.push(check.name.clone());
                    }
                    other => {
                        warn!(name = check.name, conclusion = other, "unknown check conclusion");
                    }
                }
            }
            "in_progress" | "queued" => {
                pending.push(check.name.clone());
            }
            other => {
                warn!(name = check.name, status = other, "unknown check status");
                pending.push(check.name.clone());
            }
        }
    }

    if !failed.is_empty() {
        CiStatus::Failed { failed_checks: failed }
    } else if !pending.is_empty() {
        CiStatus::Pending { pending_checks: pending }
    } else {
        CiStatus::Passed
    }
}

/// Query the current CI status for a pull request.
///
/// Combines results from both the Checks API and the legacy Status API.
pub async fn check_ci_status(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
) -> Result<CiStatus, DomainError> {
    let sha = get_pr_head_sha(client, api_base, token, owner, repo, pr_number).await?;
    debug!(sha, "resolved PR head SHA");

    // Fetch both APIs, ignore errors from individual sources (log warnings).
    let mut all_checks: Vec<CheckResult> = Vec::new();

    match fetch_check_runs(client, api_base, token, owner, repo, &sha).await {
        Ok(mut runs) => {
            debug!(count = runs.len(), "check runs fetched");
            all_checks.append(&mut runs);
        }
        Err(e) => {
            warn!("failed to fetch check runs: {e}");
        }
    }

    match fetch_commit_status(client, api_base, token, owner, repo, &sha).await {
        Ok(mut statuses) => {
            debug!(count = statuses.len(), "legacy statuses fetched");
            all_checks.append(&mut statuses);
        }
        Err(e) => {
            warn!("failed to fetch commit statuses: {e}");
        }
    }

    if all_checks.is_empty() {
        warn!(sha, "no check runs or statuses found — returning Unknown");
        return Ok(CiStatus::Unknown);
    }

    Ok(aggregate_checks(&all_checks))
}

/// Poll CI status at `interval` until it resolves or `max_wait` elapses.
///
/// An optional `CancellationToken` allows the caller to abort polling early.
/// Uses simple doubling backoff capped at `2 * interval`.
///
/// # Returns
///
/// - `CiStatus::Passed` or `CiStatus::Failed { .. }` once resolved.
/// - `CiStatus::Pending { .. }` if `max_wait` was exceeded.
#[allow(clippy::too_many_arguments)]
pub async fn poll_until_complete(
    client: &reqwest::Client,
    api_base: &str,
    token: &str,
    owner: &str,
    repo: &str,
    pr_number: u64,
    interval: Duration,
    max_wait: Duration,
    cancel: Option<&CancellationToken>,
) -> Result<CiStatus, DomainError> {
    let start = tokio::time::Instant::now();
    let mut current_interval = interval;

    loop {
        // Cancellation check.
        if let Some(ref ct) = cancel {
            if ct.is_cancelled() {
                info!("CI polling cancelled by token");
                return Ok(CiStatus::Unknown);
            }
        }

        // Timeout check.
        if start.elapsed() >= max_wait {
            info!("CI polling max_wait exceeded");
            return Ok(check_ci_status(client, api_base, token, owner, repo, pr_number).await?);
        }

        let status = check_ci_status(client, api_base, token, owner, repo, pr_number).await?;
        match &status {
            CiStatus::Passed | CiStatus::Failed { .. } => {
                info!(?status, "CI polling complete");
                return Ok(status);
            }
            CiStatus::Pending { pending_checks } => {
                debug!(
                    pending = pending_checks.len(),
                    "CI still pending — waiting {current_interval:?}"
                );
            }
            CiStatus::Unknown => {
                debug!("CI status unknown — waiting {current_interval:?}");
            }
        }

        sleep(current_interval).await;
        // Exponential back-off, capped at 2× the initial interval.
        current_interval = (current_interval * 2).min(interval * 2);
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn make_check(name: &str, status: &str, conclusion: Option<&str>) -> CheckResult {
        CheckResult {
            name: name.to_owned(),
            status: status.to_owned(),
            conclusion: conclusion.map(str::to_owned),
            details_url: None,
            started_at: None,
            completed_at: None,
        }
    }

    #[test]
    fn aggregate_all_passed() {
        let checks = vec![
            make_check("lint", "completed", Some("success")),
            make_check("tests", "completed", Some("success")),
        ];
        assert_eq!(aggregate_checks(&checks), CiStatus::Passed);
    }

    #[test]
    fn aggregate_one_failed() {
        let checks = vec![
            make_check("lint", "completed", Some("success")),
            make_check("tests", "completed", Some("failure")),
        ];
        assert!(matches!(
            aggregate_checks(&checks),
            CiStatus::Failed { .. }
        ));
        if let CiStatus::Failed { failed_checks } = aggregate_checks(&checks) {
            assert_eq!(failed_checks, vec!["tests"]);
        }
    }

    #[test]
    fn aggregate_pending() {
        let checks = vec![
            make_check("lint", "completed", Some("success")),
            make_check("tests", "in_progress", None),
        ];
        assert!(matches!(
            aggregate_checks(&checks),
            CiStatus::Pending { .. }
        ));
    }

    #[test]
    fn aggregate_empty_is_unknown() {
        assert_eq!(aggregate_checks(&[]), CiStatus::Unknown);
    }

    #[test]
    fn aggregate_neutral_counts_as_passed() {
        let checks = vec![make_check("docs", "completed", Some("neutral"))];
        assert_eq!(aggregate_checks(&checks), CiStatus::Passed);
    }

    #[test]
    fn aggregate_skipped_counts_as_passed() {
        let checks = vec![make_check("optional", "completed", Some("skipped"))];
        assert_eq!(aggregate_checks(&checks), CiStatus::Passed);
    }

    #[test]
    fn aggregate_cancelled_counts_as_failed() {
        let checks = vec![make_check("build", "completed", Some("cancelled"))];
        assert!(matches!(
            aggregate_checks(&checks),
            CiStatus::Failed { .. }
        ));
    }
}
