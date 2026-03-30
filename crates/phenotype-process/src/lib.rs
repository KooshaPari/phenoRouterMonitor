//! # Phenotype Process
//!
//! Process execution utilities with support for both synchronous and asynchronous command execution.
//!
//! ## Features
//!
//! - Synchronous command execution via `ProcessExecutor::run_command`
//! - Asynchronous command execution via `ProcessExecutor::run_async`
//! - Proper error handling with `phenotype-error-core` integration
//! - Support for command arguments and output capture
//!
//! ## Usage
//!
//! ```no_run
//! use phenotype_process::ProcessExecutor;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = ProcessExecutor::run_command("echo", &["hello"])?;
//! println!("Output: {}", output);
//! # Ok(())
//! # }
//! ```
//!
//! For async operations:
//!
//! ```no_run
//! use phenotype_process::ProcessExecutor;
//!
//! # #[tokio::main]
//! # async fn main() -> Result<(), Box<dyn std::error::Error>> {
//! let output = ProcessExecutor::run_async("echo", &["hello"]).await?;
//! println!("Output: {}", output);
//! # Ok(())
//! # }
//! ```

mod process;

pub use process::ProcessExecutor;
