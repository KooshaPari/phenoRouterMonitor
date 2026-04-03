//! Health monitoring for Phenotype infrastructure

use serde::{Deserialize, Serialize};

/// Health band classification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthBand {
    Excellent,
    Good,
    Fair,
    Poor,
    Critical,
}

/// Score for a specific dimension of health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionScore {
    pub name: String,
    pub score: u8,
    pub max_score: u8,
}

/// Overall project health summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectHealth {
    pub project_id: String,
    pub overall_score: u8,
    pub band: HealthBand,
    pub dimensions: Vec<DimensionScore>,
}

impl Default for HealthBand {
    fn default() -> Self {
        HealthBand::Good
    }
}

impl HealthBand {
    /// Convert a score (0-100) to a health band
    pub fn from_score(score: u8) -> Self {
        match score {
            90..=100 => HealthBand::Excellent,
            75..=89 => HealthBand::Good,
            50..=74 => HealthBand::Fair,
            25..=49 => HealthBand::Poor,
            _ => HealthBand::Critical,
        }
    }
}
