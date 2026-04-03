use crate::cost::Cost;
use crate::errors::CostError;

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
