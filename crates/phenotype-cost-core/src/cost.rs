use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, Div, Mul, Sub};

use serde::{Deserialize, Serialize};

use crate::errors::CostError;
use crate::types::CostUnit;

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
