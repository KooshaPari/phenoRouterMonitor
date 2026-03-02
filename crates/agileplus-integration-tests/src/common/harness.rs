//! Service startup and teardown harness for integration tests.
//!
//! Traceability: WP19-T107

use std::path::PathBuf;
use std::process::Child;
use std::time::{Duration, Instant};

use thiserror::Error;

/// Errors that can occur when starting or using the test harness.
#[derive(Debug, Error)]
pub enum HarnessError {
    #[error("process-compose is not installed; integration tests require it")]
    ProcessComposeNotInstalled,

    #[error("services did not become healthy within timeout")]
    HealthCheckTimeout,

    #[error("failed to start services: {0}")]
    StartFailed(String),

    #[error("HTTP request failed: {0}")]
    Http(#[from] reqwest::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("JSON decode error: {0}")]
    Json(#[from] serde_json::Error),

    #[error("UTF-8 decode error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
}

/// Holds service process handles and HTTP client for integration tests.
///
/// On `Drop`, services are stopped via `process-compose down`.
pub struct TestHarness {
    /// Handle to the process-compose daemon (kept alive for the test).
    _process_handle: Option<Child>,
    /// Reqwest client pre-configured with the API base URL.
    client: reqwest::Client,
    /// Base URL of the API server under test.
    base_url: String,
    /// Working directory where `process-compose.yml` lives.
    compose_dir: PathBuf,
}

impl TestHarness {
    /// Attempt to start all services and return a fully-initialized harness.
    ///
    /// Returns [`HarnessError::ProcessComposeNotInstalled`] if `process-compose`
    /// is not found on `PATH`; callers should skip the test in that case.
    pub async fn start() -> Result<Self, HarnessError> {
        if !is_process_compose_installed() {
            return Err(HarnessError::ProcessComposeNotInstalled);
        }

        let compose_dir = project_root();
        let base_url = "http://localhost:3000".to_string();

        // Launch process-compose in the background.
        let child = std::process::Command::new("process-compose")
            .arg("up")
            .arg("-f")
            .arg("process-compose.yml")
            .arg("--detach")
            .current_dir(&compose_dir)
            .spawn()?;

        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        let harness = Self {
            _process_handle: Some(child),
            client,
            base_url,
            compose_dir,
        };

        harness.wait_for_health(Duration::from_secs(30)).await?;

        Ok(harness)
    }

    /// Build a harness that talks to an already-running API server.
    ///
    /// Useful for CI environments that start services outside of tests.
    pub fn from_running(base_url: impl Into<String>) -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(10))
            .build()
            .expect("failed to build reqwest client");
        Self {
            _process_handle: None,
            client,
            base_url: base_url.into(),
            compose_dir: project_root(),
        }
    }

    /// Return a reference to the pre-configured HTTP client.
    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }

    /// Return the base API URL (e.g. `http://localhost:3000`).
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build a full URL from a path fragment (e.g. `/api/features`).
    pub fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    // -----------------------------------------------------------------------
    // Private helpers
    // -----------------------------------------------------------------------

    async fn wait_for_health(&self, timeout: Duration) -> Result<(), HarnessError> {
        let start = Instant::now();
        loop {
            if let Ok(resp) = self.client.get(self.url("/health")).send().await {
                if resp.status().is_success() {
                    return Ok(());
                }
            }
            if start.elapsed() > timeout {
                return Err(HarnessError::HealthCheckTimeout);
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }
}

impl Drop for TestHarness {
    fn drop(&mut self) {
        // Best-effort shutdown; ignore errors during cleanup.
        let _ = std::process::Command::new("process-compose")
            .arg("down")
            .arg("-f")
            .arg("process-compose.yml")
            .current_dir(&self.compose_dir)
            .output();
    }
}

/// Check whether `process-compose` is available on `PATH`.
pub fn is_process_compose_installed() -> bool {
    std::process::Command::new("which")
        .arg("process-compose")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Return the workspace root by walking up from the crate manifest directory.
pub fn project_root() -> PathBuf {
    // In tests the working directory is the crate root; the workspace root
    // is two levels up (crates/agileplus-integration-tests → workspace).
    let crate_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    crate_dir
        .parent() // crates/
        .and_then(|p| p.parent()) // workspace root
        .map(PathBuf::from)
        .unwrap_or_else(|| crate_dir.clone())
}

// ---------------------------------------------------------------------------
// Unit tests — no external services required
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn project_root_exists() {
        let root = project_root();
        assert!(root.exists(), "project root should exist: {:?}", root);
    }

    #[test]
    fn process_compose_check_does_not_panic() {
        // We don't assert the result — CI may or may not have it installed.
        let _ = is_process_compose_installed();
    }

    #[test]
    fn harness_url_concatenation() {
        // Create a dummy harness using from_running to test URL building.
        let harness = TestHarness::from_running("http://localhost:3000");
        assert_eq!(harness.url("/health"), "http://localhost:3000/health");
        assert_eq!(harness.url("/api/features"), "http://localhost:3000/api/features");
    }
}
