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
    async fn subscribe(&self, handler: impl EventHandler<E> + Send + Sync + 'static) -> Result<(), EventBusError>;

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
