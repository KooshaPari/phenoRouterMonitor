//! # phenotype-error-core
//!
//! Centralized error types for Phenotype ecosystem.
//!
//! ## Features
//!
//! - **Error Hierarchy**: Source, Domain, Application, Infrastructure errors
//! - **Retryable Trait**: Automatic retry policy detection
//! - **Context Propagation**: Structured error context with spans
//! - **Derive Macros**: #[from], #[context], #[retry]
//!
//! ## Quick Start
//!
//! ```rust
//! use phenotype_error::prelude::*;
//!
//! #[derive(Debug, Error)]
//! pub enum MyError {
//!     #[error("entity not found: {entity}:{id}")]
//!     NotFound { entity: &'static str, id: String },
//!     
//!     #[error("validation failed: {field}")]
//!     Validation { field: String, #[from] source: serde_json::Error },
//! }
//!
//! impl Retryable for MyError {
//!     fn retry_policy(&self) -> RetryPolicy {
//!         match self {
//!             MyError::NotFound { .. } => RetryPolicy::None,
//!             MyError::Validation { .. } => RetryPolicy::None,
//!             _ => RetryPolicy::Fixed { max_attempts: 3, delay: Duration::from_millis(100) },
//!         }
//!     }
//! }
//! ```

pub mod error;
pub mod retry;
pub mod context;

pub use error::{Error, Source, Domain, Application, Infrastructure};
pub use retry::{RetryPolicy, Retryable, RetryStrategy};
pub use context::{Context, ContextExt, Span};

// Re-exports for convenience
pub use thiserror::Error;
pub use anyhow::{Result, Context as AnyhowContext};

// ============================================================================
// Error Types
// ============================================================================

/// Core error type for Phenotype ecosystem
///
/// # Variants
///
/// - **Source**: Standard library errors (IO, Parse, Network)
/// - **Domain**: Business logic errors (NotFound, Validation, Conflict)
/// - **Application**: Application-level errors (Config, Auth, Permission)
/// - **Infrastructure**: Infrastructure errors (Database, Cache, Queue)
#[derive(Debug, Error)]
pub enum CoreError {
    // ===== Source Errors =====
    /// IO error from filesystem or network
    #[error("IO error: {source}")]
    Io {
        #[from]
        source: std::io::Error,
    },
    
    /// JSON parse/serialize error
    #[error("JSON error: {source}")]
    Json {
        #[from]
        source: serde_json::Error,
    },
    
    /// TOML parse/serialize error
    #[error("TOML error: {source}")]
    Toml {
        #[from]
        source: toml::de::Error,
    },
    
    // ===== Domain Errors =====
    /// Entity not found
    #[error("not found: {entity}:{id}")]
    NotFound {
        entity: &'static str,
        id: String,
    },
    
    /// Validation failed
    #[error("validation failed for '{field}': {message}")]
    Validation {
        field: String,
        message: String,
    },
    
    /// Conflict (duplicate, version mismatch)
    #[error("conflict: {reason}")]
    Conflict { reason: String },
    
    /// Business rule violation
    #[error("business rule violated: {rule}")]
    BusinessRule {
        rule: &'static str,
        details: Option<String>,
    },
    
    // ===== Application Errors =====
    /// Configuration error
    #[error("configuration error: {message}")]
    Config {
        message: String,
        #[from]
        source: std::io::Error,
    },
    
    /// Authentication required
    #[error("unauthorized: {reason}")]
    Unauthorized { reason: String },
    
    /// Permission denied
    #[error("forbidden: {action} on {resource}")]
    Forbidden {
        action: &'static str,
        resource: String,
    },
    
    // ===== Infrastructure Errors =====
    /// Database error
    #[error("database error: {message}")]
    Database { message: String },
    
    /// Cache error
    #[error("cache error: {message}")]
    Cache { message: String },
    
    /// Message queue error
    #[error("queue error: {message}")]
    Queue { message: String },
    
    /// External service error
    #[error("external service error: {service}: {message}")]
    ExternalService {
        service: &'static str,
        message: String,
    },
    
    /// Timeout error
    #[error("operation timed out after {duration:?}")]
    Timeout {
        operation: String,
        duration: std::time::Duration,
    },
    
    /// Rate limit exceeded
    #[error("rate limit exceeded: {limit} requests per {window:?}")]
    RateLimit {
        limit: u32,
        window: std::time::Duration,
    },
}

impl CoreError {
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
    
