// Cost calculation error types

use thiserror::Error;

/// Cost calculation result type
pub type CostResult<T> = Result<T, CostError>;

/// Error types for cost calculation and budget operations
#[derive(Debug, Error)]
pub enum CostError {
    /// Invalid model identifier
    #[error("Unknown model: {0}. Check pricing database for supported models.")]
    UnknownModel(String),

    /// Negative token count
    #[error("Invalid token count: {0}. Token counts must be non-negative.")]
    InvalidTokenCount(i64),

    /// Budget exceeded
    #[error("Budget limit exceeded. Limit: ${limit}, Current: ${current}, Request would exceed: ${requested}")]
    BudgetExceeded {
        limit: String,
        current: String,
        requested: String,
    },

    /// Invalid budget configuration
    #[error("Invalid budget configuration: {0}")]
    InvalidBudgetConfig(String),

    /// Cost calculation error
    #[error("Cost calculation failed: {0}")]
    CalculationError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),

    /// Invalid time range
    #[error("Invalid time range: {0}")]
    InvalidTimeRange(String),

    /// Provider pricing error
    #[error("Provider pricing not configured: {0}")]
    ProviderPricingNotFound(String),
}
