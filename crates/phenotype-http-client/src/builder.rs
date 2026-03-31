//! HTTP client builder with retry and timeout configuration.

use reqwest::{Client, ClientBuilder};
use std::time::Duration;

/// Configuration for exponential backoff retry.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub base_delay: Duration,
    pub max_delay: Duration,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            base_delay: Duration::from_millis(200),
            max_delay: Duration::from_secs(10),
        }
    }
}

impl RetryConfig {
    pub fn new(max_retries: u32, base_delay_ms: u64, max_delay_ms: u64) -> Self {
        Self {
            max_retries,
            base_delay: Duration::from_millis(base_delay_ms),
            max_delay: Duration::from_millis(max_delay_ms),
        }
    }

    /// Calculate delay for a given retry attempt.
    pub fn delay_for(&self, attempt: u32) -> Duration {
        let exponential = 2u64.saturating_pow(attempt.min(10));
        let delay = self.base_delay.as_millis() as u64 * exponential;
        Duration::from_millis(delay.min(self.max_delay.as_millis()))
    }
}

/// Builder for creating configured HTTP clients.
#[derive(Debug, Default)]
pub struct ClientBuilder {
    timeout: Option<Duration>,
    connect_timeout: Option<Duration>,
    retry_config: Option<RetryConfig>,
}

impl ClientBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Set request timeout.
    pub fn timeout(mut self, duration: Duration) -> Self {
        self.timeout = Some(duration);
        self
    }

    /// Set connection timeout.
    pub fn connect_timeout(mut self, duration: Duration) -> Self {
        self.connect_timeout = Some(duration);
        self
    }

    /// Configure retry behavior.
    pub fn retry(mut self, config: RetryConfig) -> Self {
        self.retry_config = Some(config);
        self
    }

    /// Build the HTTP client.
    pub fn build(&self) -> Result<Client, reqwest::Error> {
        let mut builder = Client::builder();

        if let Some(t) = self.timeout {
            builder = builder.timeout(t);
        }

        if let Some(t) = self.connect_timeout {
            builder = builder.connect_timeout(t);
        }

        builder.build()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_config_delay() {
        let config = RetryConfig::default();
        assert_eq!(config.delay_for(0), Duration::from_millis(200));
        assert_eq!(config.delay_for(1), Duration::from_millis(400));
        assert_eq!(config.delay_for(2), Duration::from_millis(800));
    }

    #[test]
    fn test_retry_config_max_delay() {
        let config = RetryConfig::new(5, 1000, 2000);
        assert_eq!(config.delay_for(10), Duration::from_secs(2));
    }
}
