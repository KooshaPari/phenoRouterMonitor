//! Canonical structured errors for Phenotype and AgilePlus adapters.

use std::fmt;

/// Primary error type for cross-crate infrastructure and contract boundaries.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ErrorKind {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("io error: {0}")]
    Io(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("storage error: {0}")]
    Storage(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("validation error: {0}")]
    Validation(String),

    #[error("internal error: {0}")]
    Internal(String),

    #[error("connection error: {0}")]
    Connection(String),
}

impl ErrorKind {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn io(e: impl std::error::Error) -> Self {
        Self::Io(e.to_string())
    }

    pub fn serialization(msg: impl Into<String>) -> Self {
        Self::Serialization(msg.into())
    }

    pub fn storage(msg: impl Into<String>) -> Self {
        Self::Storage(msg.into())
    }

    pub fn conflict(msg: impl Into<String>) -> Self {
        Self::Conflict(msg.into())
    }

    pub fn validation(msg: impl Into<String>) -> Self {
        Self::Validation(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }

    pub fn connection(msg: impl Into<String>) -> Self {
        Self::Connection(msg.into())
    }

    /// Stable short name for logging and tests (for example `"NotFound"`).
    pub fn kind(&self) -> &'static str {
        match self {
            Self::NotFound(_) => "NotFound",
            Self::Io(_) => "Io",
            Self::Serialization(_) => "Serialization",
            Self::Storage(_) => "Storage",
            Self::Conflict(_) => "Conflict",
            Self::Validation(_) => "Validation",
            Self::Internal(_) => "Internal",
            Self::Connection(_) => "Connection",
        }
    }

    pub fn chain(self, ctx: impl Into<String>) -> ErrorContext {
        ErrorContext {
            inner: self,
            context: ctx.into(),
        }
    }
}

impl From<std::io::Error> for ErrorKind {
    fn from(e: std::io::Error) -> Self {
        if e.kind() == std::io::ErrorKind::NotFound {
            Self::not_found(e.to_string())
        } else {
            Self::io(e)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErrorContext {
    pub inner: ErrorKind,
    pub context: String,
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} — {}", self.context, self.inner)
    }
}

impl std::error::Error for ErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.inner)
    }
}

pub type Error = ErrorKind;

pub type ErrorKindInner = ErrorKind;

pub trait ErrorExt {
    fn chain(self, ctx: impl Into<String>) -> ErrorContext;
}

impl ErrorExt for ErrorKind {
    fn chain(self, ctx: impl Into<String>) -> ErrorContext {
        self.chain(ctx)
    }
}

pub type Result<T> = std::result::Result<T, ErrorKind>;
