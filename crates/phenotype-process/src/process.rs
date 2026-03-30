//! Process execution implementation.
//!
//! Provides synchronous and asynchronous command execution with proper error handling.

use phenotype_error_core::ErrorKind;
use std::process::{Command, Stdio};

/// Process executor for running commands synchronously and asynchronously.
///
/// This struct provides methods to execute external commands and capture their output.
#[derive(Debug, Clone)]
pub struct ProcessExecutor;

impl ProcessExecutor {
    /// Synchronously execute a command and return its output.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to execute (e.g., "echo", "ls", "cargo")
    /// * `args` - Command arguments as a slice of string references
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the command output as a string on success,
    /// or an `ErrorKind` on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The command cannot be found (NotFound)
    /// - The command execution fails (Internal)
    /// - The output cannot be decoded as UTF-8 (Internal)
    /// - Permission to execute is denied (PermissionDenied)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use phenotype_process::ProcessExecutor;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ProcessExecutor::run_command("echo", &["hello", "world"])?;
    /// assert!(output.contains("hello"));
    /// # Ok(())
    /// # }
    /// ```
    pub fn run_command(cmd: &str, args: &[&str]) -> Result<String, ErrorKind> {
        let mut command = Command::new(cmd);
        command
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        let output = command
            .output()
            .map_err(|e| match e.kind() {
                std::io::ErrorKind::NotFound => {
                    ErrorKind::not_found(format!("command not found: {}", cmd))
                }
                std::io::ErrorKind::PermissionDenied => {
                    ErrorKind::permission_denied(format!("permission denied to execute: {}", cmd))
                }
                _ => ErrorKind::internal(format!("failed to execute command '{}': {}", cmd, e)),
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(ErrorKind::internal(format!(
                "command '{}' exited with status {}: {}",
                cmd,
                output.status.code().unwrap_or(-1),
                stderr
            )));
        }

        String::from_utf8(output.stdout).map_err(|e| {
            ErrorKind::internal(format!(
                "failed to decode output from command '{}': {}",
                cmd, e
            ))
        })
    }

    /// Asynchronously execute a command and return its output.
    ///
    /// This method uses `tokio::task::spawn_blocking` to execute the command
    /// without blocking the async runtime.
    ///
    /// # Arguments
    ///
    /// * `cmd` - The command to execute (e.g., "echo", "ls", "cargo")
    /// * `args` - Command arguments as a slice of string references
    ///
    /// # Returns
    ///
    /// Returns a `Future` that resolves to a `Result` containing the command output
    /// as a string on success, or an `ErrorKind` on failure.
    ///
    /// # Errors
    ///
    /// This function will return an error if:
    /// - The command cannot be found (NotFound)
    /// - The command execution fails (Internal)
    /// - The output cannot be decoded as UTF-8 (Internal)
    /// - Permission to execute is denied (PermissionDenied)
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use phenotype_process::ProcessExecutor;
    ///
    /// # #[tokio::main]
    /// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let output = ProcessExecutor::run_async("echo", &["hello", "world"]).await?;
    /// assert!(output.contains("hello"));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn run_async(cmd: &str, args: &[&str]) -> Result<String, ErrorKind> {
        let cmd = cmd.to_string();
        let args: Vec<String> = args.iter().map(|s| s.to_string()).collect();

        tokio::task::spawn_blocking(move || {
            let mut command = Command::new(&cmd);
            command
                .args(&args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());

            let output = command
                .output()
                .map_err(|e| match e.kind() {
                    std::io::ErrorKind::NotFound => {
                        ErrorKind::not_found(format!("command not found: {}", cmd))
                    }
                    std::io::ErrorKind::PermissionDenied => {
                        ErrorKind::permission_denied(format!(
                            "permission denied to execute: {}",
                            cmd
                        ))
                    }
                    _ => ErrorKind::internal(format!(
                        "failed to execute command '{}': {}",
                        cmd, e
                    )),
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ErrorKind::internal(format!(
                    "command '{}' exited with status {}: {}",
                    cmd,
                    output.status.code().unwrap_or(-1),
                    stderr
                )));
            }

            String::from_utf8(output.stdout).map_err(|e| {
                ErrorKind::internal(format!(
                    "failed to decode output from command '{}': {}",
                    cmd, e
                ))
            })
        })
        .await
        .map_err(|e| ErrorKind::internal(format!("async task failed: {}", e)))?
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_command_echo() {
        let result = ProcessExecutor::run_command("echo", &["hello", "world"]);
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("hello"));
        assert!(output.contains("world"));
    }

    #[test]
    fn test_run_command_with_no_args() {
        let result = ProcessExecutor::run_command("echo", &[]);
        assert!(result.is_ok());
    }

    #[test]
    fn test_run_command_failure() {
        let result = ProcessExecutor::run_command("false", &[]);
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e.kind(), "Internal");
            }
            _ => panic!("expected error"),
        }
    }

    #[test]
    fn test_run_command_not_found() {
        let result = ProcessExecutor::run_command("nonexistent_command_xyz", &[]);
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e.kind(), "NotFound");
            }
            _ => panic!("expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_run_async_echo() {
        let result = ProcessExecutor::run_async("echo", &["async", "test"]).await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("async"));
        assert!(output.contains("test"));
    }

    #[tokio::test]
    async fn test_run_async_not_found() {
        let result = ProcessExecutor::run_async("nonexistent_command_xyz", &[]).await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e.kind(), "NotFound");
            }
            _ => panic!("expected NotFound error"),
        }
    }

    #[tokio::test]
    async fn test_run_async_failure() {
        let result = ProcessExecutor::run_async("false", &[]).await;
        assert!(result.is_err());
        match result {
            Err(e) => {
                assert_eq!(e.kind(), "Internal");
            }
            _ => panic!("expected error"),
        }
    }
}
