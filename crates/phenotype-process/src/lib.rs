//! # Phenotype Process Library
//!
//! Cross-platform process and PTY management for the Phenotype ecosystem.
//!
//! ## Features
//!
//! - PTY (Pseudo-Terminal) creation and management
//! - Process spawning with configurable options
//! - Window size control for PTY processes
//! - Cross-platform support (Unix and Windows)
//!
//! ## Usage
//!
//! ```rust
//! use phenotype_process::{PtyProcess, ProcessConfig};
//!
//! # async fn example() -> anyhow::Result<()> {
//! let config = ProcessConfig::default();
//! let mut pty = PtyProcess::spawn("bash", &[], &config)?;
//!
//! // Resize the terminal
//! pty.resize(80, 24)?;
//!
//! // Write to the PTY
//! pty.write(b"ls -la\n")?;
//!
//! // Read output
//! let output = pty.read_to_end().await?;
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use thiserror::Error;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::process::{Child, Command};
use tokio::sync::Mutex;

pub mod error;
pub mod pty;
pub mod process;

pub use error::{ProcessError, Result};
pub use pty::{PtyMaster, PtyPair, PtySize};
pub use process::ProcessConfig;

// Re-export for convenience
pub type PtyResult<T> = std::result::Result<T, ProcessError>;

// ----------------------------------------------------------------------------
// Main Entry Point
// ----------------------------------------------------------------------------

/// A managed process that can be controlled and monitored.
pub struct ManagedProcess {
    child: Option<Child>,
    pid: u32,
    name: String,
    metadata: HashMap<String, String>,
}

impl ManagedProcess {
    /// Create a new managed process from a spawned child.
    pub fn new(child: Child) -> Self {
        let pid = child.id().unwrap_or(0);
        Self {
            child: Some(child),
            pid,
            name: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Get the process ID.
    pub fn pid(&self) -> u32 {
        self.pid
    }

    /// Get the process name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Set the process name.
    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = name.into();
        self
    }

    /// Add metadata to the process.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Get metadata value.
    pub fn get_metadata(&self, key: &str) -> Option<&str> {
        self.metadata.get(key).map(|s| s.as_str())
    }

    /// Kill the process.
    pub async fn kill(&mut self) -> PtyResult<()> {
        if let Some(child) = self.child.take() {
            child.kill().await?;
        }
        Ok(())
    }

    /// Wait for the process to exit.
    pub async fn wait(&mut self) -> PtyResult<std::process::ExitStatus> {
        if let Some(child) = self.child.take() {
            child.wait().await.map_err(ProcessError::Io)
        } else {
            Err(ProcessError::AlreadyExited)
        }
    }

    /// Check if the process is still running.
    pub async fn is_running(&mut self) -> PtyResult<bool> {
        if let Some(ref mut child) = self.child {
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.child = None;
                    Ok(false)
                }
                Ok(None) => Ok(true),
                Err(e) => Err(ProcessError::Io(e.to_string())),
            }
        } else {
            Ok(false)
        }
    }
}

/// Builder for spawning processes.
pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    env: HashMap<String, String>,
    cwd: Option<PathBuf>,
    stdin: Stdio,
    stdout: Stdio,
    stderr: Stdio,
}

impl ProcessBuilder {
    /// Create a new process builder.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            args: Vec::new(),
            env: HashMap::new(),
            cwd: None,
            stdin: Stdio::piped(),
            stdout: Stdio::piped(),
            stderr: Stdio::piped(),
        }
    }

    /// Add an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Add multiple arguments.
    pub fn args(mut self, args: impl IntoIterator<Item = String>) -> Self {
        self.args.extend(args);
        self
    }

    /// Set an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set the working directory.
    pub fn cwd(mut self, path: impl Into<PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    /// Set stdin configuration.
    pub fn stdin(mut self, stdin: Stdio) -> Self {
        self.stdin = stdin;
        self
    }

    /// Set stdout configuration.
    pub fn stdout(mut self, stdout: Stdio) -> Self {
        self.stdout = stdout;
        self
    }

    /// Set stderr configuration.
    pub fn stderr(mut self, stderr: Stdio) -> Self {
        self.stderr = stderr;
        self
    }

    /// Spawn the process.
    pub async fn spawn(mut self) -> PtyResult<ManagedProcess> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        cmd.stdin(self.stdin);
        cmd.stdout(self.stdout);
        cmd.stderr(self.stderr);

        if let Some(cwd) = self.cwd.take() {
            cmd.current_dir(cwd);
        }

        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        let child = cmd.spawn().map_err(|e| ProcessError::Spawn {
            command: self.command.clone(),
            source: e.to_string(),
        })?;

        Ok(ManagedProcess::new(child))
    }
}

/// Spawn a process with the given command and arguments.
pub async fn spawn(
    command: impl Into<String>,
    args: impl IntoIterator<Item = String>,
) -> PtyResult<ManagedProcess> {
    ProcessBuilder::new(command).args(args).spawn().await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_spawn_echo() {
        let result = spawn("echo", ["hello", "world"]).await;
        assert!(result.is_ok());
        let mut process = result.unwrap();
        let status = process.wait().await;
        assert!(status.is_ok());
        assert!(status.unwrap().success());
    }

    #[tokio::test]
    async fn test_process_builder() {
        let process = ProcessBuilder::new("echo")
            .arg("test")
            .env("TEST_VAR", "test_value")
            .spawn()
            .await
            .unwrap();

        assert!(process.pid() > 0);
        let status = process.wait().await.unwrap();
        assert!(status.success());
    }
}
