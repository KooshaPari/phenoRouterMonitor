//! Event bus port for domain event publishing and subscription.

use async_trait::async_trait;
use serde::Serialize;

/// Domain event marker trait.
pub trait DomainEvent: Send + Sync + Serialize {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> &str;
}

/// Event envelope with metadata.
#[derive(Debug, Clone, serde::Serialize)]
pub struct EventEnvelope<E: DomainEvent> {
    pub event: E,
    pub event_type: &'static str,
    pub aggregate_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub correlation_id: Option<String>,
    pub causation_id: Option<String>,
}

impl<E: DomainEvent> EventEnvelope<E> {
    pub fn new(event: E) -> Self {
        Self {
            event_type: event.event_type(),
            aggregate_id: event.aggregate_id().to_string(),
            event,
            timestamp: chrono::Utc::now(),
            correlation_id: None,
            causation_id: None,
        }
    }

    pub fn with_correlation_id(mut self, id: String) -> Self {
        self.correlation_id = Some(id);
        self
    }

    pub fn with_causation_id(mut self, id: String) -> Self {
        self.causation_id = Some(id);
        self
    }
}

/// Event publisher port for publishing domain events.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event.
    async fn publish<E: DomainEvent>(&self, envelope: EventEnvelope<E>) -> Result<(), EventBusError>;

    /// Publish multiple events in a batch.
    async fn publish_batch<E: DomainEvent>(&self, envelopes: Vec<EventEnvelope<E>>) -> Result<(), EventBusError>;
}

/// Event subscriber port for consuming domain events.
#[async_trait]
pub trait EventSubscriber<E: DomainEvent>: Send + Sync {
    /// Subscribe to events of the given type.
    async fn subscribe(&self, handler: impl EventHandler<E> + 'static) -> Result<(), EventBusError>;

    /// Unsubscribe from events.
    async fn unsubscribe(&self) -> Result<(), EventBusError>;
}

/// Event handler marker trait.
#[async_trait]
pub trait EventHandler<E: DomainEvent>: Send + Sync {
    async fn handle(&self, event: E) -> Result<(), EventBusError>;
}

/// Event bus errors.
#[derive(Debug, thiserror::Error)]
pub enum EventBusError {
    #[error("connection error: {0}")]
    Connection(String),

    #[error("publish failed: {0}")]
    PublishFailed(String),

    #[error("subscription failed: {0}")]
    SubscriptionFailed(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("timeout")]
    Timeout,

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Clone, serde::Serialize)]
    struct TestEvent {
        id: String,
    }

    impl DomainEvent for TestEvent {
        fn event_type(&self) -> &'static str {
            "test.event"
        }
        fn aggregate_id(&self) -> &str {
            &self.id
        }
    }

    #[test]
    fn event_envelope_new() {
        let event = TestEvent {
            id: "agg-1".into(),
        };
        let envelope = EventEnvelope::new(event);
        assert_eq!(envelope.event_type, "test.event");
        assert_eq!(envelope.aggregate_id, "agg-1");
        assert!(envelope.correlation_id.is_none());
        assert!(envelope.causation_id.is_none());
    }

    #[test]
    fn event_envelope_with_correlation_id() {
        let event = TestEvent {
            id: "agg-1".into(),
        };
        let envelope = EventEnvelope::new(event).with_correlation_id("corr-123".into());
        assert_eq!(envelope.correlation_id.as_deref(), Some("corr-123"));
        assert!(envelope.causation_id.is_none());
    }

    #[test]
    fn event_envelope_with_causation_id() {
        let event = TestEvent {
            id: "agg-1".into(),
        };
        let envelope = EventEnvelope::new(event).with_causation_id("cause-456".into());
        assert_eq!(envelope.causation_id.as_deref(), Some("cause-456"));
        assert!(envelope.correlation_id.is_none());
    }

    #[test]
    fn event_envelope_chained_builder() {
        let event = TestEvent {
            id: "agg-1".into(),
        };
        let envelope = EventEnvelope::new(event)
            .with_correlation_id("corr-1".into())
            .with_causation_id("cause-1".into());
        assert_eq!(envelope.correlation_id.as_deref(), Some("corr-1"));
        assert_eq!(envelope.causation_id.as_deref(), Some("cause-1"));
    }

    #[test]
    fn event_envelope_debug() {
        let event = TestEvent {
            id: "agg-1".into(),
        };
        let envelope = EventEnvelope::new(event);
        let debug = format!("{:?}", envelope);
        assert!(debug.contains("test.event"));
        assert!(debug.contains("agg-1"));
    }

    #[test]
    fn event_bus_error_connection_display() {
        let err = EventBusError::Connection("refused".into());
        assert_eq!(err.to_string(), "connection error: refused");
    }

    #[test]
    fn event_bus_error_publish_failed_display() {
        let err = EventBusError::PublishFailed("queue full".into());
        assert_eq!(err.to_string(), "publish failed: queue full");
    }

    #[test]
    fn event_bus_error_subscription_failed_display() {
        let err = EventBusError::SubscriptionFailed("invalid topic".into());
        assert_eq!(err.to_string(), "subscription failed: invalid topic");
    }

    #[test]
    fn event_bus_error_serialization_display() {
        let err = EventBusError::Serialization("bad json".into());
        assert_eq!(err.to_string(), "serialization error: bad json");
    }

    #[test]
    fn event_bus_error_timeout_display() {
        let err = EventBusError::Timeout;
        assert_eq!(err.to_string(), "timeout");
    }

    #[test]
    fn event_bus_error_internal_display() {
        let err = EventBusError::Internal("crash".into());
        assert_eq!(err.to_string(), "internal error: crash");
    }
}
