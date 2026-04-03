//! Plane.so REST API client with rate limiting.
//!
//! Traceability: WP18-T104

mod endpoints;
mod models;
mod rate_limit;
mod resources;
mod transport;

use std::sync::Arc;

use anyhow::Result;
use reqwest::Method;
use tokio::sync::Mutex;

pub use self::models::{
    PlaneCreateCycleRequest, PlaneCreateModuleRequest, PlaneCycleResponse, PlaneIssue,
    PlaneModuleResponse, PlaneWorkItem, PlaneWorkItemResponse,
};
use self::rate_limit::TokenBucket;

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

    async fn execute_request_json<T: serde::Serialize + ?Sized>(
        &self,
        method: Method,
        url: &str,
        body: &T,
    ) -> Result<reqwest::Response> {
        transport::request_json(&self.client, method, url, &self.api_key, body).await
    }

    async fn execute_request_without_body(
        &self,
        method: Method,
        url: &str,
    ) -> Result<reqwest::Response> {
        transport::request_without_body(&self.client, method, url, &self.api_key).await
    }
}

#[cfg(test)]
mod tests;
