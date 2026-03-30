//! Event store trait and implementations.

use serde::{de::DeserializeOwned, Serialize};

/// Event store trait for storing and retrieving events
pub trait EventStore: Send + Sync {
    /// Append an event to the store
    fn append<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> crate::error::Result<i64>;

    /// Get all events for an entity
    fn get_events<T: Serialize + DeserializeOwned + Send + Sync>(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> crate::error::Result<Vec<crate::event::EventEnvelope<T>>>;

    /// Count events for an entity
    fn count(&self, entity_type: &str, entity_id: &str) -> crate::error::Result<i64>;
}
