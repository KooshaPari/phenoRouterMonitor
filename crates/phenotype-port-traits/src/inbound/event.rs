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

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEvent {
        event_type: String,
        aggregate_id: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &str {
            &self.event_type
        }
        fn aggregate_id(&self) -> &str {
            &self.aggregate_id
        }
    }

    #[test]
    fn domain_event_trait_methods() {
        let event = TestEvent {
            event_type: "order.created".into(),
            aggregate_id: "order-1".into(),
        };
        assert_eq!(event.event_type(), "order.created");
        assert_eq!(event.aggregate_id(), "order-1");
    }

    #[test]
    fn event_handler_error_handler_not_found_display() {
        let err = EventHandlerError::HandlerNotFound("OrderHandler".into());
        assert_eq!(err.to_string(), "handler not found: OrderHandler");
    }

    #[test]
    fn event_handler_error_processing_failed_display() {
        let err = EventHandlerError::ProcessingFailed("timeout".into());
        assert_eq!(err.to_string(), "processing failed: timeout");
    }

    #[test]
    fn event_handler_error_internal_display() {
        let err = EventHandlerError::Internal("panic".into());
        assert_eq!(err.to_string(), "internal error: panic");
    }
}
