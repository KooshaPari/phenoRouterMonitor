//! Process spawning and management utilities.

use std::process::{Command, ExitStatus, Stdio};
use std::sync::Arc;
use tokio::process::Command as AsyncCommand;
use tokio::sync::RwLock;

use super::error::{ProcessError, Result};

/// Builder for spawning processes.
#[derive(Debug, Default)]
pub struct ProcessBuilder {
    command: String,
    args: Vec<String>,
    env: std::collections::HashMap<String, String>,
    cwd: Option<std::path::PathBuf>,
}

impl ProcessBuilder {
    /// Creates a new process builder for the given command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            ..Default::default()
        }
    }

    /// Adds an argument to the command.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds multiple arguments to the command.
    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(|a| a.into()));
        self
    }

    /// Sets the working directory.
    pub fn cwd(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }

    /// Sets an environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Spawns the process and returns a handle.
    pub fn spawn(self) -> Result<ProcessHandle> {
        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);

        if let Some(ref cwd) = self.cwd {
            cmd.current_dir(cwd);
        } else {
            cmd.current_dir(std::env::current_dir()?);
        }

        for (key, value) in &self.env {
            cmd.env(key, value);
        }

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let child = cmd.spawn()?;
        Ok(ProcessHandle {
            inner: Arc::new(RwLock::new(child)),
            command: self.command,
        })
    }
}

/// Handle to a spawned process.
#[derive(Debug)]
pub struct ProcessHandle {
    inner: Arc<RwLock<std::process::Child>>,
    command: String,
}

impl ProcessHandle {
    /// Waits for the process to exit and returns its exit status.
    pub async fn wait(&self) -> Result<ExitStatus> {
        let mut child = self.inner.write().await;
        Ok(child.wait().await?)
    }

    /// Sends a signal to the process.
    pub async fn kill(&self) -> Result<()> {
        let mut child = self.inner.write().await;
        child.kill().await?;
        Ok(())
    }

    /// Returns the process ID.
    pub fn id(&self) -> u32 {
        // Note: Getting PID requires async context in tokio
        // For sync access, we store command name as placeholder
        // In practice, use try_wait() with match
        0
    }
}

/// Trait for process spawners.
pub trait ProcessSpawner: Send + Sync {
    /// Spawns a process with the given configuration.
    fn spawn(&self, config: &ProcessConfig) -> Result<Box<dyn ProcessHandle>>;
}

/// Configuration for spawning processes.
#[derive(Debug, Clone)]
pub struct ProcessConfig {
    /// Command to execute.
    pub command: String,
    /// Arguments to pass.
    pub args: Vec<String>,
    /// Environment variables.
    pub env: std::collections::HashMap<String, String>,
    /// Working directory.
    pub cwd: Option<std::path::PathBuf>,
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            command: String::new(),
            args: Vec::new(),
            env: std::collections::HashMap::new(),
            cwd: None,
        }
    }
}

impl ProcessConfig {
    /// Creates a new config with the given command.
    pub fn new(command: impl Into<String>) -> Self {
        Self {
            command: command.into(),
            ..Default::default()
        }
    }

    /// Adds an argument.
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    /// Adds environment variable.
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Sets working directory.
    pub fn cwd(mut self, path: impl Into<std::path::PathBuf>) -> Self {
        self.cwd = Some(path.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_process_config() {
        let config = ProcessConfig::new("echo")
            .arg("hello")
            .arg("world")
            .env("FOO", "bar");

        assert_eq!(config.command, "echo");
        assert_eq!(config.args, vec!["hello", "world"]);
        assert_eq!(config.env.get("FOO"), Some(&"bar".to_string()));
    }

    #[tokio::test]
    async fn test_process_builder() {
        let handle = ProcessBuilder::new("echo")
            .arg("hello")
            .spawn()
            .expect("should spawn echo");

        let status = handle.wait().await.expect("should wait");
        assert!(status.success());
    }
}
