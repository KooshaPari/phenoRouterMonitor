//! Generic use case port for application services.

use async_trait::async_trait;

/// Input type for the use case.
pub trait UseCaseInput: Send + Sync + Sized {}
/// Output type from the use case.
pub trait UseCaseOutput: Send + Sync + Sized {}

/// Generic use case port following hexagonal architecture patterns.
///
/// Implement this trait for application services that handle business logic.
#[async_trait]
pub trait UseCase<I: UseCaseInput, O: UseCaseOutput>: Send + Sync {
    /// Execute the use case with the given input.
    async fn execute(&self, input: I) -> Result<O, UseCaseError>;
}

/// Errors that can occur during use case execution.
#[derive(Debug, thiserror::Error)]
pub enum UseCaseError {
    #[error("validation failed: {0}")]
    Validation(String),

    #[error("not found: {entity} {id}")]
    NotFound { entity: String, id: String },

    #[error("policy violation: {0}")]
    PolicyViolation(String),

    #[error("internal error: {0}")]
    Internal(String),
}
