//! HTTP client

pub mod error;

pub use error::{HttpError, Result};

/// HTTP client wrapper
pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: &str) -> Result<String> {
        Ok(self.client.get(url).send().await?.text().await?)
    }
}

impl Default for HttpClient {
    fn default() -> Self {
        Self::new()
    }
}

