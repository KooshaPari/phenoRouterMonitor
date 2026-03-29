//! Error types for process management.

use std::fmt;

/// Result type alias.
pub type Result<T> = std::result::Result<T, ProcessError>;

/// Process-related errors.
#[derive(Debug)]
pub enum ProcessError {
    /// Failed to spawn the process.
    Spawn {
        command: String,
        source: String,
    },

    /// IO error occurred.
    Io(String),

    /// Process already exited.
    AlreadyExited,

    /// PTY operation failed.
    Pty(String),

    /// Resize operation failed.
    Resize(String),

    /// Process was not found.
    NotFound(u32),

    /// Permission denied.
    PermissionDenied(String),

    /// Operation timed out.
    Timeout,
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spawn { command, source } => {
                write!(f, "Failed to spawn process '{}': {}", command, source)
            }
            Self::Io(s) => write!(f, "IO error: {}", s),
            Self::AlreadyExited => write!(f, "Process has already exited"),
            Self::Pty(s) => write!(f, "PTY error: {}", s),
            Self::Resize(s) => write!(f, "Resize error: {}", s),
            Self::NotFound(pid) => write!(f, "Process {} not found", pid),
            Self::PermissionDenied(s) => write!(f, "Permission denied: {}", s),
            Self::Timeout => write!(f, "Operation timed out"),
        }
    }
}

impl std::error::Error for ProcessError {}
