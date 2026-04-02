//! Contract testing

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ContractTest {
    pub name: String,
    pub description: String,
    pub created_at: DateTime<Utc>,
    pub requirements: Vec<String>,
}

impl ContractTest {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: String::new(),
            created_at: Utc::now(),
            requirements: Vec::new(),
        }
    }
}

#[derive(Debug)]
pub struct Contract {
    pub tests: Vec<ContractTest>,
}

impl Contract {
    pub fn new() -> Self {
        Self { tests: Vec::new() }
    }
    
    pub fn add_test(&mut self, test: ContractTest) {
        self.tests.push(test);
    }
}

impl Default for Contract {
    fn default() -> Self {
        Self::new()
    }
}
