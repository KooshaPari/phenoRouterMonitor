use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::fmt;

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
