//! Integration tests for the ProcessExecutor.
//!
//! These tests verify real command execution and output handling.

use phenotype_process::ProcessExecutor;

#[test]
fn test_echo_command() {
    let result = ProcessExecutor::run_command("echo", &["test_output"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("test_output"));
}

#[test]
fn test_echo_multiword_args() {
    let result = ProcessExecutor::run_command("echo", &["hello", "from", "rust"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("hello"));
    assert!(output.contains("from"));
    assert!(output.contains("rust"));
}

#[test]
fn test_command_with_special_chars() {
    let result = ProcessExecutor::run_command("echo", &["test@123!#"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("test@123!#"));
}

#[test]
fn test_nonexistent_command_error() {
    let result = ProcessExecutor::run_command("definitely_not_a_real_command_xyz", &[]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), "NotFound");
    assert!(err.to_string().contains("command not found"));
}

#[test]
fn test_command_failure_stderr() {
    let result = ProcessExecutor::run_command("sh", &["-c", "echo error >&2; exit 1"]);
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), "Internal");
    assert!(err.to_string().contains("exited with status 1"));
}

#[tokio::test]
async fn test_async_echo_command() {
    let result = ProcessExecutor::run_async("echo", &["async_output"]).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("async_output"));
}

#[tokio::test]
async fn test_async_multiword_args() {
    let result = ProcessExecutor::run_async("echo", &["async", "test", "words"]).await;
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(output.contains("async"));
    assert!(output.contains("test"));
}

#[tokio::test]
async fn test_async_nonexistent_command() {
    let result = ProcessExecutor::run_async("nonexistent_cmd_xyz", &[]).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), "NotFound");
}

#[tokio::test]
async fn test_async_command_failure() {
    let result = ProcessExecutor::run_async("sh", &["-c", "exit 42"]).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert_eq!(err.kind(), "Internal");
    assert!(err.to_string().contains("exited with status 42"));
}

#[test]
#[cfg(unix)]
fn test_ls_command() {
    let result = ProcessExecutor::run_command("ls", &["-1"]);
    assert!(result.is_ok());
    let output = result.unwrap();
    assert!(!output.is_empty());
}
