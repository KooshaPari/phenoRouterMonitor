//! # Event Ports
//!
//! Outbound ports for event-driven architecture.
//!
//! ## Event-Driven Architecture (EDA)
//!
//! ```text
//! ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
//! │   Domain   │────▶│  Publisher  │────▶│  Broker    │────▶ Subscribers
//! │  (Events) │     │             │     │ (Kafka, etc)│
//! └─────────────┘     └─────────────┘     └─────────────┘
//! ```
//!
//! ## Interfaces
//!
//! - [`EventPublisher`] - Publish domain events
//! - [`EventSubscriber`] - Consume events

use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use super::Error;

/// Domain event trait.
///
/// All domain events should implement this trait.
pub trait DomainEvent: Serialize + for<'de> Deserialize<'de> + Send + Sync {
    /// Event type name (for routing)
    fn event_type(&self) -> &str;

    /// Event version (for schema evolution)
    fn event_version(&self) -> u32;

    /// Aggregate ID this event belongs to
    fn aggregate_id(&self) -> &str;
}

/// Event metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Event ID (UUID)
    pub event_id: String,
    /// Event type
    pub event_type: String,
    /// Event version
    pub event_version: u32,
    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Correlation ID (for tracing)
    pub correlation_id: Option<String>,
    /// Causation ID
    pub causation_id: Option<String>,
}

/// Wrapper for events with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Envelope<T> {
    pub metadata: EventMetadata,
    pub payload: T,
}

/// Event publisher port.
///
/// Used to publish domain events to a message broker.
#[async_trait]
pub trait EventPublisher: Send + Sync {
    /// Publish a domain event
    async fn publish<E>(&self, topic: &str, event: &E) -> Result<(), Error>
    where
        E: DomainEvent;

    /// Publish multiple events
    async fn publish_batch<E>(&self, topic: &str, events: &[E]) -> Result<(), Error>
    where
        E: DomainEvent;

    /// Create a topic/stream
    async fn create_topic(&self, topic: &str) -> Result<(), Error>;
}

/// Event subscriber port.
///
/// Used to consume domain events from a message broker.
#[async_trait]
pub trait EventSubscriber<E>: Send + Sync
where
    E: DomainEvent,
{
    /// Subscribe to a topic
    async fn subscribe(&self, topic: &str) -> Result<(), Error>;

    /// Unsubscribe from a topic
    async fn unsubscribe(&self, topic: &str) -> Result<(), Error>;
}
