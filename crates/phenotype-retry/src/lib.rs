//! Phenotype Retry — Retry logic with exponential, linear, fixed, and fibonacci backoff strategies.
//!
//! This crate provides comprehensive retry functionality for async operations with configurable
//! backoff strategies, retry policies, and custom error predicates.
//!
//! # Features
//!
//! - Multiple backoff strategies: Fixed, Exponential, Linear, Fibonacci
//! - Configurable retry policies with max attempts and timeout support
//! - Jitter support to prevent thundering herd
//! - Custom error predicates for fine-grained retry control
//! - Fully async with tokio integration
//!
//! # Examples
//!
//! ```rust,ignore
//! use phenotype_retry::{backoff::Backoff, policy::RetryPolicy, executor::execute_with_retry};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let policy = RetryPolicy::default()
//!         .with_max_attempts(3)
//!         .with_backoff(Backoff::exponential(100, 2.0, None));
//!
//!     let result = execute_with_retry(
//!         || async { Ok("success") },
//!         policy,
//!         None,
//!     ).await?;
//!
//!     Ok(())
//! }
//! ```

pub mod backoff;
pub mod executor;
pub mod policy;

pub use backoff::Backoff;
pub use executor::{execute_with_retry, RetryError};
pub use policy::RetryPolicy;

#[cfg(test)]
mod tests;
