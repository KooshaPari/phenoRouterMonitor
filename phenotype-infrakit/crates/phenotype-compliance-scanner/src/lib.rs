//! Documentation Scanner

use std::path::Path;

const REQUIRED_DOCS: &[&str] = &["CLAUDE.md", "README.md", "CONTRIBUTING.md", "LICENSE", "CHANGELOG.md"];

pub struct DocumentationScanner;

impl DocumentationScanner {
    pub fn new() -> Self { Self }
    pub fn scan(&self, path: &Path) -> ComplianceResult {
        let mut present = Vec::new();
        let mut missing = Vec::new();
        for doc in REQUIRED_DOCS {
            if path.join(doc).exists() { present.push(doc.to_string()); } else { missing.push(doc.to_string()); }
        }
        let score = (present.len() as f32 / REQUIRED_DOCS.len() as f32) * 100.0;
        ComplianceResult { score, present, missing }
    }
}

impl Default for DocumentationScanner { fn default() -> Self { Self::new() } }

#[derive(Debug, Clone)]
pub struct ComplianceResult { pub score: f32, pub present: Vec<String>, pub missing: Vec<String> }
