//! Message queue port for asynchronous communication.
//!
//! Message queues enable decoupling of components through topic-based pub/sub messaging.
//! Implementations abstract the underlying queue mechanism (Kafka, RabbitMQ, etc.).

use async_trait::async_trait;
use std::fmt::Debug;

/// Errors that can occur during message queue operations.
#[derive(Debug, Clone)]
pub enum MessageQueueError {
    TopicError(String),
    SendError(String),
    ReceiveError(String),
    SerializationError(String),
}

impl std::fmt::Display for MessageQueueError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MessageQueueError::TopicError(msg) => write!(f, "Topic error: {}", msg),
            MessageQueueError::SendError(msg) => write!(f, "Send error: {}", msg),
            MessageQueueError::ReceiveError(msg) => write!(f, "Receive error: {}", msg),
            MessageQueueError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
        }
    }
}

impl std::error::Error for MessageQueueError {}

/// Message queue port for topic-based pub/sub communication.
///
/// # Type Parameters
///
/// - `Message`: The message type. Must implement `Send + Sync`.
///
/// # Example
///
/// ```ignore
/// #[derive(Serialize, Deserialize)]
/// struct OrderNotification {
///     order_id: String,
///     status: String,
/// }
///
/// impl MessageQueue for KafkaQueue {
///     type Message = OrderNotification;
///     type Error = MessageQueueError;
///
///     async fn send(&self, topic: &str, message: Self::Message) -> Result<(), Self::Error> {
///         // Send to Kafka topic...
///     }
///
///     async fn receive(&self, topic: &str) -> Result<Option<Self::Message>, Self::Error> {
///         // Receive from Kafka topic...
///     }
/// }
/// ```
#[async_trait]
pub trait MessageQueue: Send + Sync + Debug {
    /// The type of messages this queue handles.
    type Message: Send + Sync;

    /// Error type returned by queue operations.
    type Error: std::error::Error + Send + Sync + Debug;

    /// Send a message to a topic.
    ///
    /// # Arguments
    ///
    /// - `topic`: The topic name or identifier
    /// - `message`: The message to send
    async fn send(&self, topic: &str, message: Self::Message) -> Result<(), Self::Error>;

    /// Receive a message from a topic.
    ///
    /// Returns `Ok(None)` if no message is available (non-blocking).
    /// Implementations may choose blocking semantics with configurable timeouts.
    async fn receive(&self, topic: &str) -> Result<Option<Self::Message>, Self::Error>;
}
