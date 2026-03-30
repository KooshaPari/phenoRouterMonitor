//! Domain error type for business logic errors.

use thiserror::Error;

/// Error type for domain layer operations.
#[derive(Debug, Clone, Error)]
pub enum DomainError {
    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("business rule violated: {0}")]
    BusinessRuleViolated(String),

    #[error("aggregate not found: {0}")]
    AggregateNotFound(String),

    #[error("event error: {0}")]
    EventError(String),

    #[error("workflow error: {0}")]
    WorkflowError(String),

    #[error("validation failed: {0}")]
    ValidationFailed(String),

    #[error("conflict: {0}")]
    Conflict(String),
}

impl DomainError {
    pub fn invalid_state(msg: impl Into<String>) -> Self { Self::InvalidState(msg.into()) }
    pub fn business_rule_violated(msg: impl Into<String>) -> Self { Self::BusinessRuleViolated(msg.into()) }
    pub fn aggregate_not_found(id: impl Into<String>) -> Self { Self::AggregateNotFound(id.into()) }
    pub fn event_error(msg: impl Into<String>) -> Self { Self::EventError(msg.into()) }
    pub fn workflow_error(msg: impl Into<String>) -> Self { Self::WorkflowError(msg.into()) }
    pub fn validation_failed(msg: impl Into<String>) -> Self { Self::ValidationFailed(msg.into()) }
    pub fn conflict(msg: impl Into<String>) -> Self { Self::Conflict(msg.into()) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_invalid_state_error() { let err = DomainError::invalid_state("transition not allowed"); assert_eq!(err.to_string(), "invalid state: transition not allowed"); }

    #[test]
    fn test_business_rule_violated_error() { let err = DomainError::business_rule_violated("budget exceeded"); assert_eq!(err.to_string(), "business rule violated: budget exceeded"); }

    #[test]
    fn test_aggregate_not_found_error() { let err = DomainError::aggregate_not_found("Project/proj-123"); assert_eq!(err.to_string(), "aggregate not found: Project/proj-123"); }

    #[test]
    fn test_event_error() { let err = DomainError::event_error("invalid event"); assert_eq!(err.to_string(), "event error: invalid event"); }

    #[test]
    fn test_workflow_error() { let err = DomainError::workflow_error("step failed"); assert_eq!(err.to_string(), "workflow error: step failed"); }

    #[test]
    fn test_validation_failed_error() { let err = DomainError::validation_failed("invalid email"); assert_eq!(err.to_string(), "validation failed: invalid email"); }

    #[test]
    fn test_conflict_error() { let err = DomainError::conflict("version conflict"); assert_eq!(err.to_string(), "conflict: version conflict"); }
}
