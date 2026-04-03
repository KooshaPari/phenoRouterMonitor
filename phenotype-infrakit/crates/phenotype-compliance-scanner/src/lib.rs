//! Compliance scanner for Phenotype infrastructure

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub rule_id: String,
    pub file_path: String,
    pub message: String,
}

#[derive(Default)]
pub struct ComplianceScanner;

impl ComplianceScanner {
    pub fn new() -> Self {
        Self
    }

    pub async fn scan(&self, _path: &str) -> anyhow::Result<Vec<Finding>> {
        Ok(Vec::new())
    }
}
