//! Event handler port for domain event processing.

use async_trait::async_trait;

/// Marker trait for domain event types.
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> &str;
    fn aggregate_id(&self) -> &str;
}

/// Event handler port for processing domain events.
#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    /// Handle the given event.
    async fn handle(&self, event: E) -> Result<(), EventHandlerError>;
}

/// Errors that can occur during event handling.
#[derive(Debug, thiserror::Error)]
pub enum EventHandlerError {
    #[error("handler not found: {0}")]
    HandlerNotFound(String),

    #[error("processing failed: {0}")]
    ProcessingFailed(String),

    #[error("internal error: {0}")]
    Internal(String),
}
