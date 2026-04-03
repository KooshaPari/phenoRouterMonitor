//! EventStore trait - generic append-only event storage.

use crate::error::Result;
use crate::event::EventEnvelope;

pub trait EventStore: Send + Sync {
    fn append(&self, event: &EventEnvelope<serde_json::Value>) -> Result<i64>;

    fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<serde_json::Value>>>;

    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;

    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}
