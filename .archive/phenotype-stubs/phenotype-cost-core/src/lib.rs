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

use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};
use thiserror::Error;

/// Errors that can occur during cost operations.
#[derive(Debug, Clone, Error)]
pub enum CostError {
    #[error("Budget exceeded: needed {needed}, available {available}")]
    BudgetExceeded { needed: Cost, available: Cost },

    #[error("Invalid cost operation: {message}")]
    InvalidOperation { message: String },

    #[error("Overflow in cost calculation")]
    Overflow,
}

/// Represents the computational complexity of an algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    /// O(1) - Constant time
    Constant,
    /// O(log n) - Logarithmic time
    Logarithmic,
    /// O(n) - Linear time
    Linear,
    /// O(n log n) - Linearithmic time
    Linearithmic,
    /// O(n²) - Quadratic time
    Quadratic,
    /// O(n³) - Cubic time
    Cubic,
    /// O(2ⁿ) - Exponential time
    Exponential,
    /// O(n!) - Factorial time
    Factorial,
}

impl Complexity {
    /// Returns a human-readable description of the complexity.
    pub fn description(&self) -> &'static str {
        match self {
            Complexity::Constant => "O(1) - Constant time",
            Complexity::Logarithmic => "O(log n) - Logarithmic time",
            Complexity::Linear => "O(n) - Linear time",
            Complexity::Linearithmic => "O(n log n) - Linearithmic time",
            Complexity::Quadratic => "O(n²) - Quadratic time",
            Complexity::Cubic => "O(n³) - Cubic time",
            Complexity::Exponential => "O(2ⁿ) - Exponential time",
            Complexity::Factorial => "O(n!) - Factorial time",
        }
    }

    /// Estimates the relative cost for a given input size.
    pub fn estimate_cost(&self, n: u64) -> u64 {
        match self {
            Complexity::Constant => 1,
            Complexity::Logarithmic => (n as f64).log2().max(1.0) as u64,
            Complexity::Linear => n,
            Complexity::Linearithmic => n * (n as f64).log2().max(1.0) as u64,
            Complexity::Quadratic => n.saturating_mul(n),
            Complexity::Cubic => n.saturating_mul(n).saturating_mul(n),
            Complexity::Exponential => 2u64.saturating_pow(n.min(64) as u32),
            Complexity::Factorial => {
                if n <= 20 {
                    (1..=n).fold(1u64, |acc, x| acc.saturating_mul(x))
                } else {
                    u64::MAX
                }
            }
        }
    }

    /// Compares two complexity classes.
    pub fn compare(a: Complexity, b: Complexity) -> Ordering {
        let weight = |c: Complexity| -> u8 {
            match c {
                Complexity::Constant => 0,
                Complexity::Logarithmic => 1,
                Complexity::Linear => 2,
                Complexity::Linearithmic => 3,
                Complexity::Quadratic => 4,
                Complexity::Cubic => 5,
                Complexity::Exponential => 6,
                Complexity::Factorial => 7,
            }
        };
        weight(a).cmp(&weight(b))
    }
}

/// Unit of measurement for costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostUnit {
    /// Token-based cost (for LLM operations)
    Tokens,
    /// Compute time in milliseconds
    Milliseconds,
    /// Memory usage in bytes
    Bytes,
    /// Monetary cost in cents
    Cents,
    /// Arbitrary units
    Units,
}

impl fmt::Display for CostUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CostUnit::Tokens => write!(f, "tokens"),
            CostUnit::Milliseconds => write!(f, "ms"),
            CostUnit::Bytes => write!(f, "bytes"),
            CostUnit::Cents => write!(f, "cents"),
            CostUnit::Units => write!(f, "units"),
        }
    }
}

/// Represents a computational cost with associated unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost {
    value: u64,
    unit: CostUnit,
}

impl Cost {
    /// Creates a new cost with the specified value and unit.
    pub fn new(value: u64, unit: CostUnit) -> Self {
        Self { value, unit }
    }

