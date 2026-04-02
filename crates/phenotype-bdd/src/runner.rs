//! Scenario runner for executing BDD tests

use super::{parser::StepType, BddError, Result};
use crate::parser::{Scenario, Step};
use crate::steps::{StepContext, StepRegistry};
use thiserror::Error;

/// Error during scenario execution
#[derive(Error, Debug)]
pub enum RunError {
    #[error("Step not found: {step_type:?} {text}")]
    StepNotFound { step_type: StepType, text: String },
    #[error("Step execution failed: {0}")]
    StepExecutionFailed(String),
    #[error("Scenario {0} failed")]
    ScenarioFailed(String),
}

impl From<RunError> for BddError {
    fn from(e: RunError) -> Self {
        match e {
            RunError::StepNotFound { step_type, text } => {
                BddError::StepNotFound(format!("{:?}: {}", step_type, text))
            }
            RunError::StepExecutionFailed(msg) => BddError::StepExecutionFailed(msg),
            RunError::ScenarioFailed(name) => BddError::ScenarioFailed(name),
        }
    }
}

/// Runner for executing scenarios
pub struct ScenarioRunner<'a> {
    step_registry: &'a StepRegistry,
}

impl<'a> ScenarioRunner<'a> {
    /// Create a new scenario runner
    pub fn new(step_registry: &'a StepRegistry) -> Self {
        Self { step_registry }
    }

    /// Run a single scenario
    pub fn run_scenario(&self, scenario: &Scenario) -> Result<()> {
        let mut context = StepContext::new();

        for step in &scenario.steps {
            self.run_step(step, &mut context)?;
        }

        Ok(())
    }

    /// Run a single step
    fn run_step(&self, step: &Step, context: &mut StepContext) -> Result<()> {
        let effective_type = match step.step_type {
            StepType::And | StepType::But => {
                self.find_step_any(&step.text)
                    .ok_or(RunError::StepNotFound {
                        step_type: step.step_type,
                        text: step.text.clone(),
                    })?;
                return Ok(());
            }
            t => t,
        };

        let step_def = self
            .step_registry
            .find_step(effective_type, &step.text)
            .ok_or_else(|| RunError::StepNotFound {
                step_type: step.step_type,
                text: step.text.clone(),
            })?;

        (step_def.handler)(context);

        Ok(())
    }

    /// Try to find a step of any type matching the text
    fn find_step_any(&self, text: &str) -> Option<&crate::steps::StepDefinition> {
        use crate::parser::StepType;
        self.step_registry
            .find_step(StepType::Given, text)
            .or_else(|| self.step_registry.find_step(StepType::When, text))
            .or_else(|| self.step_registry.find_step(StepType::Then, text))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scenario_runner() {
        let mut registry = StepRegistry::new();

        static CALLED: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

        registry.register(StepType::Given, "a user", |_ctx| {
            CALLED.store(true, std::sync::atomic::Ordering::SeqCst);
        });

        let runner = ScenarioRunner::new(&registry);

        let scenario = Scenario {
            name: "Test".to_string(),
            steps: vec![Step {
                step_type: StepType::Given,
                text: "a user".to_string(),
                table: None,
            }],
            tags: vec![],
        };

        runner.run_scenario(&scenario).unwrap();
        assert!(CALLED.load(std::sync::atomic::Ordering::SeqCst));
    }
}
