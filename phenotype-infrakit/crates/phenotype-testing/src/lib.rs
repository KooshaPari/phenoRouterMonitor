//! Testing utilities for Phenotype

/// Test fixture utilities
pub struct Fixture;

impl Fixture {
    /// Create a new fixture
    pub fn new() -> Self {
        Self
    }
}

impl Default for Fixture {
    fn default() -> Self {
        Self::new()
    }
}
