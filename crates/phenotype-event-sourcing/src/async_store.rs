//! Async event store trait for async runtime compatibility.
//!
//! This module provides an async version of the EventStore trait for use
//! in async contexts (e.g., database backends, network storage).

use crate::event::EventEnvelope;
use async_trait::async_trait;
use std::fmt::Debug;

/// Async event store trait for async runtimes.
///
/// This trait provides an async interface for event sourcing operations.
/// Implementations can use async storage backends like databases or
/// network-attached storage.
#[async_trait]
pub trait AsyncEventStore<T, EntityType = String, EntityId = String>:
    Send + Sync
where
    T: Send + Sync + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    /// Append an event to the entity's stream.
    async fn append(&self, event: &EventEnvelope<T>) -> Result<i64, crate::EventSourcingError>;

    /// Get all events for a specific entity.
    async fn get(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, crate::EventSourcingError>;

    /// Get all events for an entity type.
    async fn get_all(&self, entity_type: &str) -> Result<Vec<EventEnvelope<T>>, crate::EventSourcingError>;

    /// Get events since a specific sequence number.
    async fn get_since(
        &self,
        entity_type: &str,
        entity_id: &str,
        sequence: i64,
    ) -> Result<Vec<EventEnvelope<T>>, crate::EventSourcingError>;

    /// Get events in a sequence range.
    async fn get_range(
        &self,
        entity_type: &str,
        entity_id: &str,
        from_seq: i64,
        to_seq: i64,
    ) -> Result<Vec<EventEnvelope<T>>, crate::EventSourcingError>;

    /// Get the latest sequence number for an entity.
    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Option<i64>, crate::EventSourcingError>;

    /// Verify the hash chain integrity for an entity.
    async fn verify_chain(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<bool, crate::EventSourcingError>;

    /// Count total events for an entity type.
    async fn count(&self, entity_type: &str) -> Result<usize, crate::EventSourcingError>;
}
