use crate::event::{Event, EventEnvelope};
use crate::error::EventSourcingError;

pub type Result<T> = std::result::Result<T, EventSourcingError>;

/// Trait for event store implementations.
pub trait EventStore: Send + Sync {
    /// Append an event to the store.
    fn append(&self, event: Event) -> Result<EventEnvelope>;

    /// Get all events for an aggregate.
    fn get_events(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope>>;

    /// Get a specific event by sequence number.
    fn get_event_by_sequence(&self, sequence: i64) -> Result<EventEnvelope>;

    /// Get the last sequence number.
    fn get_last_sequence(&self) -> Result<i64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_store_trait_bounds() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Box<dyn EventStore>>();
    }
}
