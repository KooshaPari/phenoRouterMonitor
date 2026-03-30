//! HTTP transport error types.

use thiserror::Error;

/// Unified error type for HTTP transport operations.
#[derive(Debug, Error)]
pub enum TransportError {
    #[error("request failed: {0}")]
    Request(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("authentication error: {0}")]
    Authentication(String),

    #[error("auth token validation failed: {0}")]
    Auth(String),

    #[error("rate limited: retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    #[error("server error: {status} - {message}")]
    Server { status: u16, message: String },

    #[error("not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("unknown: {0}")]
    Unknown(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl TransportError {
    /// Check if this error should trigger a retry.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            TransportError::Timeout(_)
                | TransportError::Connection(_)
                | TransportError::RateLimited { .. }
                | TransportError::Server {
                    status: 429 | 500..=599,
                    ..
                }
        )
    }

    /// Get the error kind for categorization.
    pub fn kind(&self) -> ErrorKind {
        match self {
            TransportError::Request(_) => ErrorKind::Request,
            TransportError::Timeout(_) => ErrorKind::Timeout,
            TransportError::Connection(_) => ErrorKind::Connection,
            TransportError::Authentication(_) => ErrorKind::Authentication,
            TransportError::Auth(_) => ErrorKind::Auth,
            TransportError::RateLimited { .. } => ErrorKind::RateLimited,
            TransportError::Server { .. } => ErrorKind::Server,
            TransportError::NotFound(_) => ErrorKind::NotFound,
            TransportError::Serialization(_) => ErrorKind::Serialization,
            TransportError::Unknown(_) => ErrorKind::Unknown,
            TransportError::Io(_) => ErrorKind::Io,
        }
    }
}

/// Error kind categorization.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorKind {
    Request,
    Timeout,
    Connection,
    Authentication,
    Auth,
    RateLimited,
    Server,
    NotFound,
    Serialization,
    Unknown,
    Io,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn auth_error_creation() {
        let err = TransportError::Auth("invalid token".into());
        assert_eq!(err.kind(), ErrorKind::Auth);
        assert!(!err.is_retryable());
    }

    #[test]
    fn timeout_is_retryable() {
        let err = TransportError::Timeout("5 seconds".into());
        assert!(err.is_retryable());
    }

    #[test]
    fn not_found_not_retryable() {
        let err = TransportError::NotFound("resource".into());
        assert!(!err.is_retryable());
    }

    #[test]
    fn authentication_not_retryable() {
        let err = TransportError::Authentication("invalid".into());
        assert!(!err.is_retryable());
    }

    #[test]
    fn rate_limit_retryable() {
        let err = TransportError::RateLimited { retry_after: 60 };
        assert!(err.is_retryable());
    }

    #[test]
    fn server_5xx_retryable() {
        let err = TransportError::Server {
            status: 503,
            message: "service unavailable".into(),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn server_429_retryable() {
        let err = TransportError::Server {
            status: 429,
            message: "too many requests".into(),
        };
        assert!(err.is_retryable());
    }

    #[test]
    fn server_4xx_not_retryable() {
        let err = TransportError::Server {
            status: 400,
            message: "bad request".into(),
        };
        assert!(!err.is_retryable());
    }

    #[test]
    fn error_kind_consistency() {
        let errors = vec![
            TransportError::Request("req".into()),
            TransportError::Timeout("tm".into()),
            TransportError::Connection("conn".into()),
            TransportError::Authentication("auth".into()),
            TransportError::Auth("auth_token".into()),
            TransportError::RateLimited { retry_after: 10 },
            TransportError::Server {
                status: 500,
                message: "err".into(),
            },
            TransportError::NotFound("nf".into()),
            TransportError::Serialization("ser".into()),
            TransportError::Unknown("unk".into()),
        ];

        for err in errors {
            let kind = err.kind();
            assert_ne!(kind, ErrorKind::Io, "should not be Io kind");
            // Verify that the kind matches the error type
            let kind_str = format!("{:?}", kind);
            let err_str = format!("{:?}", err);
            assert!(err_str.contains(&kind_str.split("(").next().unwrap()),
                "error type should match kind");
        }
    }
}
