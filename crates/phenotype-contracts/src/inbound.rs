//! Inbound ports (driving side) - interfaces for external requests.
//!
//! These are the interfaces that external actors (REST API, gRPC, CLI, etc.)
//! implement to drive the application.

use crate::Result;

/// Use case port for executing business operations.
pub trait UseCase: Send + Sync {
    type Request: Send + Sync;
    type Response: Send + Sync;

    /// Executes the use case with the given request.
    fn execute(&self, request: Self::Request) -> Result<Self::Response>;
}

/// Command handler port for processing commands.
pub trait CommandHandler: Send + Sync {
    type Command: Send + Sync;

    /// Handles a command.
    fn handle(&self, command: Self::Command) -> Result<()>;
}

/// Query handler port for processing queries.
pub trait QueryHandler: Send + Sync {
    type Query: Send + Sync;
    type Result: Send + Sync;

    /// Handles a query and returns the result.
    fn handle(&self, query: Self::Query) -> Result<Self::Result>;
}

/// Event handler port for processing domain events.
pub trait EventHandler: Send + Sync {
    type Event: Send + Sync;

    /// Handles a domain event.
    fn handle(&self, event: Self::Event) -> Result<()>;
}