    /// Create an external service error
    pub fn external(service: &'static str, message: impl Into<String>) -> Self {
        Self::ExternalService {
            service,
            message: message.into(),
        }
    }
}

impl Retryable for CoreError {
    fn retry_policy(&self) -> RetryPolicy {
        match self {
            // Non-retryable
            Self::NotFound { .. } => RetryPolicy::None,
            Self::Validation { .. } => RetryPolicy::None,
            Self::Conflict { .. } => RetryPolicy::None,
            Self::BusinessRule { .. } => RetryPolicy::None,
            Self::Unauthorized { .. } => RetryPolicy::None,
            Self::Forbidden { .. } => RetryPolicy::None,
            
            // Retryable with exponential backoff
            Self::Io { source } if source.kind() == std::io::ErrorKind::WouldBlock => {
                RetryPolicy::Exponential {
                    max_attempts: 3,
                    base: std::time::Duration::from_millis(100),
                    max_delay: std::time::Duration::from_secs(5),
                }
            }
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
            
            // Default retryable
            _ => RetryPolicy::Fixed {
                max_attempts: 1,
                delay: std::time::Duration::from_millis(100),
            },
        }
    }
}

// ============================================================================
// Context Propagation
// ============================================================================

/// Error context with tracing information
#[derive(Debug, Clone)]
pub struct ErrorContext<E> {
    /// The underlying error
    pub error: E,
    /// Stack trace spans
    pub spans: Vec<Span>,
    /// Error timestamps
    pub timestamps: Vec<chrono::DateTime<chrono::Utc>>,
    /// Additional metadata
    pub metadata: std::collections::HashMap<String, String>,
}

impl<E: std::fmt::Display> std::fmt::Display for ErrorContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)?;
        for span in &self.spans {
            write!(f, "\n  at {}", span)?;
        }
        if !self.metadata.is_empty() {
            write!(f, "\n  metadata: {:?}", self.metadata)?;
        }
        Ok(())
    }
}

impl<E: std::fmt::Debug> std::fmt::Debug for ErrorContext<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ErrorContext")
            .field("error", &self.error)
            .field("spans", &self.spans)
            .field("timestamps", &self.timestamps)
            .field("metadata", &self.metadata)
            .finish()
    }
}

impl<E> ErrorContext<E> {
    /// Create a new error context
    pub fn new(error: E) -> Self {
        Self {
            error,
            spans: Vec::new(),
            timestamps: vec![chrono::Utc::now()],
            metadata: std::collections::HashMap::new(),
        }
    }
    
    /// Add a span to the context
    pub fn with_span(mut self, span: Span) -> Self {
        self.spans.push(span);
        self
    }
    
    /// Add metadata to the context
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

impl<E> From<E> for ErrorContext<E> {
    fn from(error: E) -> Self {
        Self::new(error)
    }
}

// ============================================================================
// Prelude
// ============================================================================

/// Prelude module for common imports
pub mod prelude {
    pub use super::Error;
    pub use super::RetryPolicy;
    pub use super::Retryable;
    pub use super::RetryStrategy;
    pub use super::Context;
    pub use super::Span;
    pub use super::ErrorContext;
    pub use super::thiserror::Error;
    pub use super::anyhow::{Result, Context as AnyhowContext};
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_not_found() {
        let error = CoreError::not_found("user", "123");
        assert!(matches!(error, CoreError::NotFound { entity: "user", id: ref id } if id == "123"));
    }
    
    #[test]
    fn test_validation() {
        let error = CoreError::validation("email", "invalid format");
        assert!(matches!(error, CoreError::Validation { field, message } if field == "email" && message == "invalid format"));
    }
    
    #[test]
    fn test_retry_policy_not_found() {
        let error = CoreError::not_found("user", "123");
        assert!(matches!(error.retry_policy(), RetryPolicy::None));
    }
    
    #[test]
    fn test_retry_policy_database() {
        let error = CoreError::database("connection failed");
        assert!(matches!(error.retry_policy(), RetryPolicy::Fixed { max_attempts: 3, .. }));
    }
    
    #[test]
    fn test_context() {
        let error = CoreError::not_found("user", "123");
        let context = ErrorContext::new(error)
            .with_span(Span::new("auth", "verify_user"))
            .with_metadata("request_id", "abc123");
        
        assert!(context.spans.len() == 1);
        assert!(context.metadata.get("request_id") == Some(&"abc123".to_string()));
    }
}
