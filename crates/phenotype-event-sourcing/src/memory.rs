//! In-memory event store implementation.

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

    pub fn append(&mut self, _event: serde_json::Value, _entity: &str) -> i64 {
        1
    }
}

impl Default for InMemoryEventStore {
    fn default() -> Self {
        Self::new()
    }
}
