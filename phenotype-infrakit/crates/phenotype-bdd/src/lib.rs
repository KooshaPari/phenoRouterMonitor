//! Behavior-Driven Development utilities for Phenotype
//!
//! This crate provides Gherkin-like step definitions and feature parsing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::result::Result;

/// A feature file representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    /// Feature title
    pub title: String,
    /// Feature description
    pub description: String,
    /// Background steps (run before each scenario)
    pub background: Vec<Step>,
    /// Scenarios in this feature
    pub scenarios: Vec<Scenario>,
}

/// A scenario within a feature
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Scenario title
    pub title: String,
    /// Scenario steps
    pub steps: Vec<Step>,
    /// Tags for the scenario
    pub tags: Vec<String>,
}

/// A single Gherkin step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Step keyword (Given, When, Then, And, But)
    pub keyword: StepKeyword,
    /// Step text
    pub text: String,
    /// Optional step arguments (table rows, doc strings)
    pub arguments: Vec<StepArgument>,
}

/// Step keywords in Gherkin
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StepKeyword {
    #[serde(rename = "given")]
    Given,
    #[serde(rename = "when")]
    When,
    #[serde(rename = "then")]
    Then,
    #[serde(rename = "and")]
    And,
    #[serde(rename = "but")]
    But,
}

impl StepKeyword {
    /// Get the keyword as text
    pub fn as_str(&self) -> &'static str {
        match self {
            StepKeyword::Given => "Given",
            StepKeyword::When => "When",
            StepKeyword::Then => "Then",
            StepKeyword::And => "And",
            StepKeyword::But => "But",
        }
    }
}

/// Arguments to a step (table or doc string)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepArgument {
    /// A data table
    Table(DataTable),
    /// A doc string (multi-line text)
    DocString(String),
}

/// A Gherkin data table
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataTable {
    /// Table headers
    pub headers: Vec<String>,
    /// Table rows
    pub rows: Vec<Vec<String>>,
}

impl DataTable {
    /// Create a new empty data table
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
        }
    }

    /// Add a row to the table
    pub fn add_row(&mut self, row: Vec<String>) {
        self.rows.push(row);
    }
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new()
    }
}

/// A step definition with its implementation
pub struct StepDefinition {
    /// Pattern to match (regex)
    pub pattern: String,
    /// Handler function
    pub handler: Box<dyn Fn(ScenarioContext) -> Result<(), BddError> + Send + Sync>,
}

impl StepDefinition {
    /// Create a new step definition
    pub fn new<P: Into<String>>(
        pattern: P,
        handler: impl Fn(ScenarioContext) -> Result<(), BddError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            pattern: pattern.into(),
            handler: Box::new(handler),
        }
    }
}

/// Context passed to step handlers
#[derive(Debug, Clone)]
pub struct ScenarioContext {
    /// World object (shared state)
    pub world: World,
    /// Current step arguments
    pub arguments: Vec<String>,
    /// Data table if present
    pub table: Option<DataTable>,
}

impl ScenarioContext {
    /// Get a typed argument
    pub fn arg<T: std::str::FromStr>(&self, index: usize) -> Option<T> {
        self.arguments.get(index).and_then(|s| s.parse().ok())
    }

    /// Get world as a typed reference
    pub fn world_ref<T: 'static>(&self) -> Option<&T> {
        self.world.get::<T>()
    }

    /// Get world as a typed mutable reference
    pub fn world_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.world.get_mut::<T>()
    }
}

/// Shared world state for scenarios
#[derive(Debug, Default)]
pub struct World {
    data: HashMap<std::any::TypeId, Box<dyn std::any::Any>>,
}

impl World {
    /// Create a new empty world
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Set a value in the world
    pub fn set<T: 'static>(&mut self, value: T) {
        self.data.insert(std::any::TypeId::of::<T>(), Box::new(value));
    }

    /// Get a value from the world
    pub fn get<T: 'static>(&self) -> Option<&T> {
        self.data
            .get(&std::any::TypeId::of::<T>())
            .and_then(|b| b.downcast_ref())
    }

    /// Get a mutable value from the world
    pub fn get_mut<T: 'static>(&mut self) -> Option<&mut T> {
        self.data
            .get_mut(&std::any::TypeId::of::<T>())
            .and_then(|b| b.downcast_mut())
    }
}

