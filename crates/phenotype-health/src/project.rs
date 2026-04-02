//! Project-level health tracking for unified dashboard.
//!
//! Extends the base health checker pattern to cover cross-project
//! compliance, documentation, and quality metrics.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Programming language stacks supported for project health tracking.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum LanguageStack {
    Rust,
    TypeScript,
    Python,
    Go,
    Mixed(Vec<String>),
}

/// Health dimensions tracked per project.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
#[serde(rename_all = "snake_case")]
pub enum HealthDimension {
    Documentation,
    TestCoverage,
    Security,
    Dependencies,
    Compliance,
    CodeQuality,
}

impl HealthDimension {
    /// Weight of this dimension in overall health score calculation.
    #[must_use]
    pub fn weight(&self) -> f32 {
        match self {
            Self::Documentation => 0.15,
            Self::TestCoverage => 0.20,
            Self::Security => 0.25,
            Self::Dependencies => 0.15,
            Self::Compliance => 0.15,
            Self::CodeQuality => 0.10,
        }
    }
}

/// Health band classification based on overall score.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthBand {
    Excellent, // 90-100
    Good,      // 75-89
    Fair,      // 60-74
    Poor,      // 40-59
    Critical,  // 0-39
}

impl HealthBand {
    /// Calculate the health band from a score (0-100).
    #[must_use]
    pub fn from_score(score: f32) -> Self {
        match score {
            s if s >= 90.0 => Self::Excellent,
            s if s >= 75.0 => Self::Good,
            s if s >= 60.0 => Self::Fair,
            s if s >= 40.0 => Self::Poor,
            _ => Self::Critical,
        }
    }
}

/// Severity level for health findings.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Info,
    Warning,
    Error,
    Critical,
}

/// A specific finding within a health dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub severity: Severity,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_number: Option<u32>,
}

/// Score and findings for a specific health dimension.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub score: f32,
    pub target: f32,
    pub raw_value: f32,
    pub unit: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub findings: Vec<Finding>,
}

impl DimensionScore {
    /// Create a new dimension score with no findings.
    #[must_use]
    pub fn new(score: f32, target: f32, raw_value: f32, unit: impl Into<String>) -> Self {
        Self {
            score,
            target,
            raw_value,
            unit: unit.into(),
            findings: Vec::new(),
        }
    }

    /// Add a finding to this dimension score.
    pub fn add_finding(&mut self, finding: Finding) {
        self.findings.push(finding);
    }
}

/// Project health score with dimensional breakdown.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub repo_name: String,
    pub owner: String,
    pub language: LanguageStack,
    pub overall_score: f32,
    pub band: HealthBand,
    pub dimensions: HashMap<HealthDimension, DimensionScore>,
    pub last_scan: DateTime<Utc>,
    pub scan_version: String,
}

impl ProjectHealth {
    /// Calculate overall score from dimensions (weighted average).
    #[must_use]
    pub fn calculate_overall_score(dimensions: &HashMap<HealthDimension, DimensionScore>) -> f32 {
        if dimensions.is_empty() {
            return 0.0;
        }

        let weighted_sum: f32 = dimensions
            .iter()
            .map(|(dim, score)| dim.weight() * score.score)
            .sum();

        let total_weight: f32 = dimensions.keys().map(|d| d.weight()).sum();

        weighted_sum / total_weight
    }

    /// Create a new project health entry from dimension scores.
    #[must_use]
    pub fn new(
        repo_name: impl Into<String>,
        owner: impl Into<String>,
        language: LanguageStack,
        dimensions: HashMap<HealthDimension, DimensionScore>,
    ) -> Self {
        let overall_score = Self::calculate_overall_score(&dimensions);
        let band = HealthBand::from_score(overall_score);

        Self {
            repo_name: repo_name.into(),
            owner: owner.into(),
            language,
            overall_score,
            band,
            dimensions,
            last_scan: Utc::now(),
            scan_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }

    /// Get the score for a specific dimension.
    #[must_use]
    pub fn dimension_score(&self, dimension: HealthDimension) -> Option<f32> {
        self.dimensions.get(&dimension).map(|d| d.score)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_health_band_from_score() {
        assert!(matches!(
            HealthBand::from_score(95.0),
            HealthBand::Excellent
        ));
        assert!(matches!(HealthBand::from_score(80.0), HealthBand::Good));
        assert!(matches!(HealthBand::from_score(65.0), HealthBand::Fair));
        assert!(matches!(HealthBand::from_score(50.0), HealthBand::Poor));
        assert!(matches!(HealthBand::from_score(30.0), HealthBand::Critical));
    }

    #[test]
    fn test_dimension_weights_sum_to_one() {
        let dimensions = [
            HealthDimension::Documentation,
            HealthDimension::TestCoverage,
            HealthDimension::Security,
            HealthDimension::Dependencies,
            HealthDimension::Compliance,
            HealthDimension::CodeQuality,
        ];

        let total: f32 = dimensions.iter().map(|d| d.weight()).sum();
        assert!((total - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_calculate_overall_score() {
        let mut dimensions = HashMap::new();
        dimensions.insert(
            HealthDimension::Documentation,
            DimensionScore::new(80.0, 100.0, 4.0, "files"),
        );
        dimensions.insert(
            HealthDimension::Security,
            DimensionScore::new(100.0, 100.0, 0.0, "alerts"),
        );

        let score = ProjectHealth::calculate_overall_score(&dimensions);
        // (0.15 * 80 + 0.25 * 100) / (0.15 + 0.25) = (12 + 25) / 0.4 = 92.5
        assert!(score > 90.0 && score < 95.0);
    }
}
