use serde::{Deserialize, Serialize};

use crate::cost::Cost;
use crate::errors::CostError;
use crate::types::Complexity;

/// Cost analyzer for analyzing code paths.
#[derive(Debug, Clone)]
pub struct CostAnalyzer {
    complexity: Complexity,
    base_cost: Cost,
}

impl CostAnalyzer {
    /// Creates a new cost analyzer.
    pub fn new(complexity: Complexity, base_cost: Cost) -> Self {
        Self {
            complexity,
            base_cost,
        }
    }

    /// Creates a new cost analyzer with tokens as the unit.
    pub fn with_tokens(complexity: Complexity, base_cost: u64) -> Self {
        Self {
            complexity,
            base_cost: Cost::from_tokens(base_cost),
        }
    }

    /// Estimates the cost for a given input size.
    pub fn estimate(&self, input_size: u64) -> Cost {
        let multiplier = self.complexity.estimate_cost(input_size);
        self.base_cost.scale(multiplier)
    }

    /// Analyzes whether a cost would exceed a budget.
    pub fn analyze(&self, input_size: u64, budget: Cost) -> Result<CostAnalysis, CostError> {
        let estimated = self.estimate(input_size);
        let percentage = estimated.percentage_of(budget)?;

        Ok(CostAnalysis {
            estimated,
            budget,
            percentage,
            within_budget: estimated <= budget,
        })
    }
}

/// Result of a cost analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis {
    /// The estimated cost.
    pub estimated: Cost,
    /// The available budget.
    pub budget: Cost,
    /// The percentage of budget used.
    pub percentage: f64,
    /// Whether the estimated cost is within budget.
    pub within_budget: bool,
}
