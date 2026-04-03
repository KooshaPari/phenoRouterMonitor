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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_case_error_validation_display() {
        let err = UseCaseError::Validation("email invalid".into());
        assert_eq!(err.to_string(), "validation failed: email invalid");
    }

    #[test]
    fn use_case_error_not_found_display() {
        let err = UseCaseError::NotFound {
            entity: "Order".into(),
            id: "99".into(),
        };
        assert_eq!(err.to_string(), "not found: Order 99");
    }

    #[test]
    fn use_case_error_policy_violation_display() {
        let err = UseCaseError::PolicyViolation("rate limit exceeded".into());
        assert_eq!(err.to_string(), "policy violation: rate limit exceeded");
    }

    #[test]
    fn use_case_error_internal_display() {
        let err = UseCaseError::Internal("panic".into());
        assert_eq!(err.to_string(), "internal error: panic");
    }

    #[test]
    fn use_case_error_debug() {
        let err = UseCaseError::Validation("x".into());
        let debug = format!("{:?}", err);
        assert!(debug.contains("Validation"));
    }
}
