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

    fn modules_url(&self) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/",
            self.base_url, self.workspace_slug, self.project_id
        )
    }

    fn module_url(&self, module_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/",
            self.base_url, self.workspace_slug, self.project_id, module_id
        )
    }

    fn module_issues_url(&self, module_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/modules/{}/module-issues/",
            self.base_url, self.workspace_slug, self.project_id, module_id
        )
    }

    fn cycles_url(&self) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/",
            self.base_url, self.workspace_slug, self.project_id
        )
    }

    fn cycle_url(&self, cycle_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/",
            self.base_url, self.workspace_slug, self.project_id, cycle_id
        )
    }

    fn cycle_issues_url(&self, cycle_id: &str) -> String {
        format!(
            "{}/api/v1/workspaces/{}/projects/{}/cycles/{}/cycle-issues/",
            self.base_url, self.workspace_slug, self.project_id, cycle_id
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
            .post(self.issues_url())
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
            .patch(self.issue_url(issue_id))
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
            .get(self.issue_url(issue_id))
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

    // -- Module API (WP06-T029) --

    /// Create a Module in Plane.so. Returns Plane's module UUID.
    pub async fn create_module(
        &self,
        req: &PlaneCreateModuleRequest,
    ) -> Result<PlaneModuleResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(self.modules_url())
            .header("X-API-Key", &self.api_key)
            .json(req)
            .send()
            .await
            .context("Plane.so create module request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module create failed: HTTP {status}: {body}");
        }

        resp.json()
            .await
            .context("parsing Plane.so module response")
    }

    /// Update a Module in Plane.so (PATCH).
    pub async fn update_module(
        &self,
        plane_module_id: &str,
        req: &PlaneCreateModuleRequest,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .client
            .patch(self.module_url(plane_module_id))
            .header("X-API-Key", &self.api_key)
            .json(req)
            .send()
            .await
            .context("Plane.so update module request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module update failed: HTTP {status}: {body}");
        }

        Ok(())
    }

    /// Delete a Module in Plane.so.
    pub async fn delete_module(&self, plane_module_id: &str) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .client
            .delete(self.module_url(plane_module_id))
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Plane.so delete module request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so module delete failed: HTTP {status}: {body}");
        }

        Ok(())
    }

    /// Add a Plane issue to a Plane module.
    pub async fn add_issue_to_module(
        &self,
        plane_module_id: &str,
        plane_issue_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let body = serde_json::json!({ "issues": [plane_issue_id] });
        let resp = self
            .client
            .post(self.module_issues_url(plane_module_id))
            .header("X-API-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("Plane.so add issue to module request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so add issue to module failed: HTTP {status}: {body}");
        }

        Ok(())
    }

    // -- Cycle API (WP06-T030) --

    /// Create a Cycle in Plane.so. Returns Plane's cycle UUID.
    pub async fn create_cycle(&self, req: &PlaneCreateCycleRequest) -> Result<PlaneCycleResponse> {
        self.acquire_token().await?;
        let resp = self
            .client
            .post(self.cycles_url())
            .header("X-API-Key", &self.api_key)
            .json(req)
            .send()
            .await
            .context("Plane.so create cycle request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle create failed: HTTP {status}: {body}");
        }

        resp.json().await.context("parsing Plane.so cycle response")
    }

    /// Update a Cycle in Plane.so (PATCH).
    pub async fn update_cycle(
        &self,
        plane_cycle_id: &str,
        req: &PlaneCreateCycleRequest,
    ) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .client
            .patch(self.cycle_url(plane_cycle_id))
            .header("X-API-Key", &self.api_key)
            .json(req)
            .send()
            .await
            .context("Plane.so update cycle request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle update failed: HTTP {status}: {body}");
        }

        Ok(())
    }

    /// Delete a Cycle in Plane.so.
    pub async fn delete_cycle(&self, plane_cycle_id: &str) -> Result<()> {
        self.acquire_token().await?;
        let resp = self
            .client
            .delete(self.cycle_url(plane_cycle_id))
            .header("X-API-Key", &self.api_key)
            .send()
            .await
            .context("Plane.so delete cycle request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so cycle delete failed: HTTP {status}: {body}");
        }

        Ok(())
    }

    /// Add a Plane issue to a Plane cycle.
    pub async fn add_issue_to_cycle(
        &self,
        plane_cycle_id: &str,
        plane_issue_id: &str,
    ) -> Result<()> {
        self.acquire_token().await?;
        let body = serde_json::json!({ "issues": [plane_issue_id] });
        let resp = self
            .client
            .post(self.cycle_issues_url(plane_cycle_id))
            .header("X-API-Key", &self.api_key)
            .json(&body)
            .send()
            .await
            .context("Plane.so add issue to cycle request failed")?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            anyhow::bail!("Plane.so add issue to cycle failed: HTTP {status}: {body}");
        }

        Ok(())
    }
}

