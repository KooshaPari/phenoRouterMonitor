use thiserror::Error;

use crate::cost::Cost;

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
