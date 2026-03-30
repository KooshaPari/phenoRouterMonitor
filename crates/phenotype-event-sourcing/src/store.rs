//! EventStore trait — generic append-only event storage.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::event::EventEnvelope;

/// Generic event store for append-only event storage with hash chain support.
pub trait EventStore<T: Send + Sync = serde_json::Value>: Send + Sync {
    fn append(&self, event: &EventEnvelope<T>, entity_type: &str, entity_id: &str) -> Result<i64>;
    fn get_events(&self, entity_type: &str, entity_id: &str) -> Result<Vec<EventEnvelope<T>>>;
    fn get_events_since(&self, entity_type: &str, entity_id: &str, sequence: i64) -> Result<Vec<EventEnvelope<T>>>;
    fn get_events_by_range(&self, entity_type: &str, entity_id: &str, from: DateTime<Utc>, to: DateTime<Utc>) -> Result<Vec<EventEnvelope<T>>>;
    fn get_latest_sequence(&self, entity_type: &str, entity_id: &str) -> Result<i64>;
    fn verify_chain(&self, entity_type: &str, entity_id: &str) -> Result<()>;
}
