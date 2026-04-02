//! Testing utilities

pub mod fixture;

pub use fixture::Fixture;

/// Test fixture
pub struct Fixture {
    pub name: String,
}

impl Fixture {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
