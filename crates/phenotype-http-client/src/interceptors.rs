//! Request/response interceptors for logging and metrics.

use std::time::Instant;
use tracing::{info_span, instrument, Span};

/// Middleware for request/response logging.
#[derive(Debug, Default)]
pub struct LoggingInterceptor;

impl LoggingInterceptor {
    pub fn new() -> Self {
        Self::default()
    }

    /// Start a span for the request.
    pub fn start_span(&self, method: &str, url: &str) -> Span {
        info_span!("http_request", method = %method, url = %url)
    }
}

/// Timing middleware.
pub struct TimingInterceptor {
    start: Instant,
}

impl TimingInterceptor {
    pub fn new() -> Self {
        Self { start: Instant::now() }
    }

    /// Get elapsed time since creation.
    pub fn elapsed(&self) -> std::time::Duration {
        self.start.elapsed()
    }
}

impl Default for TimingInterceptor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_timing_interceptor() {
        let interceptor = TimingInterceptor::new();
        std::thread::sleep(std::time::Duration::from_millis(10));
        assert!(interceptor.elapsed().as_millis() >= 10);
    }
}
