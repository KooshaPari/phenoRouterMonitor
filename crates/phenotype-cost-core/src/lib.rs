//! Cost modeling, analysis, and budgeting for computational operations.
//!
//! This crate provides a comprehensive framework for modeling and analyzing
//! computational costs, including:
//!
//! - `Cost`: Core cost type with arithmetic operations
//! - `CostModel<T>`: Trait for custom cost functions
//! - `CostAnalyzer`: Tools for analyzing code path costs
//! - `Complexity`: Enum for algorithmic complexity classification
//!
//! # Example
//!
//! ```rust
//! use phenotype_cost_core::{Cost, Complexity, CostModel};
//!
//! // Define a simple cost model
//! struct TokenCost {
//!     per_token: u64,
//! }
//!
//! impl CostModel for TokenCost {
//!     fn calculate(&self, context: &str) -> Cost {
//!         Cost::from_tokens(context.len() as u64)
//!     }
//! }
//!
//! // Create and use costs
//! let request_cost = Cost::from_tokens(1000);
//! let response_cost = Cost::from_tokens(500);
//! let total = (request_cost + response_cost).unwrap();
//! ```

pub mod analyzer;
pub mod budget;
pub mod cost;
pub mod errors;
pub mod model;
pub mod types;

pub use analyzer::{CostAnalysis, CostAnalyzer};
pub use budget::BudgetManager;
pub use cost::Cost;
pub use errors::CostError;
pub use model::{CostModel, TokenCostModel};
pub use types::{Complexity, CostUnit};

#[cfg(test)]
mod tests {
    use super::*;
    use std::cmp::Ordering;
    use std::ops::{Add, Div};

    #[test]
    fn test_cost_creation() {
        let cost = Cost::from_tokens(100);
        assert_eq!(cost.value(), 100);
        assert_eq!(cost.unit(), CostUnit::Tokens);
    }

    #[test]
    fn test_cost_addition() {
        let a = Cost::from_tokens(50);
        let b = Cost::from_tokens(30);
        let result = (a + b).unwrap();
        assert_eq!(result.value(), 80);
    }

    #[test]
    fn test_cost_addition_different_units() {
        let a = Cost::from_tokens(50);
        let b = Cost::from_milliseconds(30);
        assert!(a.add(b).is_err());
    }

    #[test]
    fn test_cost_subtraction() {
        let a = Cost::from_tokens(50);
        let b = Cost::from_tokens(30);
        let result = (a - b).unwrap();
        assert_eq!(result.value(), 20);
    }

    #[test]
    fn test_cost_multiplication() {
        let cost = Cost::from_tokens(10);
        let scaled = cost * 5;
        assert_eq!(scaled.value(), 50);
    }

    #[test]
    fn test_cost_division() {
        let cost = Cost::from_tokens(100);
        let divided = cost.div(4).unwrap();
        assert_eq!(divided.value(), 25);
    }

    #[test]
    fn test_cost_division_by_zero() {
        let cost = Cost::from_tokens(100);
        assert!(cost.div(0).is_err());
    }

    #[test]
    fn test_cost_exceeds() {
        let cost = Cost::from_tokens(100);
        let budget = Cost::from_tokens(50);
        assert!(cost.exceeds(budget).is_err());

        let small_cost = Cost::from_tokens(30);
        assert!(small_cost.exceeds(budget).is_ok());
    }

    #[test]
    fn test_cost_percentage() {
        let cost = Cost::from_tokens(50);
        let budget = Cost::from_tokens(100);
        let percentage = cost.percentage_of(budget).unwrap();
        assert!((percentage - 50.0).abs() < 0.001);
    }

    #[test]
    fn test_complexity_estimate() {
        assert_eq!(Complexity::Constant.estimate_cost(1000), 1);
        assert!(Complexity::Linear.estimate_cost(100) > 1);
        assert!(Complexity::Quadratic.estimate_cost(100) > Complexity::Linear.estimate_cost(100));
    }

    #[test]
    fn test_complexity_compare() {
        assert_eq!(Complexity::compare(Complexity::Constant, Complexity::Linear), Ordering::Less);
        assert_eq!(Complexity::compare(Complexity::Quadratic, Complexity::Linear), Ordering::Greater);
        assert_eq!(Complexity::compare(Complexity::Linear, Complexity::Linear), Ordering::Equal);
    }

    #[test]
    fn test_token_cost_model() {
        let model = TokenCostModel::new(10);
        let cost = model.calculate("hello world");
        assert_eq!(cost.value(), 110);
        assert_eq!(cost.unit(), CostUnit::Tokens);
    }

    #[test]
    fn test_cost_analyzer_estimate() {
        let analyzer = CostAnalyzer::with_tokens(Complexity::Linear, 10);
        let estimated = analyzer.estimate(100);
        assert_eq!(estimated.value(), 1000);
    }

    #[test]
    fn test_cost_analyzer_analyze() {
        let analyzer = CostAnalyzer::with_tokens(Complexity::Linear, 10);
        let budget = Cost::from_tokens(500);
        let analysis = analyzer.analyze(25, budget).unwrap();
        assert_eq!(analysis.percentage, 50.0);
        assert!(analysis.within_budget);
    }

    #[test]
    fn test_budget_manager_spend() {
        let mut manager = BudgetManager::with_token_budget(100);
        assert!(manager.spend(Cost::from_tokens(30)).is_ok());
        assert_eq!(manager.remaining().value(), 70);
        assert_eq!(manager.spent().value(), 30);
    }

    #[test]
    fn test_budget_manager_exceed() {
        let mut manager = BudgetManager::with_token_budget(100);
        assert!(manager.spend(Cost::from_tokens(150)).is_err());
    }

    #[test]
    fn test_budget_manager_reset() {
        let mut manager = BudgetManager::with_token_budget(100);
        manager.spend(Cost::from_tokens(50)).unwrap();
        manager.reset();
        assert_eq!(manager.remaining().value(), 100);
        assert_eq!(manager.spent().value(), 0);
    }

    #[test]
    fn test_budget_manager_percentage() {
        let mut manager = BudgetManager::with_token_budget(100);
        manager.spend(Cost::from_tokens(25)).unwrap();
        let percentage = manager.percentage_used().unwrap();
        assert!((percentage - 25.0).abs() < 0.001);
    }

    #[test]
    fn test_cost_display() {
        let cost = Cost::from_tokens(42);
        assert_eq!(format!("{}", cost), "42 tokens");
    }

    #[test]
    fn test_complexity_description() {
        assert_eq!(Complexity::Constant.description(), "O(1) - Constant time");
        assert_eq!(Complexity::Quadratic.description(), "O(n²) - Quadratic time");
    }

    #[test]
    fn test_budget_manager_refund() {
        let mut manager = BudgetManager::with_token_budget(100);
        manager.spend(Cost::from_tokens(30)).unwrap();
        manager.refund(Cost::from_tokens(10)).unwrap();
        assert_eq!(manager.remaining().value(), 80);
        assert_eq!(manager.spent().value(), 20);
    }

    #[test]
    fn test_cost_saturation() {
        let cost = Cost::from_tokens(u64::MAX);
        let scaled = cost.scale(2);
        assert_eq!(scaled.value(), u64::MAX);
    }
}
