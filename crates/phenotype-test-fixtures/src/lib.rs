//! Phenotype Test Fixtures
//!
//! Provides reusable test fixtures and property-based testing utilities
//! for the Phenotype ecosystem.

pub mod fixture;
pub mod property;
pub mod spy;

pub use fixture::{Fixture, FixtureManager};
pub use property::{PropertyTest, Strategy};
pub use spy::{Spy, SpyAssertions};

use thiserror::Error;

/// Errors that can occur when working with test fixtures
#[derive(Error, Debug)]
pub enum FixtureError {
    #[error("Fixture not found: {0}")]
    NotFound(String),
    #[error("Invalid fixture data: {0}")]
    InvalidData(String),
    #[error("Property test failed: {0}")]
    PropertyTestFailed(String),
}

/// Result type for test fixture operations
pub type Result<T> = std::result::Result<T, FixtureError>;

/// Initialize the test fixtures library
pub fn init() {
    // Initialize any global state needed for fixtures
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fixture_error_display() {
        let err = FixtureError::NotFound("test".to_string());
        assert_eq!(err.to_string(), "Fixture not found: test");
    }
}
