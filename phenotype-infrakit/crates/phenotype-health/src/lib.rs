//! Phenotype Health

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus { Healthy, Degraded, Unhealthy, Unknown }

impl Default for HealthStatus { fn default() -> Self { Self::Unknown } }

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HealthBand { Excellent, Good, Fair, Poor, Critical, Unknown }

impl HealthBand {
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 90.0 => HealthBand::Excellent,
            s if s >= 75.0 => HealthBand::Good,
            s if s >= 60.0 => HealthBand::Fair,
            s if s >= 40.0 => HealthBand::Poor,
            s if s > 0.0 => HealthBand::Critical,
            _ => HealthBand::Unknown,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity { Info, Warning, Error, Critical }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
}

impl Finding {
    pub fn info(msg: impl Into<String>) -> Self {
        Self { severity: Severity::Info, message: msg.into() }
    }
    pub fn warning(msg: impl Into<String>) -> Self {
        Self { severity: Severity::Warning, message: msg.into() }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub dimension: String,
    pub weight: f32,
    pub score: f32,
    pub findings: Vec<Finding>,
}

impl DimensionScore {
    pub fn new(dimension: impl Into<String>, weight: f32, score: f32) -> Self {
        Self { dimension: dimension.into(), weight, score, findings: Vec::new() }
    }
    pub fn with_finding(mut self, finding: Finding) -> Self {
        self.findings.push(finding);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub repo_name: String,
    pub language: String,
    pub overall_score: f32,
    pub band: HealthBand,
    pub dimensions: Vec<DimensionScore>,
    pub findings_count: usize,
}

impl ProjectHealth {
    pub fn compute_overall_score(&mut self) {
        let mut total_weighted = 0.0;
        let mut total_weight = 0.0;
        for dim in &self.dimensions {
            total_weighted += dim.score * dim.weight;
            total_weight += dim.weight;
        }
        self.overall_score = if total_weight > 0.0 { total_weighted / total_weight } else { 0.0 };
        self.band = HealthBand::from_score(self.overall_score);
        self.findings_count = self.dimensions.iter().map(|d| d.findings.len()).sum();
    }
}
