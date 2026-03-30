//! Inbound ports for hexagonal architecture.

use async_trait::async_trait;

pub type UseCaseResult<T> = std::result::Result<T, UseCaseError>;

#[derive(Debug, Clone)]
pub enum UseCaseError {
    ValidationFailed(String),
    BusinessRuleViolation(String),
    NotFound(String),
    Conflict(String),
    Unauthorized(String),
    Internal(String),
}

impl std::fmt::Display for UseCaseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ValidationFailed(msg) => write!(f, "validation failed: {msg}"),
            Self::BusinessRuleViolation(msg) => write!(f, "business rule violation: {msg}"),
            Self::NotFound(msg) => write!(f, "not found: {msg}"),
            Self::Conflict(msg) => write!(f, "conflict: {msg}"),
            Self::Unauthorized(msg) => write!(f, "unauthorized: {msg}"),
            Self::Internal(msg) => write!(f, "internal error: {msg}"),
        }
    }
}

impl std::error::Error for UseCaseError {}

#[async_trait]
pub trait UseCase: Send + Sync {
    type Request: Send + Sync;
    type Response: Send + Sync;
    async fn execute(&self, request: Self::Request) -> UseCaseResult<Self::Response>;
}

#[async_trait]
pub trait CommandHandler: Send + Sync {
    type Command: Send + Sync;
    async fn handle(&self, command: Self::Command) -> UseCaseResult<()>;
}

#[async_trait]
pub trait QueryHandler: Send + Sync {
    type Query: Send + Sync;
    type Response: Send + Sync;
    async fn handle(&self, query: Self::Query) -> UseCaseResult<Self::Response>;
}

#[async_trait]
pub trait EventSubscriber: Send + Sync {
    type Event: Send + Sync;
    async fn handle_event(&self, event: Self::Event) -> UseCaseResult<()>;
    fn event_type(&self) -> &'static str;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug)]
    struct TestRequest(String);

    struct TestUseCase;

    #[async_trait]
    impl UseCase for TestUseCase {
        type Request = TestRequest;
        type Response = TestRequest;

        async fn execute(&self, req: Self::Request) -> UseCaseResult<Self::Response> {
            if req.0.is_empty() {
                Err(UseCaseError::ValidationFailed("request cannot be empty".to_string()))
            } else {
                Ok(req)
            }
        }
    }

    #[tokio::test]
    async fn test_use_case_execution_success() {
        let use_case = TestUseCase;
        let result = use_case.execute(TestRequest("test".to_string())).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_use_case_error_display() {
        let err = UseCaseError::ValidationFailed("test error".to_string());
        assert_eq!(err.to_string(), "validation failed: test error");
    }
}
