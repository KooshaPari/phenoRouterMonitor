//! Risk assessment engine for task routing decisions.
//!
//! Calculates composite risk scores based on complexity, cost, dependencies,
//! and security sensitivity. Output always in [0.0, 1.0].

use serde::{Deserialize, Serialize};

/// Complexity levels for task assessment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComplexityLevel {
    Simple = 0,
    Moderate = 1,
    Complex = 2,
    VeryComplex = 3,
}

impl ComplexityLevel {
    /// Map complexity to risk score [0.0, 1.0].
    pub fn to_score(self) -> f64 {
        match self {
            Self::Simple => 0.1,
            Self::Moderate => 0.4,
            Self::Complex => 0.7,
            Self::VeryComplex => 1.0,
        }
    }
}

/// Input factors for risk calculation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskFactors {
    /// Complexity level of the task.
    pub complexity: ComplexityLevel,
    /// Cost in cents (mapped to [0.0, 1.0]).
    pub cost_cents: u32,
    /// Number of dependencies.
    pub dependency_count: usize,
    /// Whether task is security-sensitive (adds +0.3 if true, max 1.0).
    pub security_sensitive: bool,
    /// Optional maximum cost ceiling in cents (for normalization).
    pub max_cost_cents: u32,
}

impl RiskFactors {
    /// Create risk factors with defaults.
    pub fn new(complexity: ComplexityLevel) -> Self {
        Self {
            complexity,
            cost_cents: 0,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 10_000, // 100 USD default ceiling
        }
    }
}

/// Risk calculator for task assessment.
#[derive(Debug, Clone)]
pub struct RiskCalculator {
    complexity_weight: f64,
    cost_weight: f64,
    dependency_weight: f64,
}

impl RiskCalculator {
    /// Create a new risk calculator with standard weights.
    ///
    /// Weights: complexity=0.40, cost=0.35, dependencies=0.25 (sum=1.0).
    pub fn new() -> Self {
        Self {
            complexity_weight: 0.40,
            cost_weight: 0.35,
            dependency_weight: 0.25,
        }
    }

    /// Create a risk calculator with custom weights.
    ///
    /// # Panics
    /// Panics if weights don't sum to approximately 1.0 (within 0.01).
    pub fn with_weights(complexity: f64, cost: f64, dependency: f64) -> Self {
        let sum = complexity + cost + dependency;
        assert!(
            (sum - 1.0).abs() < 0.01,
            "Weights must sum to 1.0, got {:.2}",
            sum
        );
        Self {
            complexity_weight: complexity,
            cost_weight: cost,
            dependency_weight: dependency,
        }
    }

    /// Assess complexity risk [0.0, 1.0].
    pub fn assess_complexity(&self, level: ComplexityLevel) -> f64 {
        level.to_score()
    }

    /// Assess cost risk [0.0, 1.0].
    ///
    /// Maps cents to [0.0, 1.0] using max_cost_cents as ceiling.
    pub fn assess_cost(&self, factors: &RiskFactors) -> f64 {
        if factors.max_cost_cents == 0 {
            return 0.0;
        }
        let normalized =
            factors.cost_cents.min(factors.max_cost_cents) as f64 / factors.max_cost_cents as f64;
        normalized.min(1.0)
    }

    /// Assess dependency risk [0.0, 1.0].
    ///
    /// Maps dependency count to [0.0, 1.0] with 10 as max.
    pub fn assess_dependencies(&self, factors: &RiskFactors) -> f64 {
        (factors.dependency_count as f64 / 10.0).min(1.0)
    }

    /// Calculate composite risk score [0.0, 1.0].
    ///
    /// Formula: (complexity * 0.40) + (cost * 0.35) + (deps * 0.25) + security_boost
    /// Security boost: +0.3 if security_sensitive, clamped to max 1.0
    pub fn calculate(&self, factors: &RiskFactors) -> f64 {
        let complexity_score = self.assess_complexity(factors.complexity);
        let cost_score = self.assess_cost(factors);
        let dep_score = self.assess_dependencies(factors);

        let mut score = (complexity_score * self.complexity_weight)
            + (cost_score * self.cost_weight)
            + (dep_score * self.dependency_weight);

        // Add security boost if applicable
        if factors.security_sensitive {
            score += 0.3;
        }

        // Clamp to [0.0, 1.0]
        score.clamp(0.0, 1.0)
    }
}

