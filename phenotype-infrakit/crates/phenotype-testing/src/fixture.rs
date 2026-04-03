//! Test fixtures

/// Test fixture trait
pub trait Fixturable {
    fn name(&self) -> &str;
}

/// Test fixture
pub struct Fixture {
    pub name: String,
}

impl Fixture {
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}
