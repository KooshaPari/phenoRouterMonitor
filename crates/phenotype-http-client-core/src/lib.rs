//! # HTTP Client Core
//!
//! Core traits and utilities for HTTP clients in the Phenotype ecosystem.
//!
//! ## Features
//!
//! - `HttpTransport` - Core transport trait for HTTP clients
//! - `RetryPolicy` - Retry logic with backoff
//! - `TransportError` - Unified error types
//! - `AuthMiddleware` - Authentication middleware support
//!
//! ## Usage
//!
//! ```rust,ignore
//! use phenotype_http_client_core::{HttpTransport, RetryPolicy, TransportError};
//!
//! struct MyClient {
//!     base_url: url::Url,
//!     inner: reqwest::Client,
//! }
//!
//! #[async_trait::async_trait]
//! impl HttpTransport for MyClient {
//!     async fn request(&self, req: http::Request<Vec<u8>>) -> Result<http::Response<Vec<u8>>, TransportError> {
//!         // implementation
//!     }
//! }
//! ```

pub mod error;
pub mod retry;
pub mod transport;
pub mod auth;

pub use error::{TransportError, ErrorKind};
pub use retry::{RetryPolicy, Backoff};
pub use transport::HttpTransport;
pub use auth::{AuthMiddleware, BearerToken};

use thiserror::Error;

/// Result type for HTTP transport operations.
pub type Result<T> = std::result::Result<T, TransportError>;

/// Extension trait for adding retry logic to transports.
pub trait RetryExt: HttpTransport {
    /// Wrap this transport with retry logic.
    fn with_retry(self, policy: RetryPolicy) -> RetryTransport<Self>
    where
        Self: Sized,
    {
        RetryTransport {
            inner: self,
            policy,
        }
    }

    /// Wrap this transport with authentication.
    fn with_auth(self, auth: AuthMiddleware) -> AuthTransport<Self>
    where
        Self: Sized,
    {
        AuthTransport {
            inner: self,
            auth,
        }
    }
}

impl<T: HttpTransport> RetryExt for T {}

/// Transport wrapper that adds retry logic.
#[derive(Debug, Clone)]
pub struct RetryTransport<T> {
    inner: T,
    policy: RetryPolicy,
}

#[async_trait::async_trait]
impl<T: HttpTransport> HttpTransport for RetryTransport<T> {
    async fn request(
        &self,
        req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>> {
        let mut attempt = 0;
        let mut last_error = None;

        loop {
            match self.inner.request(req.clone()).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    attempt += 1;
                    last_error = Some(e);

                    if attempt >= self.policy.max_attempts {
                        break;
                    }

                    if !self.policy.should_retry(&e) {
                        break;
                    }

                    let delay = self.policy.backoff.calculate(attempt);
                    tokio::time::sleep(delay).await;
                }
            }
        }

        Err(last_error.unwrap_or_else(|| TransportError::Unknown("No error".into())))
    }
}

/// Transport wrapper that adds authentication.
#[derive(Debug, Clone)]
pub struct AuthTransport<T> {
    inner: T,
    auth: AuthMiddleware,
}

#[async_trait::async_trait]
impl<T: HttpTransport> HttpTransport for AuthTransport<T> {
    async fn request(
        &self,
        mut req: http::Request<Vec<u8>>,
    ) -> Result<http::Response<Vec<u8>>> {
        self.auth.apply(&mut req);
        self.inner.request(req).await
    }
}
