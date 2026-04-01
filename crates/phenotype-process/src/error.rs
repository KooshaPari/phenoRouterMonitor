//! Error types for process and PTY operations.

use thiserror::Error;

/// Result type alias for process operations.
pub type Result<T> = std::result::Result<T, ProcessError>;

/// Errors that can occur during process operations.
#[derive(Debug, Error)]
#[error("process error: {0}")]
pub struct ProcessError {
    msg: String,
    #[from]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl ProcessError {
    /// Creates a new process error.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            source: None,
        }
    }

    /// Creates a new process error with a source.
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(msg: impl Into<String>, source: E) -> Self {
        Self {
            msg: msg.into(),
            source: Some(Box::new(source)),
        }
    }

    /// Returns the error message.
    pub fn msg(&self) -> &str {
        &self.msg
    }

    /// Returns the source error if any.
    pub fn source_error(&self) -> Option<&(dyn std::error::Error + Send + Sync + 'static)> {
        self.source.as_ref().map(|e| e.as_ref())
    }
}

impl From<std::io::Error> for ProcessError {
    fn from(e: std::io::Error) -> Self {
        Self::with_source("IO error", e)
    }
}

/// Result type alias for PTY operations.
pub type PtyResult<T> = std::result::Result<T, PtyError>;

/// Errors that can occur during PTY operations.
#[derive(Debug, Error)]
#[error("PTY error: {0}")]
pub struct PtyError {
    msg: String,
    #[from]
    source: Option<Box<dyn std::error::Error + Send + Sync>>,
}

impl PtyError {
    /// Creates a new PTY error.
    pub fn new(msg: impl Into<String>) -> Self {
        Self {
            msg: msg.into(),
            source: None,
        }
    }

    /// Creates a new PTY error with a source.
    pub fn with_source<E: std::error::Error + Send + Sync + 'static>(msg: impl Into<String>, source: E) -> Self {
        Self {
            msg: msg.into(),
            source: Some(Box::new(source)),
        }
    }
}

impl From<std::io::Error> for PtyError {
    fn from(e: std::io::Error) -> Self {
        Self::with_source("PTY IO error", e)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_error_new() {
        let err = ProcessError::new("test error");
        assert_eq!(err.msg(), "test error");
        assert!(err.source_error().is_none());
    }

    #[test]
    fn test_process_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err = ProcessError::from(io_err);
        assert!(err.msg().contains("IO error"));
        assert!(err.source_error().is_some());
    }

    #[test]
    fn test_pty_error_new() {
        let err = PtyError::new("test PTY error");
        assert_eq!(err.msg(), "test PTY error");
    }
}
