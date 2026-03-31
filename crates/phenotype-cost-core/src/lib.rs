//! Cost modeling, analysis, and budgeting for computational operations.

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
}

/// Represents the computational complexity of an algorithm.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Complexity {
    Constant,      // O(1)
    Logarithmic,   // O(log n)
    Linear,        // O(n)
    Linearithmic,  // O(n log n)
    Quadratic,     // O(n²)
    Cubic,         // O(n³)
    Exponential,   // O(2ⁿ)
    Factorial,     // O(n!)
}

impl Complexity {
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
                if n <= 20 { (1..=n).fold(1u64, |acc, x| acc.saturating_mul(x)) } else { u64::MAX }
            }
        }
    }
}

/// Unit of measurement for costs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CostUnit { Tokens, Milliseconds, Bytes, Cents, Units }

impl fmt::Display for CostUnit {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self { CostUnit::Tokens => write!(f, "tokens"), CostUnit::Milliseconds => write!(f, "ms"), CostUnit::Bytes => write!(f, "bytes"), CostUnit::Cents => write!(f, "cents"), CostUnit::Units => write!(f, "units") }
    }
}

/// Represents a computational cost with associated unit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cost { value: u64, unit: CostUnit }

impl Cost {
    pub fn new(value: u64, unit: CostUnit) -> Self { Self { value, unit } }
    pub fn from_tokens(tokens: u64) -> Self { Self { value: tokens, unit: CostUnit::Tokens } }
    pub fn from_milliseconds(ms: u64) -> Self { Self { value: ms, unit: CostUnit::Milliseconds } }
    pub fn from_bytes(bytes: u64) -> Self { Self { value: bytes, unit: CostUnit::Bytes } }
    pub fn from_cents(cents: u64) -> Self { Self { value: cents, unit: CostUnit::Cents } }
    pub fn from_units(units: u64) -> Self { Self { value: units, unit: CostUnit::Units } }
    pub fn value(&self) -> u64 { self.value }
    pub fn unit(&self) -> CostUnit { self.unit }
    pub fn scale(&self, factor: u64) -> Self { Self { value: self.value.saturating_mul(factor), unit: self.unit } }
    pub fn exceeds(&self, budget: Cost) -> Result<(), CostError> {
        if self.unit != budget.unit { return Err(CostError::InvalidOperation { message: format!("Cannot compare {:?} vs {:?}", self.unit, budget.unit) }); }
        if self.value > budget.value { Err(CostError::BudgetExceeded { needed: *self, available: budget }) } else { Ok(()) }
    }
    pub fn percentage_of(&self, budget: Cost) -> Result<f64, CostError> {
        if self.unit != budget.unit { return Err(CostError::InvalidOperation { message: format!("Cannot compare {:?} vs {:?}", self.unit, budget.unit) }); }
        if budget.value == 0 { return Err(CostError::InvalidOperation { message: "Budget cannot be zero".to_string() }); }
        Ok((self.value as f64) / (budget.value as f64) * 100.0)
    }
}

impl Add for Cost {
    type Output = Result<Self, CostError>;
    fn add(self, other: Self) -> Self::Output {
        if self.unit != other.unit { return Err(CostError::InvalidOperation { message: format!("Cannot add {:?} and {:?}", self.unit, other.unit) }); }
        Ok(Self { value: self.value.saturating_add(other.value), unit: self.unit })
    }
}

impl Sub for Cost {
    type Output = Result<Self, CostError>;
    fn sub(self, other: Self) -> Self::Output {
        if self.unit != other.unit { return Err(CostError::InvalidOperation { message: format!("Cannot subtract {:?} from {:?}", other.unit, self.unit) }); }
        Ok(Self { value: self.value.saturating_sub(other.value), unit: self.unit })
    }
}

impl Mul<u64> for Cost { type Output = Self; fn mul(self, factor: u64) -> Self { self.scale(factor) } }

impl Div<u64> for Cost { type Output = Result<Self, CostError>; fn div(self, divisor: u64) -> Self::Output {
    if divisor == 0 { return Err(CostError::InvalidOperation { message: "Cannot divide by zero".to_string() }); }
    Ok(Self { value: self.value / divisor, unit: self.unit })
}}

impl PartialOrd for Cost {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> { if self.unit == other.unit { Some(self.value.cmp(&other.value)) } else { None } }
}

impl fmt::Display for Cost {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result { write!(f, "{} {}", self.value, self.unit) }
}

/// Trait for types that can calculate costs.
pub trait CostModel: Send + Sync {
    fn calculate(&self, context: &str) -> Cost;
    fn complexity(&self) -> Complexity { Complexity::Constant }
}

