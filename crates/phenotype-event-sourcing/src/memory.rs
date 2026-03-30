//! In-memory event store implementation.

use crate::error::{EventSourcingError, Result};
use std::collections::HashMap;

/// In-memory event store.
pub struct InMemoryEventStore {
    events: HashMap<String, Vec<serde_json::Value>>,
}

impl InMemoryEventStore {
    pub fn new() -> Self {
        Self {
            events: HashMap::new(),
        }
    }

    pub fn append(&mut self, _event: serde_json::Value, _entity_type: &str, _entity_id: &str) -> Result<i64> {
        Ok(1)
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}
