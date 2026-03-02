//! Plane.so REST API client with rate limiting.
//!
//! Traceability: WP18-T104

use std::sync::Arc;
use std::time::{Duration, Instant};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

/// Plane.so API client with token bucket rate limiter.
#[derive(Debug, Clone)]
pub struct PlaneClient {
    base_url: String,
    api_key: String,
    workspace_slug: String,
    project_id: String,
    client: reqwest::Client,
    rate_limiter: Arc<Mutex<TokenBucket>>,
}

#[derive(Debug)]
struct TokenBucket {
    tokens: f64,
    max_tokens: f64,
    refill_rate: f64, // tokens per second
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
            let needed = 1.0 - self.tokens;
            Duration::from_secs_f64(needed / self.refill_rate)
        }
    }
}

/// Issue representation in Plane.so.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaneIssue {
    pub id: Option<String>,
    pub name: String,
    pub description_html: Option<String>,
    pub state: Option<String>,
    pub priority: Option<i32>,
    pub parent: Option<String>,
    pub labels: Vec<String>,
}

/// Response from Plane.so API for issue creation/update.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneIssueResponse {
    pub id: String,
    pub name: String,
    pub description_html: Option<String>,
    pub state: Option<String>,
    pub updated_at: Option<String>,
}

impl PlaneClient {
    /// Create a new Plane.so client.
    /// Rate limited to 50 requests/minute.
    pub fn new(
        base_url: String,
        api_key: String,
        workspace_slug: String,
        project_id: String,
    ) -> Self {
        Self {
            base_url,
            api_key,
            workspace_slug,
            project_id,
            client: reqwest::Client::new(),
            // 50 req/min = 0.833 req/sec
            rate_limiter: Arc::new(Mutex::new(TokenBucket::new(50.0, 50.0 / 60.0))),
        }
    }

    /// Wait for rate limit token, then proceed.
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
            "{}/api/v1/workspaces/{}/projects/{}/issues/",
            self.base_url, self.workspace_slug, self.project_id
        )
    }

    fn issue_url(&self, issue_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/issues/{}/",
            self.base_url, self.workspace_slug, self.project_id, issue_id
        )
    }

    pub fn labels_url(&self) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/labels/",
            self.base_url, self.workspace_slug, self.project_id
        )
    }

    /// Make a raw GET request and return response body as String.
    pub async fn get_raw(&self, url: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp = self
            .client
            .get(url)
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Plane.so GET request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so API error {status}: {body}");
        }

        resp.text().await.context("reading Plane.so response body")
    }

    /// Make a raw POST request with JSON body and return response body as String.
    pub async fn post_raw(&self, url: &str, json_body: &str) -> Result<String> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(url)
            .header("X-API-Key", &self.api_key)
            .header("Content-Type", "application/json")
            .body(json_body.to_string())
            .send()
            .await
            .context("Plane.so POST request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so API error {status}: {body}");
        }

        resp.text().await.context("reading Plane.so response body")
    }

    /// Create an issue in Plane.so.
    pub async fn create_issue(&self, issue: &PlaneIssue) -> Result<PlaneIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(&self.issues_url())
            .header("X-API-Key", &self.api_key)
            .json(issue)
            .send()
            .await
            .context("Plane.so create issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so API error {status}: {body}");
        }

        resp.json().await.context("parsing Plane.so response")
    }

    /// Update an existing issue.
    pub async fn update_issue(
        &self,
        issue_id: &str,
        issue: &PlaneIssue,
    ) -> Result<PlaneIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .patch(&self.issue_url(issue_id))
            .header("X-API-Key", &self.api_key)
            .json(issue)
            .send()
            .await
            .context("Plane.so update issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so API error {status}: {body}");
        }

        resp.json().await.context("parsing Plane.so response")
    }

    /// Get an issue by ID.
    pub async fn get_issue(&self, issue_id: &str) -> Result<PlaneIssueResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .get(&self.issue_url(issue_id))
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Plane.so get issue request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so API error {status}: {body}");
        }

        resp.json().await.context("parsing Plane.so response")
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
    fn token_bucket_exhaustion() {
        let mut bucket = TokenBucket::new(2.0, 0.1);
        assert!(bucket.try_acquire());
        assert!(bucket.try_acquire());
        assert!(!bucket.try_acquire()); // exhausted
    }

    #[test]
    fn plane_issue_serialize() {
        let issue = PlaneIssue {
            id: None,
            name: "Test issue".to_string(),
            description_html: Some("<p>desc</p>".to_string()),
            state: None,
            priority: Some(2),
            parent: None,
            labels: vec!["agileplus".to_string()],
        };
        let json = serde_json::to_string(&issue).unwrap();
        assert!(json.contains("Test issue"));
    }
}
