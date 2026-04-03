//! BDD testing utilities for Phenotype

/// Gherkin parser and test runner
pub struct GherkinParser;

impl GherkinParser {
    /// Create a new parser
    pub fn new() -> Self {
        Self
    }
}

impl Default for GherkinParser {
    fn default() -> Self {
        Self::new()
    }
}
