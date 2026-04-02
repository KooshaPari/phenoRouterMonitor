//! # phenotype-testing - Shared Testing Infrastructure
//!
//! This crate provides common testing utilities for all Phenotype ecosystem crates,
//! including fixtures, mocks, chaos testing, and benchmark infrastructure.
//!
//! ## Core Features
//!
//! - **Test Fixtures**: Standardized test setup/teardown patterns
//! - **Mock Infrastructure**: Common mocking utilities for testing
//! - **Chaos Testing**: Tools for testing system resilience
//! - **Benchmarks**: Performance testing infrastructure
//!
//! ## Example
//!
//! ```rust
//! use phenotype_testing::{TestFixture, TestEnv};
//!
//! // Create a test environment
//! let env = TestEnv::new();
//! assert_eq!(env.name(), "test");
//! ```
//!
//! ## Usage
//!
//! Add to your `Cargo.toml`:
//! ```toml
//! [dev-dependencies]
//! phenotype-testing = { path = "../phenotype-testing" }
//! ```

pub mod fixture;
pub mod mock;
pub mod chaos;
pub mod benchmark;

pub use fixture::{TestFixture, TestEnv, TestResult, InfrastructureFixture};
pub use mock::MockStorage;
pub use chaos::ChaosRunner;
pub use benchmark::BenchmarkConfig;

// ============================================================================
// Module exports
// ============================================================================

/// Re-exports for convenience
pub mod prelude {
    pub use super::fixture::{TestFixture, TestEnv, TestResult, InfrastructureFixture};
    pub use super::mock::MockStorage;
    pub use super::chaos::ChaosRunner;
    pub use super::benchmark::BenchmarkConfig;
}
