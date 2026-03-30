//! Inbound ports (driving side) - interfaces for external requests.

use crate::error;

/// Use case port for executing business operations.
pub trait UseCase: Send + Sync {
    type Request: Send + Sync;
    type Response: Send + Sync;

    fn execute(&self, request: Self::Request) -> error::Result<Self::Response>;
}

/// Command handler port for processing commands.
pub trait CommandHandler: Send + Sync {
    type Command: Send + Sync;

    fn handle(&self, command: Self::Command) -> error::Result<()>;
}

/// Query handler port for processing queries.
pub trait QueryHandler: Send + Sync {
    type Query: Send + Sync;
    type Output: Send + Sync;

    fn handle(&self, query: Self::Query) -> error::Result<Self::Output>;
}

/// Event handler port for processing domain events.
pub trait EventHandler: Send + Sync {
    type Event: Send + Sync;

    fn handle(&self, event: Self::Event) -> error::Result<()>;
}