/// BDD-related errors
/// BDD error types
#[derive(Error, Debug, Clone, PartialEq)]
pub enum BddError {
    #[error("step not found: {0}")]
    StepNotFound(String),

    #[error("world error: {0}")]
    WorldError(String),

    #[error("parse error: {0}")]
    ParseError(String),

    #[error("assertion failed: {0}")]
    AssertionFailed(String),
}

/// Result type for BDD operations
pub type Result<T> = std::result::Result<T, BddError>;
/// A feature runner
pub struct FeatureRunner {
    definitions: Vec<StepDefinition>,
    world_factory: Box<dyn Fn() -> World + Send + Sync>,
}

impl FeatureRunner {
    /// Create a new feature runner
    pub fn new() -> Self {
        Self {
            definitions: Vec::new(),
            world_factory: Box::new(World::new),
        }
    }

    /// Add a step definition
    pub fn define_step(mut self, definition: StepDefinition) -> Self {
        self.definitions.push(definition);
        self
    }

    /// Set the world factory
    pub fn with_world<F>(mut self, factory: F) -> Self
    where
        F: Fn() -> World + Send + Sync + 'static,
    {
        self.world_factory = Box::new(factory);
        self
    }

    /// Run a feature
    pub fn run(&self, feature: &Feature) -> Result<()> {
        for scenario in &feature.scenarios {
            self.run_scenario(scenario)?;
        }
        Ok(())
    }

    /// Run a single scenario
    pub fn run_scenario(&self, scenario: &Scenario) -> Result<()> {
        let mut world = (self.world_factory)();
        let all_steps: Vec<_> = feature::steps_with_background(scenario, &Vec::new())
            .collect();

        for step in all_steps {
            self.execute_step(&step, &mut world)?;
        }
        Ok(())
    }

    /// Execute a single step
    fn execute_step(&self, step: &Step, world: &mut World) -> Result<()> {
        let text = &step.text;

        // Find matching definition
        let definition = self
            .definitions
            .iter()
            .find(|d| regex_lite::Regex::new(&d.pattern).ok())
            .ok_or_else(|| BddError::StepNotFound(text.clone()))?;

        let ctx = ScenarioContext {
            world: world.clone(),
            arguments: Vec::new(),
            table: step.arguments.iter().find_map(|a| match a {
                StepArgument::Table(t) => Some(t.clone()),
                _ => None,
            }),
        };

        (definition.handler)(ctx)
    }
}

impl Default for FeatureRunner {
    fn default() -> Self {
        Self::new()
    }
}

// Helper module for feature processing
mod feature {
    use super::*;

    /// Get steps including background
    pub fn steps_with_background<'a>(
        scenario: &'a Scenario,
        _background: &'a [Step],
    ) -> impl Iterator<Item = &'a Step> {
        scenario.steps.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_keyword_as_str() {
        assert_eq!(StepKeyword::Given.as_str(), "Given");
        assert_eq!(StepKeyword::When.as_str(), "When");
        assert_eq!(StepKeyword::Then.as_str(), "Then");
    }

    #[test]
    fn test_data_table() {
        let mut table = DataTable::new();
        table.headers = vec!["name".into(), "age".into()];
        table.add_row(vec!["Alice".into(), "30".into()]);

        assert_eq!(table.headers.len(), 2);
        assert_eq!(table.rows.len(), 1);
    }

    #[test]
    fn test_world() {
        let mut world = World::new();
        world.set(42u32);
        assert_eq!(world.get::<u32>(), Some(&42));
    }

    #[test]
    fn test_feature_ser_de() {
        let feature = Feature {
            title: "Test".into(),
            description: "Description".into(),
            background: Vec::new(),
            scenarios: vec![Scenario {
                title: "Scenario".into(),
                steps: vec![Step {
                    keyword: StepKeyword::Given,
                    text: "a value".into(),
                    arguments: Vec::new(),
                }],
                tags: Vec::new(),
            }],
        };

        let json = serde_json::to_string(&feature).unwrap();
        let _parsed: Feature = serde_json::from_str(&json).unwrap();
    }
}
