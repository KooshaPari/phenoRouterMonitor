//! Canonical error kinds for the Phenotype ecosystem.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Error as IoError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum ErrorKindInner {
    NotFound,
    Serialization,
    Validation,
    Internal,
    Io,
    Storage,
    Connection,
    Conflict,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorKind {
    inner: ErrorKindInner,
    message: String,
}

impl ErrorKind {
    #[must_use]
    pub fn not_found(resource: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::NotFound,
            message: resource.into(),
        }
    }

    #[must_use]
    pub fn serialization(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Serialization,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn validation(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Validation,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn internal(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Internal,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn storage(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Storage,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn connection(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Connection,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn conflict(message: impl Into<String>) -> Self {
        Self {
            inner: ErrorKindInner::Conflict,
            message: message.into(),
        }
    }

    #[must_use]
    pub fn io(err: IoError) -> Self {
        Self {
            inner: ErrorKindInner::Io,
            message: err.to_string(),
        }
    }

    #[must_use]
    pub fn kind(&self) -> &'static str {
        match self.inner {
            ErrorKindInner::NotFound => "NotFound",
            ErrorKindInner::Serialization => "Serialization",
            ErrorKindInner::Validation => "Validation",
            ErrorKindInner::Internal => "Internal",
            ErrorKindInner::Io => "Io",
            ErrorKindInner::Storage => "Storage",
            ErrorKindInner::Connection => "Connection",
            ErrorKindInner::Conflict => "Conflict",
        }
    }

    #[must_use]
    pub fn inner(&self) -> ErrorKindInner {
        self.inner
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn chain(self, context: impl Into<String>) -> ErrorContext {
        ErrorContext {
            context: context.into(),
            source: self,
        }
    }
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let label = match self.inner {
            ErrorKindInner::NotFound => "not found",
            ErrorKindInner::Serialization => "serialization error",
            ErrorKindInner::Validation => "validation error",
            ErrorKindInner::Internal => "internal error",
            ErrorKindInner::Io => "io error",
            ErrorKindInner::Storage => "storage error",
            ErrorKindInner::Connection => "connection error",
            ErrorKindInner::Conflict => "conflict",
        };
        write!(f, "{label}: {}", self.message)
    }
}

impl std::error::Error for ErrorKind {}

impl From<IoError> for ErrorKind {
    fn from(err: IoError) -> Self {
        if err.kind() == std::io::ErrorKind::NotFound {
            Self::not_found(err.to_string())
        } else {
            Self::io(err)
        }
    }
}

impl From<serde_json::Error> for ErrorKind {
    fn from(err: serde_json::Error) -> Self {
        Self::serialization(err.to_string())
    }
}

impl From<String> for ErrorKind {
    fn from(message: String) -> Self {
        Self::internal(message)
    }
}

impl From<&str> for ErrorKind {
    fn from(message: &str) -> Self {
        Self::internal(message.to_string())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ErrorContext {
    context: String,
    source: ErrorKind,
}

impl ErrorContext {
    #[must_use]
    pub fn source(&self) -> &ErrorKind {
        &self.source
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.context, self.source)
    }
}

impl std::error::Error for ErrorContext {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

pub type Error = ErrorKind;

pub trait ErrorExt: Sized {
    #[must_use]
    fn into_error_kind(self) -> ErrorKind;
}

impl ErrorExt for ErrorKind {
    fn into_error_kind(self) -> ErrorKind {
        self
    }
}

impl ErrorExt for ErrorContext {
    fn into_error_kind(self) -> ErrorKind {
        self.source
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn not_found_kind_and_display() {
        let err = ErrorKind::not_found("user/42");
        assert_eq!(err.kind(), "NotFound");
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn io_not_found_maps_to_not_found() {
        let io_err = IoError::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ErrorKind = io_err.into();
        assert_eq!(err.kind(), "NotFound");
    }

    #[test]
    fn chain_includes_context() {
        let err = ErrorKind::not_found("user");
        let ctx = err.chain("while fetching");
        assert!(ctx.to_string().contains("while fetching"));
    }
}
