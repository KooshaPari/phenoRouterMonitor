//! Common integration test utilities — harness, fixtures, HTTP client helpers.

pub mod fixtures;
pub mod harness;

pub use fixtures::{TestFixtures, seed_test_data};
pub use harness::{HarnessError, TestHarness};
