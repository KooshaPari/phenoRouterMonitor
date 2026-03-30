//! Inbound (driving) port traits for hexagonal architecture.
//!
//! These traits define the application's public API surface.
//! Adapters on the driving side (e.g. HTTP handlers, CLI commands)
//! call into these ports.
//!
//! Traces to: FR-PHENO-003

use crate::error::ContractError;
use async_trait::async_trait;

/// A general-purpose use case port.
///
/// Represents a single application operation that accepts an input
/// and produces an output.
#[async_trait]
pub trait UseCase<Input, Output>: Send + Sync + 'static
where
    Input: Send + 'static,
    Output: Send + 'static,
{
    /// Execute the use case.
    async fn execute(&self, input: Input) -> Result<Output, ContractError>;
}

/// A command handler for CQRS write operations.
///
/// Commands mutate state and return a confirmation or result.
#[async_trait]
pub trait CommandHandler<Command>: Send + Sync + 'static
where
    Command: Send + 'static,
{
    /// The result produced after handling the command.
    type Result: Send + 'static;

    /// Handle the command.
    async fn handle(&self, command: Command) -> Result<Self::Result, ContractError>;
}

/// A query handler for CQRS read operations.
///
/// Queries are side-effect-free and return a read model.
#[async_trait]
pub trait QueryHandler<Query, Response>: Send + Sync + 'static
where
    Query: Send + 'static,
    Response: Send + 'static,
{
    /// Handle the query.
    async fn handle(&self, query: Query) -> Result<Response, ContractError>;
}

/// An event handler that reacts to domain events.
#[async_trait]
pub trait EventHandler<Event>: Send + Sync + 'static
where
    Event: Send + 'static,
{
    /// Handle the event.
    async fn handle(&self, event: Event) -> Result<(), ContractError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CreateUser {
        name: String,
    }

    struct UserId(String);

    struct CreateUserHandler;

    #[async_trait]
    impl CommandHandler<CreateUser> for CreateUserHandler {
        type Result = UserId;

        async fn handle(&self, cmd: CreateUser) -> Result<UserId, ContractError> {
            Ok(UserId(format!("u-{}", cmd.name)))
        }
    }

    struct GetUser {
        id: String,
    }

    struct UserView {
        id: String,
        name: String,
    }

    struct GetUserHandler;

    #[async_trait]
    impl QueryHandler<GetUser, UserView> for GetUserHandler {
        async fn handle(&self, query: GetUser) -> Result<UserView, ContractError> {
            Ok(UserView {
                id: query.id.clone(),
                name: format!("User {}", query.id),
            })
        }
    }

    // Traces to: FR-PHENO-003
    #[tokio::test]
    async fn command_handler_works() {
        let handler = CreateUserHandler;
        let result = handler
            .handle(CreateUser {
                name: "Alice".into(),
            })
            .await
            .unwrap();
        assert_eq!(result.0, "u-Alice");
    }

    // Traces to: FR-PHENO-003
    #[tokio::test]
    async fn query_handler_works() {
        let handler = GetUserHandler;
        let view = handler
            .handle(GetUser { id: "42".into() })
            .await
            .unwrap();
        assert_eq!(view.id, "42");
        assert_eq!(view.name, "User 42");
    }
}
