//! # Event Bus Port
//!
//! Outbound port for event publishing and subscription.

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use crate::models::DomainEvent;

/// Event bus port for publish/subscribe operations.
#[async_trait]
pub trait EventBusPort: Send + Sync {
    /// Publish a domain event.
    async fn publish(&self, event: &DomainEvent) -> Result<(), EventBusError>;

    /// Publish multiple events in a batch.
    async fn publish_batch(&self, events: &[DomainEvent]) -> Result<(), EventBusError>;

    /// Subscribe to events of a given type.
    async fn subscribe(
        &self,
        event_type: &str,
        handler: Box<dyn EventHandler>,
    ) -> Result<SubscriptionId, EventBusError>;

    /// Unsubscribe from events.
    async fn unsubscribe(&self, subscription_id: SubscriptionId) -> Result<(), EventBusError>;
}

/// A handler function for events.
pub trait EventHandler: Send + Sync {
    /// Handle a domain event.
    fn handle(&self, event: &DomainEvent) -> Result<(), Box<dyn std::error::Error + Send + Sync>>;
}

/// Unique subscription identifier.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SubscriptionId(pub String);

impl std::fmt::Display for SubscriptionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Subscription({})", self.0)
    }
}

impl From<String> for SubscriptionId {
    fn from(id: String) -> Self {
        Self(id)
    }
}

/// Event bus operation errors.
#[derive(Debug, Clone, thiserror::Error)]
pub enum EventBusError {
    #[error("event not found: {0}")]
    NotFound(String),

    #[error("serialization error: {0}")]
    Serialization(String),

    #[error("connection error: {0}")]
    Connection(String),

    #[error("timeout: {0}")]
    Timeout(String),

    #[error("operation failed: {0}")]
    Operation(String),
}