impl Default for RiskCalculator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_complexity_scores() {
        assert_eq!(ComplexityLevel::Simple.to_score(), 0.1);
        assert_eq!(ComplexityLevel::Moderate.to_score(), 0.4);
        assert_eq!(ComplexityLevel::Complex.to_score(), 0.7);
        assert_eq!(ComplexityLevel::VeryComplex.to_score(), 1.0);
    }

    #[test]
    fn test_risk_calculator_simple_task() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors::new(ComplexityLevel::Simple);
        let risk = calc.calculate(&factors);
        assert!((0.0..=1.0).contains(&risk));
        assert!(risk < 0.2); // Simple should be very low
    }

    #[test]
    fn test_risk_calculator_very_complex_task() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 0,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        assert_eq!(risk, 0.4); // VeryComplex (1.0) * 0.40
    }

    #[test]
    fn test_risk_calculator_with_cost() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 5_000,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        let expected = (0.1 * 0.40) + (0.5 * 0.35); // 0.04 + 0.175 = 0.215
        assert!((risk - expected).abs() < 0.001);
    }

    #[test]
    fn test_risk_calculator_with_dependencies() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Moderate,
            cost_cents: 0,
            dependency_count: 5,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        let expected = (0.4 * 0.40) + (0.5 * 0.25); // 0.16 + 0.125 = 0.285
        assert!((risk - expected).abs() < 0.001);
    }

    #[test]
    fn test_risk_calculator_with_security() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 0,
            dependency_count: 0,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        let expected = (0.1 * 0.40) + 0.3; // 0.04 + 0.3 = 0.34
        assert!((risk - expected).abs() < 0.001);
    }

    #[test]
    fn test_risk_calculator_security_clamping() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::VeryComplex,
            cost_cents: 10_000,
            dependency_count: 10,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        assert_eq!(risk, 1.0); // Should be clamped
    }

    #[test]
    fn test_cost_mapping() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 2500,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let cost_risk = calc.assess_cost(&factors);
        assert_eq!(cost_risk, 0.25);
    }

    #[test]
    fn test_cost_exceeding_max() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 15_000,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let cost_risk = calc.assess_cost(&factors);
        assert_eq!(cost_risk, 1.0); // Should be clamped at max
    }

    #[test]
    fn test_dependency_mapping() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 0,
            dependency_count: 5,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let dep_risk = calc.assess_dependencies(&factors);
        assert_eq!(dep_risk, 0.5);
    }

    #[test]
    fn test_dependency_exceeding_max() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 0,
            dependency_count: 15,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let dep_risk = calc.assess_dependencies(&factors);
        assert_eq!(dep_risk, 1.0); // Should be clamped
    }

    #[test]
    fn test_all_factors_combined() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Complex,
            cost_cents: 7_500,
            dependency_count: 8,
            security_sensitive: true,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        // 0.7*0.40 + 0.75*0.35 + 0.8*0.25 + 0.3 = 0.28 + 0.2625 + 0.20 + 0.3 = 1.0425 -> clamped to 1.0
        assert_eq!(risk, 1.0); // Clamped at max
    }

    #[test]
    fn test_zero_max_cost() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Simple,
            cost_cents: 1000,
            dependency_count: 0,
            security_sensitive: false,
            max_cost_cents: 0,
        };
        let cost_risk = calc.assess_cost(&factors);
        assert_eq!(cost_risk, 0.0);
    }

    #[test]
    fn test_custom_weights() {
        let calc = RiskCalculator::with_weights(0.5, 0.3, 0.2);
        assert_eq!(calc.complexity_weight, 0.5);
        assert_eq!(calc.cost_weight, 0.3);
        assert_eq!(calc.dependency_weight, 0.2);
    }

    #[test]
    #[should_panic]
    fn test_invalid_weights_panic() {
        RiskCalculator::with_weights(0.6, 0.3, 0.2); // Sum != 1.0
    }

    #[test]
    fn test_moderate_task_risk() {
        let calc = RiskCalculator::new();
        let factors = RiskFactors {
            complexity: ComplexityLevel::Moderate,
            cost_cents: 2000,
            dependency_count: 2,
            security_sensitive: false,
            max_cost_cents: 10_000,
        };
        let risk = calc.calculate(&factors);
        // 0.4*0.40 + 0.2*0.35 + 0.2*0.25
        let expected = 0.16 + 0.07 + 0.05;
        assert!((risk - expected).abs() < 0.001);
    }
}