/// Simple token-based cost model.
#[derive(Debug, Clone)]
pub struct TokenCostModel { cost_per_token: u64 }

impl TokenCostModel {
    pub fn new(cost_per_token: u64) -> Self { Self { cost_per_token } }
}

impl CostModel for TokenCostModel {
    fn calculate(&self, context: &str) -> Cost { Cost::from_tokens(context.len().saturating_mul(self.cost_per_token)) }
    fn complexity(&self) -> Complexity { Complexity::Linear }
}

/// Cost analyzer for analyzing code paths.
#[derive(Debug, Clone)]
pub struct CostAnalyzer { complexity: Complexity, base_cost: Cost }

impl CostAnalyzer {
    pub fn new(complexity: Complexity, base_cost: Cost) -> Self { Self { complexity, base_cost } }
    pub fn with_tokens(complexity: Complexity, base_cost: u64) -> Self { Self { complexity, base_cost: Cost::from_tokens(base_cost) } }
    pub fn estimate(&self, input_size: u64) -> Cost { self.base_cost.scale(self.complexity.estimate_cost(input_size)) }
    pub fn analyze(&self, input_size: u64, budget: Cost) -> Result<CostAnalysis, CostError> {
        let estimated = self.estimate(input_size);
        let percentage = estimated.percentage_of(budget)?;
        Ok(CostAnalysis { estimated, budget, percentage, within_budget: estimated <= budget })
    }
}

/// Result of a cost analysis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostAnalysis { pub estimated: Cost, pub budget: Cost, pub percentage: f64, pub within_budget: bool }

/// Budget manager for tracking and enforcing costs.
#[derive(Debug, Clone)]
pub struct BudgetManager { total: Cost, remaining: Cost, spent: Cost }

impl BudgetManager {
    pub fn new(total: Cost) -> Self { Self { remaining: total, total, spent: Cost::new(0, total.unit()) } }
    pub fn with_token_budget(tokens: u64) -> Self { Self::new(Cost::from_tokens(tokens)) }
    pub fn spend(&mut self, cost: Cost) -> Result<(), CostError> {
        if cost.unit() != self.total.unit() { return Err(CostError::InvalidOperation { message: format!("Cannot spend {:?} from {:?} budget", cost.unit(), self.total.unit()) }); }
        cost.exceeds(self.remaining)?;
        let new_spent = (self.spent + cost).map_err(|_| CostError::InvalidOperation { message: "Overflow".to_string() })?;
        self.spent = new_spent;
        self.remaining = (self.remaining - cost).map_err(|_| CostError::InvalidOperation { message: "Underflow".to_string() })?;
        Ok(())
    }
    pub fn can_spend(&self, cost: Cost) -> bool { cost <= self.remaining && cost.unit() == self.total.unit() }
    pub fn total(&self) -> Cost { self.total }
    pub fn remaining(&self) -> Cost { self.remaining }
    pub fn spent(&self) -> Cost { self.spent }
    pub fn percentage_used(&self) -> Result<f64, CostError> { self.spent.percentage_of(self.total) }
    pub fn reset(&mut self) { self.remaining = self.total; self.spent = Cost::new(0, self.total.unit()); }
    pub fn refund(&mut self, cost: Cost) -> Result<(), CostError> {
        if cost.unit() != self.total.unit() { return Err(CostError::InvalidOperation { message: format!("Cannot refund {:?} to {:?} budget", cost.unit(), self.total.unit()) }); }
        self.remaining = (self.remaining + cost).map_err(|_| CostError::InvalidOperation { message: "Overflow".to_string() })?;
        self.spent = (self.spent - cost).map_err(|_| CostError::InvalidOperation { message: "Cannot refund more than spent".to_string() })?;
        Ok(())
    }
}

/// Cost aggregator for combining multiple costs.
#[derive(Debug, Clone, Default)]
pub struct CostAggregator { costs: Vec<Cost> }