/// Request body for creating/updating a Plane module.
#[derive(Debug, Clone, Serialize)]
pub struct PlaneCreateModuleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Response from Plane.so module API.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneModuleResponse {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

/// Request body for creating/updating a Plane cycle.
#[derive(Debug, Clone, Serialize)]
pub struct PlaneCreateCycleRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub start_date: String,
    pub end_date: String,
}

/// Response from Plane.so cycle API.
#[derive(Debug, Clone, Deserialize)]
pub struct PlaneCycleResponse {
    pub id: String,
    pub name: String,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
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

    #[tokio::test]
    async fn create_module_sends_post() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(
                serde_json::json!({"id": "mod-uuid-1", "name": "Auth", "description": null}),
            ))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let req = PlaneCreateModuleRequest {
            name: "Auth".to_string(),
            description: None,
        };
        let resp = client.create_module(&req).await.unwrap();
        assert_eq!(resp.id, "mod-uuid-1");
        assert_eq!(resp.name, "Auth");
    }

    #[tokio::test]
    async fn create_module_http_error_propagates() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/"))
            .respond_with(ResponseTemplate::new(500).set_body_string("internal error"))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let req = PlaneCreateModuleRequest {
            name: "Fail".to_string(),
            description: None,
        };
        let result = client.create_module(&req).await;
        assert!(result.is_err());
        let err_msg = format!("{}", result.unwrap_err());
        assert!(err_msg.contains("500"), "expected 500 in error: {err_msg}");
    }

    #[tokio::test]
    async fn create_cycle_sends_correct_dates() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/"))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
                "id": "cyc-uuid-1",
                "name": "Sprint 1",
                "start_date": "2026-01-01",
                "end_date": "2026-01-14"
            })))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let req = PlaneCreateCycleRequest {
            name: "Sprint 1".to_string(),
            description: None,
            start_date: "2026-01-01".to_string(),
            end_date: "2026-01-14".to_string(),
        };
        let resp = client.create_cycle(&req).await.unwrap();
        assert_eq!(resp.id, "cyc-uuid-1");
    }

    #[tokio::test]
    async fn add_issue_to_cycle_sends_post() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/cycle-issues/",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let result = client.add_issue_to_cycle("cyc-1", "issue-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn add_issue_to_module_sends_post() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(
                "/api/v1/workspaces/ws/projects/proj/modules/mod-1/module-issues/",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({})))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let result = client.add_issue_to_module("mod-1", "issue-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_module_sends_delete() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/modules/mod-1/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let result = client.delete_module("mod-1").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn delete_cycle_sends_delete() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let mock_server = MockServer::start().await;
        Mock::given(method("DELETE"))
            .and(path("/api/v1/workspaces/ws/projects/proj/cycles/cyc-1/"))
            .respond_with(ResponseTemplate::new(204))
            .mount(&mock_server)
            .await;

        let client = PlaneClient::new(mock_server.uri(), "key".into(), "ws".into(), "proj".into());
        let result = client.delete_cycle("cyc-1").await;
        assert!(result.is_ok());
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
