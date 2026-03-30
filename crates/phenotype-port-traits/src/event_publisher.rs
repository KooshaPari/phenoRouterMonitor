//! Event publisher port for domain event distribution.
//!
//! Events represent facts about what has happened in the domain.
//! Publishers abstract the mechanism of distributing events to subscribers.

use async_trait::async_trait;
use std::fmt::Debug;

/// Errors that can occur during event publishing.
#[derive(Debug, Clone)]
pub enum EventPublisherError {
    SerializationError(String),
    PublishError(String),
    ChannelError(String),
}

impl std::fmt::Display for EventPublisherError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventPublisherError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            EventPublisherError::PublishError(msg) => write!(f, "Publish error: {}", msg),
            EventPublisherError::ChannelError(msg) => write!(f, "Channel error: {}", msg),
        }
    }
}

impl std::error::Error for EventPublisherError {}

/// Event publisher port for distributing domain events.
///
/// # Type Parameters
///
/// - `Event`: The domain event type. Must implement `Send + Sync`.
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize, Deserialize)]
/// struct UserCreatedEvent {
///     user_id: String,
///     email: String,
/// }
///
/// impl EventPublisher for KafkaPublisher {
///     type Event = UserCreatedEvent;
///     type Error = EventPublisherError;
///
///     async fn publish(&self, event: Self::Event) -> Result<(), Self::Error> {
///         // Send to Kafka...
///     }
/// }
/// ```
#[async_trait]
pub trait EventPublisher: Send + Sync + Debug {
    /// The type of events this publisher handles.
    type Event: Send + Sync;

    /// Error type returned by publishing operations.
    type Error: std::error::Error + Send + Sync + Debug;

    /// Publish a single event.
    async fn publish(&self, event: Self::Event) -> Result<(), Self::Error>;

    /// Publish multiple events atomically.
    ///
    /// # Note
    ///
    /// Implementations should aim for atomicity where possible to avoid partial publishes.
    async fn publish_batch(&self, events: Vec<Self::Event>) -> Result<(), Self::Error>;
}