    /// Creates a cost in tokens.
    pub fn from_tokens(tokens: u64) -> Self {
        Self {
            value: tokens,
            unit: CostUnit::Tokens,
        }
    }

    /// Creates a cost in milliseconds.
    pub fn from_milliseconds(ms: u64) -> Self {
        Self {
            value: ms,
            unit: CostUnit::Milliseconds,
        }
    }

    /// Creates a cost in bytes.
    pub fn from_bytes(bytes: u64) -> Self {
        Self {
            value: bytes,
            unit: CostUnit::Bytes,
        }
    }

    /// Creates a cost in cents.
    pub fn from_cents(cents: u64) -> Self {
        Self {
            value: cents,
            unit: CostUnit::Cents,
        }
    }

    /// Creates a cost in arbitrary units.
    pub fn from_units(units: u64) -> Self {
        Self {
            value: units,
            unit: CostUnit::Units,
        }
    }

    /// Returns the value of the cost.
    pub fn value(&self) -> u64 {
        self.value
    }

    /// Returns the unit of the cost.
    pub fn unit(&self) -> CostUnit {
        self.unit
    }

    /// Scales the cost by a multiplier.
    pub fn scale(&self, factor: u64) -> Self {
        Self {
            value: self.value.saturating_mul(factor),
            unit: self.unit,
        }
    }

    /// Checks if this cost exceeds a budget.
    pub fn exceeds(&self, budget: Cost) -> Result<(), CostError> {
        if self.unit != budget.unit {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot compare costs with different units: {:?} vs {:?}",
                    self.unit, budget.unit
                ),
            });
        }

        if self.value > budget.value {
            Err(CostError::BudgetExceeded {
                needed: *self,
                available: budget,
            })
        } else {
            Ok(())
        }
    }

    /// Returns the cost as a percentage of a budget.
    pub fn percentage_of(&self, budget: Cost) -> Result<f64, CostError> {
        if self.unit != budget.unit {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot compare costs with different units: {:?} vs {:?}",
                    self.unit, budget.unit
                ),
            });
        }

        if budget.value == 0 {
            return Err(CostError::InvalidOperation {
                message: "Budget cannot be zero".to_string(),
            });
        }

        Ok((self.value as f64) / (budget.value as f64) * 100.0)
    }
}

impl Add for Cost {
    type Output = Result<Self, CostError>;

    fn add(self, other: Self) -> Self::Output {
        if self.unit != other.unit {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot add costs with different units: {:?} vs {:?}",
                    self.unit, other.unit
                ),
            });
        }

        Ok(Self {
            value: self.value.saturating_add(other.value),
            unit: self.unit,
        })
    }
}

impl Sub for Cost {
    type Output = Result<Self, CostError>;

    fn sub(self, other: Self) -> Self::Output {
        if self.unit != other.unit {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot subtract costs with different units: {:?} vs {:?}",
                    self.unit, other.unit
                ),
            });
        }

        Ok(Self {
            value: self.value.saturating_sub(other.value),
            unit: self.unit,
        })
    }
}

impl Mul<u64> for Cost {
    type Output = Self;

    fn mul(self, factor: u64) -> Self {
        self.scale(factor)
    }
}

impl Div<u64> for Cost {
    type Output = Result<Self, CostError>;

    fn div(self, divisor: u64) -> Self::Output {
        if divisor == 0 {
            return Err(CostError::InvalidOperation {
                message: "Cannot divide by zero".to_string(),
            });
        }

        Ok(Self {
            value: self.value / divisor,
            unit: self.unit,
        })
    }
}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.unit == other.unit {
            Some(self.value.cmp(&other.value))
        } else {
            None
        }
    }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {}", self.value, self.unit)
    }
}

/// Trait for types that can calculate costs.
pub trait CostModel: Send + Sync {
    /// Calculates the cost for a given context.
    fn calculate(&self, context: &str) -> Cost;

