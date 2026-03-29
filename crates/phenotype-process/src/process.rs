//! Process configuration and utilities.

use std::path::PathBuf;

/// Configuration for spawning processes.
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Working directory for the process.
    pub cwd: Option<PathBuf>,
    /// Environment variables to set.
    pub env: std::collections::HashMap<String, String>,
    /// Whether to detach the process.
    pub detached: bool,
    /// PTY size if using PTY.
    pub pty_size: Option<super::pty::PtySize>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            cwd: None,
            env: std::collections::HashMap::new(),
            detached: false,
            pty_size: Some(super::pty::PtySize::default_size()),
        }
    }
}

impl ProcessConfig {
    /// Create a new configuration.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the working directory.
    pub fn with_cwd(mut self, cwd: impl Into<PathBuf>) -> Self {
        self.cwd = Some(cwd.into());
        self
    }

    /// Add an environment variable.
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set whether to detach the process.
    pub fn detached(mut self, detached: bool) -> Self {
        self.detached = detached;
        self
    }

    /// Set the PTY size.
    pub fn with_pty_size(mut self, size: super::pty::PtySize) -> Self {
        self.pty_size = Some(size);
        self
    }

    /// Disable PTY.
    pub fn no_pty(mut self) -> Self {
        self.pty_size = None;
        self
    }
}

/// Process information.
#[derive(Debug, Clone)]
pub struct ProcessInfo {
    /// Process ID.
    pub pid: u32,
    /// Parent process ID.
    pub ppid: u32,
    /// Process name.
    pub name: String,
    /// Command line arguments.
    pub args: Vec<String>,
    /// Working directory.
    pub cwd: Option<PathBuf>,
    /// Environment variables (filtered).
    pub env: std::collections::HashMap<String, String>,
}

impl ProcessInfo {
    /// Get the process ID as a string.
    pub fn pid_str(&self) -> String {
        self.pid.to_string()
    }

    /// Check if the process is the current process.
    pub fn is_current(&self) -> bool {
        self.pid == std::process::id()
    }
}

/// Signal for process control.
#[derive(Debug, Clone, Copy)]
pub enum Signal {
    /// Interrupt signal (SIGINT).
    Interrupt,
    /// Termination signal (SIGTERM).
    Terminate,
    /// Kill signal (SIGKILL).
    Kill,
    /// Hangup signal (SIGHUP).
    Hangup,
}

impl Signal {
    /// Send the signal to a process.
    #[cfg(unix)]
    pub fn send_to(&self, pid: u32) -> std::io::Result<()> {
        use libc::{kill, SIGINT, SIGTERM, SIGKILL, SIGHUP};

        let sig = match self {
            Self::Interrupt => SIGINT,
            Self::Terminate => SIGTERM,
            Self::Kill => SIGKILL,
            Self::Hangup => SIGHUP,
        };

        unsafe { kill(pid as libc::pid_t, sig) };
        Ok(())
    }

    #[cfg(not(unix))]
    pub fn send_to(&self, _pid: u32) -> std::io::Result<()> {
        Err(std::io::Error::new(
            std::io::ErrorKind::Unsupported,
            "Signals are not supported on this platform",
        ))
    }
}
