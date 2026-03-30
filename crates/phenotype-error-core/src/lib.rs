//! # phenotype-error-core
//! 
//! Centralized error types for Phenotype ecosystem.
//! 
//! ## Features
//! 
//! - **Error Hierarchy**: Source, Domain, Application, Infrastructure errors
//! - **Retryable Trait**: Automatic retry policy detection  
//! - **Context Propagation**: Structured error context with spans
//! 
//! ## Usage
//! 
//! ```rust
//! use phenotype_error_core::prelude::*;
//! 
//! #[derive(Debug, thiserror::Error)]
//! pub enum MyError {
//!     #[error("not found: {entity}:{id}")]
//!     NotFound { entity: &'static str, id: String },
//! }
//! 
//! impl Retryable for MyError {
//!     fn retry_policy(&self) -> RetryPolicy {
//!         RetryPolicy::None
//!     }
//! }
//! ```

pub mod error;
pub mod retry;
pub mod context;

pub use error::Error;
pub use retry::{RetryPolicy, Retryable, RetryStrategy};
pub use context::{Context, Span, ContextExt};

// Re-exports
pub use thiserror::Error;
pub use anyhow::{Result, Context as AnyhowContext};

// ============================================================================
// Core Error Type
// ============================================================================

/// Core error type for Phenotype ecosystem
#[derive(Debug, Error)]
pub enum Error {
    // ===== Source Errors =====
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    
    // ===== Domain Errors =====
    #[error("not found: {entity}:{id}")]
    NotFound {
        entity: &'static str,
        id: String,
    },
    
    #[error("validation failed for '{field}': {message}")]
    Validation {
        field: String,
        message: String,
    },
    
    #[error("conflict: {reason}")]
    Conflict {
        reason: String,
    },
    
    // ===== Application Errors =====
    #[error("configuration error: {message}")]
    Config {
        message: String,
    },
    
    #[error("unauthorized: {reason}")]
    Unauthorized {
        reason: String,
    },
    
    #[error("forbidden: {action} on {resource}")]
    Forbidden {
        action: &'static str,
        resource: String,
    },
    
    // ===== Infrastructure Errors =====
    #[error("database error: {message}")]
    Database {
        message: String,
    },
    
    #[error("cache error: {message}")]
    Cache {
        message: String,
    },
    
    #[error("queue error: {message}")]
    Queue {
        message: String,
    },
    
    #[error("external service error: {service}: {message}")]
    ExternalService {
        service: &'static str,
        message: String,
    },
    
    #[error("timeout: {operation} after {duration:?}")]
    Timeout {
        operation: String,
        duration: std::time::Duration,
    },
    
    #[error("rate limit exceeded: {limit} per {window:?}")]
    RateLimit {
        limit: u32,
        window: std::time::Duration,
    },
}

impl Error {
    /// Create a not found error
    pub fn not_found(entity: &'static str, id: impl Into<String>) -> Self {
        Self::NotFound {
            entity,
            id: id.into(),
        }
    }
    
    /// Create a validation error
    pub fn validation(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::Validation {
            field: field.into(),
            message: message.into(),
        }
    }
    
    /// Create a database error
    pub fn database(message: impl Into<String>) -> Self {
        Self::Database {
            message: message.into(),
        }
    }
}

impl Retryable for Error {
    fn retry_policy(&self) -> RetryPolicy {
        match self {
            Self::NotFound { .. } => RetryPolicy::None,
            Self::Validation { .. } => RetryPolicy::None,
            Self::Conflict { .. } => RetryPolicy::None,
            Self::Unauthorized { .. } => RetryPolicy::None,
            Self::Forbidden { .. } => RetryPolicy::None,
            Self::ExternalService { .. } => RetryPolicy::Exponential {
                max_attempts: 3,
                base: std::time::Duration::from_millis(100),
                max_delay: std::time::Duration::from_secs(10),
            },
            Self::Database { .. } => RetryPolicy::Fixed {
                max_attempts: 3,
                delay: std::time::Duration::from_millis(50),
            },
            Self::Cache { .. } => RetryPolicy::Fixed {
                max_attempts: 2,
                delay: std::time::Duration::from_millis(10),
            },
            Self::Queue { .. } => RetryPolicy::Linear {
                max_attempts: 5,
                step: std::time::Duration::from_millis(100),
            },
            Self::Io { source } if source.kind() == std::io::ErrorKind::WouldBlock => RetryPolicy::Exponential {
                max_attempts: 3,
                base: std::time::Duration::from_millis(100),
                max_delay: std::time::Duration::from_secs(5),
            },
            _ => RetryPolicy::Fixed {
                max_attempts: 1,
                delay: std::time::Duration::from_millis(100),
            },
        }
    }
}

// ============================================================================
// Prelude
// ============================================================================

/// Prelude for common imports
pub mod prelude {
    pub use super::Error;
    pub use super::RetryPolicy;
    pub use super::Retryable;
    pub use super::RetryStrategy;
    pub use super::Context;
    pub use super::Span;
    pub use super::ContextExt;
    pub use super::thiserror::Error;
    pub use anyhow::{Result, Context as AnyhowContext};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_not_found() {
        let err = Error::not_found("user", "123");
        assert!(matches!(err, Error::NotFound { entity: "user", id } if id == "123"));
    }
    
    #[test]
    fn test_validation() {
        let err = Error::validation("email", "invalid format");
        assert!(matches!(err, Error::Validation { field, message } if field == "email" && message == "invalid format"));
    }
    
    #[test]
    fn test_retry_policy() {
        assert!(matches!(Error::not_found("user", "123").retry_policy(), RetryPolicy::None));
        assert!(matches!(Error::database("conn failed").retry_policy(), RetryPolicy::Fixed { max_attempts: 3, .. }));
    }
    
    #[test]
    fn test_context() {
        let err = Error::not_found("user", "123");
        let ctx = Context::new(err)
            .with_span("auth", "verify_user")
            .with_metadata("request_id", "abc123");
        
        assert_eq!(ctx.spans.len(), 1);
        assert_eq!(ctx.metadata.get("request_id"), Some(&"abc123".to_string()));
    }
}