impl CostAggregator {
    pub fn new() -> Self { Self { costs: Vec::new() } }
    pub fn add(&mut self, cost: Cost) -> Result<(), CostError> {
        if let Some(first) = self.costs.first() {
            if first.unit() != cost.unit() { return Err(CostError::InvalidOperation { message: format!("Cannot aggregate {:?} and {:?}", first.unit(), cost.unit()) }); }
        }
        self.costs.push(cost);
        Ok(())
    }
    pub fn total(&self) -> Result<Cost, CostError> {
        let mut iter = self.costs.iter();
        let first = iter.next().ok_or(CostError::InvalidOperation { message: "Cannot calculate total of empty aggregator".to_string() })?;
        let mut total = *first;
        for cost in iter { total = (total + *cost)?; }
        Ok(total)
    }
    pub fn average(&self) -> Result<Cost, CostError> { let total = self.total()?; total.div(self.costs.len() as u64) }
    pub fn len(&self) -> usize { self.costs.len() }
    pub fn is_empty(&self) -> bool { self.costs.is_empty() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_cost_creation() { let cost = Cost::from_tokens(100); assert_eq!(cost.value(), 100); assert_eq!(cost.unit(), CostUnit::Tokens); }
    #[test] fn test_cost_addition() { let a = Cost::from_tokens(50); let b = Cost::from_tokens(30); assert_eq!((a + b).unwrap().value(), 80); }
    #[test] fn test_cost_addition_different_units() { let a = Cost::from_tokens(50); let b = Cost::from_milliseconds(30); assert!(a.add(b).is_err()); }
    #[test] fn test_cost_subtraction() { let a = Cost::from_tokens(50); let b = Cost::from_tokens(30); assert_eq!((a - b).unwrap().value(), 20); }
    #[test] fn test_cost_multiplication() { let cost = Cost::from_tokens(10); assert_eq!((cost * 5).value(), 50); }
    #[test] fn test_cost_division() { let cost = Cost::from_tokens(100); assert_eq!(cost.div(4).unwrap().value(), 25); }
    #[test] fn test_cost_division_by_zero() { let cost = Cost::from_tokens(100); assert!(cost.div(0).is_err()); }
    #[test] fn test_cost_exceeds() { let cost = Cost::from_tokens(100); let budget = Cost::from_tokens(50); assert!(cost.exceeds(budget).is_err()); let small = Cost::from_tokens(30); assert!(small.exceeds(budget).is_ok()); }
    #[test] fn test_cost_percentage() { let cost = Cost::from_tokens(50); let budget = Cost::from_tokens(100); assert!((cost.percentage_of(budget).unwrap() - 50.0).abs() < 0.001); }
    #[test] fn test_complexity_estimate() { assert_eq!(Complexity::Constant.estimate_cost(1000), 1); assert!(Complexity::Linear.estimate_cost(100) > 1); }
    #[test] fn test_token_cost_model() { let model = TokenCostModel::new(10); let cost = model.calculate("hello world"); assert_eq!(cost.value(), 110); }
    #[test] fn test_cost_analyzer_estimate() { let analyzer = CostAnalyzer::with_tokens(Complexity::Linear, 10); assert_eq!(analyzer.estimate(100).value(), 1000); }
    #[test] fn test_cost_analyzer_analyze() { let analyzer = CostAnalyzer::with_tokens(Complexity::Linear, 10); let budget = Cost::from_tokens(500); let analysis = analyzer.analyze(25, budget).unwrap(); assert_eq!(analysis.percentage, 50.0); assert!(analysis.within_budget); }
    #[test] fn test_budget_manager_spend() { let mut manager = BudgetManager::with_token_budget(100); assert!(manager.spend(Cost::from_tokens(30)).is_ok()); assert_eq!(manager.remaining().value(), 70); assert_eq!(manager.spent().value(), 30); }
    #[test] fn test_budget_manager_exceed() { let mut manager = BudgetManager::with_token_budget(100); assert!(manager.spend(Cost::from_tokens(150)).is_err()); }
    #[test] fn test_budget_manager_reset() { let mut manager = BudgetManager::with_token_budget(100); manager.spend(Cost::from_tokens(50)).unwrap(); manager.reset(); assert_eq!(manager.remaining().value(), 100); assert_eq!(manager.spent().value(), 0); }
    #[test] fn test_cost_aggregator() { let mut agg = CostAggregator::new(); agg.add(Cost::from_tokens(10)).unwrap(); agg.add(Cost::from_tokens(20)).unwrap(); assert_eq!(agg.total().unwrap().value(), 30); assert_eq!(agg.len(), 2); }
    #[test] fn test_cost_aggregator_empty() { let agg = CostAggregator::new(); assert!(agg.total().is_err()); assert!(agg.is_empty()); }
    #[test] fn test_cost_display() { assert_eq!(format!("{}", Cost::from_tokens(42)), "42 tokens"); }
    #[test] fn test_complexity_description() { assert_eq!(Complexity::Constant.description(), "O(1) - Constant time"); }
    #[test] fn test_budget_manager_refund() { let mut manager = BudgetManager::with_token_budget(100); manager.spend(Cost::from_tokens(30)).unwrap(); manager.refund(Cost::from_tokens(10)).unwrap(); assert_eq!(manager.remaining().value(), 80); }
}
