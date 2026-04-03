//! BDD testing framework

use std::collections::HashMap;

/// Step argument
#[derive(Debug, Clone)]
pub enum StepArg {
    String(String),
    Table(Vec<Vec<String>>),
    PyString(String),
}

impl StepArg {
    pub fn as_str(&self) -> Option<&str> {
        match self {
            StepArg::String(ref s) => Some(s),
            _ => None,
        }
    }
    
    pub fn as_table(&self) -> Option<&Vec<Vec<String>>> {
        match self {
            StepArg::Table(ref t) => Some(t),
            _ => None,
        }
    }
}

/// Step context
#[derive(Debug)]
pub struct StepContext {
    pub args: Vec<StepArg>,
    pub table: Option<Vec<Vec<String>>>,
}

impl StepContext {
    pub fn new() -> Self {
        Self {
            args: Vec::new(),
            table: None,
        }
    }
    
    pub fn add_arg(&mut self, arg: String) {
        self.args.push(StepArg::String(arg));
    }
}

impl Default for StepContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Feature result
#[derive(Debug)]
pub struct FeatureResult {
    pub passed: bool,
    pub scenarios: Vec<ScenarioResult>,
}

impl FeatureResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            scenarios: Vec::new(),
        }
    }
}

impl Default for FeatureResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Scenario result
#[derive(Debug)]
pub struct ScenarioResult {
    pub passed: bool,
    pub steps: Vec<StepResult>,
}

impl ScenarioResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            steps: Vec::new(),
        }
    }
}

impl Default for ScenarioResult {
    fn default() -> Self {
        Self::new()
    }
}

/// Step result
#[derive(Debug)]
pub struct StepResult {
    pub passed: bool,
    pub error: Option<String>,
}

impl StepResult {
    pub fn new() -> Self {
        Self {
            passed: true,
            error: None,
        }
    }
    
    pub fn failed(mut self, error: String) -> Self {
        self.passed = false;
        self.error = Some(error);
        self
    }
}

impl Default for StepResult {
    fn default() -> Self {
        Self::new()
    }
}

/// BDD Runner
pub struct BddRunner {
    steps: HashMap<String, fn(&StepContext)>,
}

impl BddRunner {
    pub fn new() -> Self {
        Self {
            steps: HashMap::new(),
        }
    }
    
    pub fn register_step(&mut self, pattern: &str, handler: fn(&StepContext)) {
        self.steps.insert(pattern.to_string(), handler);
    }
    
    pub fn run_feature(&self, feature: &str) -> FeatureResult {
        let mut result = FeatureResult::new();
        for line in feature.lines() {
            if line.trim().starts_with("Scenario:") {
                result.scenarios.push(ScenarioResult::new());
            }
        }
        result
    }
}

impl Default for BddRunner {
    fn default() -> Self {
        Self::new()
    }
}