    /// Returns the complexity class of this cost model.
    fn complexity(&self) -> Complexity {
        Complexity::Constant
    }
}

/// Simple token-based cost model.
#[derive(Debug, Clone)]
pub struct TokenCostModel {
    cost_per_token: u64,
}

impl TokenCostModel {
    /// Creates a new token cost model.
    pub fn new(cost_per_token: u64) -> Self {
        Self { cost_per_token }
    }
}

impl CostModel for TokenCostModel {
    fn calculate(&self, context: &str) -> Cost {
        let tokens = context.len() as u64;
        Cost::from_tokens(tokens.saturating_mul(self.cost_per_token))
    }

    fn complexity(&self) -> Complexity {
        Complexity::Linear
    }
}

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

/// Budget manager for tracking and enforcing costs.
#[derive(Debug, Clone)]
pub struct BudgetManager {
    total: Cost,
    remaining: Cost,
    spent: Cost,
}

impl BudgetManager {
    /// Creates a new budget manager.
    pub fn new(total: Cost) -> Self {
        Self {
            remaining: total,
            total,
            spent: Cost::new(0, total.unit()),
        }
    }

    /// Creates a new budget manager with token budget.
    pub fn with_token_budget(tokens: u64) -> Self {
        let budget = Cost::from_tokens(tokens);
        Self::new(budget)
    }

    /// Attempts to spend a cost from the budget.
    pub fn spend(&mut self, cost: Cost) -> Result<(), CostError> {
        if cost.unit() != self.total.unit() {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot spend {:?} from {:?} budget",
                    cost.unit(),
                    self.total.unit()
                ),
            });
        }

        cost.exceeds(self.remaining)?;

        let new_spent = (self.spent + cost).map_err(|_| CostError::Overflow)?;
        self.spent = new_spent;
        self.remaining = (self.remaining - cost).map_err(|_| CostError::InvalidOperation {
            message: "Underflow in budget calculation".to_string(),
        })?;

        Ok(())
    }

    /// Checks if a cost can be spent.
    pub fn can_spend(&self, cost: Cost) -> bool {
        cost <= self.remaining && cost.unit() == self.total.unit()
    }

    /// Returns the total budget.
    pub fn total(&self) -> Cost {
        self.total
    }

    /// Returns the remaining budget.
    pub fn remaining(&self) -> Cost {
        self.remaining
    }

    /// Returns the spent budget.
    pub fn spent(&self) -> Cost {
        self.spent
    }

    /// Returns the percentage of budget used.
    pub fn percentage_used(&self) -> Result<f64, CostError> {
        self.spent.percentage_of(self.total)
    }

    /// Resets the budget to the initial total.
    pub fn reset(&mut self) {
        self.remaining = self.total;
        self.spent = Cost::new(0, self.total.unit());
    }

    /// Refunds a cost to the budget.
    pub fn refund(&mut self, cost: Cost) -> Result<(), CostError> {
        if cost.unit() != self.total.unit() {
            return Err(CostError::InvalidOperation {
                message: format!(
                    "Cannot refund {:?} to {:?} budget",
                    cost.unit(),
                    self.total.unit()
                ),
            });
        }

        let new_remaining = (self.remaining + cost).map_err(|_| CostError::Overflow)?;
        self.remaining = new_remaining;
        self.spent = (self.spent - cost).map_err(|_| CostError::InvalidOperation {
            message: "Cannot refund more than spent".to_string(),
        })?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(cost.value(), 110); // 11 chars * 10
        assert_eq!(cost.unit(), CostUnit::Tokens);
    }

    #[test]
    fn test_cost_analyzer_estimate() {
        let analyzer = CostAnalyzer::with_tokens(Complexity::Linear, 10);
        let estimated = analyzer.estimate(100);
        assert_eq!(estimated.value(), 1000); // 10 * 100
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
