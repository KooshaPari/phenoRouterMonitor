//! BDD (Behavior-Driven Development) testing framework

use regex::Regex;
use std::collections::HashMap;

// ============================================================================
// Step Definition Types
// ============================================================================

/// A captured value from a regex match
#[derive(Debug, Clone, PartialEq)]
pub enum StepArg {
    String(String),
    Integer(i64),
    Float(f64),
}

impl StepArg {
    pub fn as_str(&self) -> &str {
        match self {
            StepArg::String(s) => s,
            StepArg::Integer(i) => &i.to_string(),
            StepArg::Float(f) => &f.to_string(),
        }
    }
}

/// A step definition with regex pattern
#[derive(Debug, Clone)]
pub struct StepDefinition {
    pub pattern: Regex,
    pub handler: Box<dyn Fn(&[StepArg]) + Send + Sync>,
}

impl StepDefinition {
    pub fn new<F>(pattern: &str, handler: F) -> Result<Self, regex::Error>
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        Ok(Self {
            pattern: Regex::new(pattern)?,
            handler: Box::new(handler),
        })
    }
}

/// Registry of step definitions
#[derive(Debug, Default)]
pub struct StepRegistry {
    given: Vec<StepDefinition>,
    when: Vec<StepDefinition>,
    then: Vec<StepDefinition>,
}

impl StepRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn given<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, regex::Error>
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.given.push(StepDefinition::new(pattern, handler)?);
        Ok(self)
    }

    pub fn when<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, regex::Error>
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.when.push(StepDefinition::new(pattern, handler)?);
        Ok(self)
    }

    pub fn then<F>(&mut self, pattern: &str, handler: F) -> Result<&mut Self, regex::Error>
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.then.push(StepDefinition::new(pattern, handler)?;
        Ok(self)
    }

    fn find_step(&self, step_type: &str, text: &str) -> Option<(&StepDefinition, Vec<StepArg>)> {
        let steps = match step_type {
            "Given" => &self.given,
            "When" => &self.when,
            "Then" => &self.then,
            "And" | "But" => return None,
            _ => return None,
        };

        for step in steps {
            if let Some(caps) = step.pattern.captures(text) {
                let args: Vec<StepArg> = caps
                    .iter()
                    .skip(1)
                    .filter_map(|m| m.map(|m| m.as_str().into()))
                    .collect();
                return Some((step, args));
            }
        }
        None
    }
}

// ============================================================================
// Scenario Context
// ============================================================================

/// Runtime context for scenario execution
#[derive(Debug, Default)]
pub struct ScenarioContext {
    vars: HashMap<String, StepArg>,
}

impl ScenarioContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&mut self, key: impl Into<String>, value: StepArg) {
        self.vars.insert(key.into(), value);
    }

    pub fn get(&self, key: &str) -> Option<&StepArg> {
        self.vars.get(key)
    }
}

// ============================================================================
// Test Result Types
// ============================================================================

/// Result of test execution
#[derive(Debug)]
pub enum TestResult {
    Passed,
    Failed { message: String, location: String },
    Skipped,
}

// ============================================================================
// Test Runner
// ============================================================================

/// BDD test runner
pub struct TestRunner {
    registry: StepRegistry,
}

impl TestRunner {
    pub fn new() -> Self {
        Self {
            registry: StepRegistry::new(),
        }
    }

    pub fn given<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.registry.given(pattern, handler).ok();
        self
    }

    pub fn when<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.registry.when(pattern, handler).ok();
        self
    }

    pub fn then<F>(&mut self, pattern: &str, handler: F) -> &mut Self
    where
        F: Fn(&[StepArg]) + Send + Sync + 'static,
    {
        self.registry.then(pattern, handler).ok();
        self
    }

    pub fn run_scenario(&self, scenario: &Scenario) -> TestResult {
        let mut ctx = ScenarioContext::new();
        for step in &scenario.steps {
            let step_text = step.text.trim_start_matches(|c: char| c.is_whitespace());
            if let Some((def, args)) = self.registry.find_step(&step.step_type, step_text) {
                (def.handler)(&args);
            }
        }
        TestResult::Passed
    }
}

/// A single step in a scenario
#[derive(Debug)]
pub struct Step {
    pub step_type: String,
    pub text: String,
}

/// A scenario (test case)
#[derive(Debug)]
pub struct Scenario {
    pub name: String,
    pub steps: Vec<Step>,
}

impl Scenario {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            steps: Vec::new(),
        }
    }

    pub fn given(mut self, text: impl Into<String>) -> Self {
        self.steps.push(Step {
            step_type: "Given".into(),
            text: text.into(),
        });
        self
    }

    pub fn when(mut self, text: impl Into<String>) -> Self {
        self.steps.push(Step {
            step_type: "When".into(),
            text: text.into(),
        });
        self
    }

    pub fn then(mut self, text: impl Into<String>) -> Self {
        self.steps.push(Step {
            step_type: "Then".into(),
            text: text.into(),
        });
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_registry() {
        let registry = StepRegistry::new();
        assert!(registry.find_step("Given", "I have a thing").is_none());
    }

    #[test]
    fn test_scenario_builder() {
        let scenario = Scenario::new("test")
            .given("a user exists")
            .when("they login")
            .then("they see their dashboard");

        assert_eq!(scenario.steps.len(), 3);
    }
}
