//! Inbound (driving) ports: UseCase, CommandHandler, QueryHandler, EventHandler.
//!
//! These define the operations the application exposes to the outside world
//! (HTTP handlers, CLI, gRPC, etc.).

use async_trait::async_trait;

use crate::error::PortError;

/// Generic use-case interface.
///
/// # Type Parameters
///
/// * `I` -- input / request
/// * `O` -- output / response
///
/// # Examples
///
/// ```rust,ignore
/// #[async_trait]
/// impl UseCase<CreateOrder, OrderId> for OrderService {
///     async fn execute(&self, input: CreateOrder) -> Result<OrderId, PortError> { /* ... */ }
/// }
/// ```
#[async_trait]
pub trait UseCase<I, O>: Send + Sync {
    /// Execute the use case.
    async fn execute(&self, input: I) -> Result<O, PortError>;
}

/// Command handler (CQRS write side).
///
/// Commands modify state and return no domain payload.
#[async_trait]
pub trait CommandHandler<C>: Send + Sync {
    /// Handle a command.
    async fn handle(&self, command: C) -> Result<(), PortError>;
}

/// Query handler (CQRS read side).
///
/// Queries read state without side-effects.
#[async_trait]
pub trait QueryHandler<Q, R>: Send + Sync {
    /// Handle a query, returning the result.
    async fn handle(&self, query: Q) -> Result<R, PortError>;
}

/// Event handler for reacting to domain events.
#[async_trait]
pub trait EventHandler<E>: Send + Sync {
    /// Handle a domain event.
    async fn handle(&self, event: E) -> Result<(), PortError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Echo;

    #[async_trait]
    impl UseCase<String, String> for Echo {
        async fn execute(&self, input: String) -> Result<String, PortError> {
            Ok(input)
        }
    }

    #[tokio::test]
    async fn use_case_echo() {
        let svc = Echo;
        let out = svc.execute("hello".into()).await.unwrap();
        assert_eq!(out, "hello");
    }

    struct NoopCmd;

    #[async_trait]
    impl CommandHandler<String> for NoopCmd {
        async fn handle(&self, _command: String) -> Result<(), PortError> {
            Ok(())
        }
    }

    #[tokio::test]
    async fn command_handler_noop() {
        NoopCmd.handle("cmd".into()).await.unwrap();
    }

    struct ConstQuery;

    #[async_trait]
    impl QueryHandler<(), u32> for ConstQuery {
        async fn handle(&self, _query: ()) -> Result<u32, PortError> {
            Ok(42)
        }
    }

    #[tokio::test]
    async fn query_handler_const() {
        let r = ConstQuery.handle(()).await.unwrap();
        assert_eq!(r, 42);
    }
}
