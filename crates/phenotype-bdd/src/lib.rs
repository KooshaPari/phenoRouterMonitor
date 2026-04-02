//! Phenotype BDD - Gherkin-style BDD testing framework for Rust
//!
//! This crate provides a native Rust implementation of Gherkin-style BDD testing,
//! allowing you to write feature files in Gherkin syntax and execute them with
//! step definitions in Rust.

use thiserror::Error;

mod parser;
mod runner;
mod steps;

pub use parser::{Feature, ParseError, Scenario, Step, StepType};
pub use runner::{RunError, ScenarioRunner};
pub use steps::{StepContext, StepDefinition, StepRegistry};

/// Errors that can occur during BDD operations
#[derive(Error, Debug)]
pub enum BddError {
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Step not found: {0}")]
    StepNotFound(String),
    #[error("Step execution failed: {0}")]
    StepExecutionFailed(String),
    #[error("Scenario failed: {0}")]
    ScenarioFailed(String),
}

/// Result type for BDD operations
pub type Result<T> = std::result::Result<T, BddError>;

/// Main entry point for running BDD features
pub struct BddRunner {
    step_registry: StepRegistry,
}

impl BddRunner {
    /// Create a new BDD runner
    pub fn new() -> Self {
        Self {
            step_registry: StepRegistry::new(),
        }
    }

    /// Register a given step
    pub fn given<F>(&mut self, pattern: &str, func: F)
    where
        F: Fn(&mut StepContext) + Send + Sync + 'static,
    {
        self.step_registry
            .register(steps::StepType::Given, pattern, func);
    }

    /// Register a when step
    pub fn when<F>(&mut self, pattern: &str, func: F)
    where
        F: Fn(&mut StepContext) + Send + Sync + 'static,
    {
        self.step_registry
            .register(steps::StepType::When, pattern, func);
    }

    /// Register a then step
    pub fn then<F>(&mut self, pattern: &str, func: F)
    where
        F: Fn(&mut StepContext) + Send + Sync + 'static,
    {
        self.step_registry
            .register(steps::StepType::Then, pattern, func);
    }

    /// Run a feature file
    pub fn run_feature(&self, feature_content: &str) -> Result<()> {
        let feature = parser::parse_feature(feature_content)?;
        let runner = ScenarioRunner::new(&self.step_registry);

        for scenario in &feature.scenarios {
            runner.run_scenario(scenario)?;
        }

        Ok(())
    }
}

impl Default for BddRunner {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse a feature file and return the Feature structure
pub fn parse_feature(content: &str) -> Result<Feature> {
    parser::parse_feature(content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bdd_runner_basic() {
        let mut runner = BddRunner::new();
        runner.given("a user", |_ctx| {});
        runner.when("they log in", |_ctx| {});
        runner.then("they are authenticated", |_ctx| {});
    }

    #[test]
    fn test_parse_simple_feature() {
        let content = r#"
Feature: User Authentication

  Scenario: Successful login
    Given a user with valid credentials
    When they log in
    Then they are authenticated
"#;

        let feature = parse_feature(content).unwrap();
        assert_eq!(feature.name, "User Authentication");
        assert_eq!(feature.scenarios.len(), 1);
    }
}
