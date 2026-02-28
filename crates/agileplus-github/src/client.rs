//! GitHub Issues REST API client with rate limiting.
//!
//! Traceability: WP19-T109

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// GitHub Issues API client with token bucket rate limiter.
#[derive(Debug, Clone)]
pub struct GitHubClient {
    base_url: String,
    token: String,
    owner: String,
    repo: String,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<TokenBucket>>,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64,
    last_refill: Instant,
}

impl TokenBucket {
    fn new(max_tokens: f64, refill_rate: f64) -> Self {
        Self {
            tokens: max_tokens,
            max_tokens,
            refill_rate,
            last_refill: Instant::now(),
        }
    }

    fn try_acquire(&mut self) -> bool {
        let now = Instant::now();
        let elapsed = now.duration_since(self.last_refill).as_secs_f64();
        self.tokens = (self.tokens + elapsed * self.refill_rate).min(self.max_tokens);
        self.last_refill = now;
        if self.tokens >= 1.0 {
            self.tokens -= 1.0;
            true
        } else {
            false
        }
    }

    fn time_until_available(&self) -> Duration {
        if self.tokens >= 1.0 {
            Duration::ZERO
        } else {
            Duration::from_secs_f64((1.0 - self.tokens) / self.refill_rate)
        }
    }
}

/// GitHub Issue create/update payload.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitHubIssuePayload {
    pub title: String,
    pub body: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub labels: Vec<String>,
}

/// GitHub Issue response.
#[derive(Debug, Clone, Deserialize)]
pub struct GitHubIssueResponse {
    pub number: i64,
    pub title: String,
    pub body: Option<String>,
    pub state: String,
    pub labels: Vec<GitHubLabel>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct GitHubLabel {
    pub name: String,
}

impl GitHubClient {
    /// Create a new GitHub client. Rate limited to 60 req/min.
    pub fn new(base_url: String, token: String, owner: String, repo: String) -> Self {
        Self {
            base_url,
            token,
            owner,
            repo,
            client: reqwest::Client::new(),
            rate_limiter: Arc::new(Mutex::new(TokenBucket::new(60.0, 1.0))),
        }
    }

    async fn acquire_token(&self) -> Result<()> {
        loop {
            let mut limiter = self.rate_limiter.lock().await;
            if limiter.try_acquire() {
                return Ok(());
            }
            let wait = limiter.time_until_available();
            drop(limiter);
            tokio::time::sleep(wait).await;
        }
    }

    fn issues_url(&self) -> String {
        format!(
            "{}/repos/{}/{}/issues",
            self.base_url, self.owner, self.repo
        )
    }

    fn issue_url(&self, number: i64) -> String {
        format!(
            "{}/repos/{}/{}/issues/{}",
            self.base_url, self.owner, self.repo, number
        )
    }

    /// Create a new issue.
    pub async fn create_issue(
        &self,
        payload: &GitHubIssuePayload,
    ) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(&self.issues_url())
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .json(payload)
            .send()
            .await
            .context("GitHub create issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }

    /// Update an existing issue.
    pub async fn update_issue(
        &self,
        number: i64,
        payload: &GitHubIssuePayload,
    ) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .patch(&self.issue_url(number))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .json(payload)
            .send()
            .await
            .context("GitHub update issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }

    /// Get an issue by number.
    pub async fn get_issue(&self, number: i64) -> Result<GitHubIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .get(&self.issue_url(number))
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/vnd.github+json")
            .header("User-Agent", "agileplus")
            .send()
            .await
            .context("GitHub get issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("GitHub API error {status}: {body}");
        }

        resp.json().await.context("parsing GitHub response")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_bucket_basic() {
        let mut bucket = TokenBucket::new(5.0, 1.0);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
    }

    #[test]
    fn github_payload_serialize() {
        let payload = GitHubIssuePayload {
            title: "Bug: crash on start".to_string(),
            body: "## Description\nApp crashes".to_string(),
            labels: vec!["bug".to_string(), "agileplus".to_string()],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(json.contains("Bug: crash on start"));
        assert!(json.contains("labels"));
    }

    #[test]
    fn empty_labels_omitted() {
        let payload = GitHubIssuePayload {
            title: "Test".to_string(),
            body: "Body".to_string(),
            labels: vec![],
        };
        let json = serde_json::to_string(&payload).unwrap();
        assert!(!json.contains("labels"));
    }
}
