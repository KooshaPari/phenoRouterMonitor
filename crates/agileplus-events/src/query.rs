//! Event query builder with fluent API.

use agileplus_domain::domain::event::Event;
use chrono::{DateTime, Utc};

#[derive(Debug, thiserror::Error)]
pub enum QueryError {
    #[error("Query error: {0}")]
    Error(String),
}

/// Fluent event query builder for in-memory filtering.
#[derive(Default)]
pub struct EventQuery {
    entity_type: Option<String>,
    entity_id: Option<i64>,
    event_type: Option<String>,
    actor: Option<String>,
    from_time: Option<DateTime<Utc>>,
    to_time: Option<DateTime<Utc>>,
    from_sequence: Option<i64>,
    to_sequence: Option<i64>,
    limit: Option<usize>,
}

impl EventQuery {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn entity_type(mut self, et: impl Into<String>) -> Self {
        self.entity_type = Some(et.into());
        self
    }

    pub fn entity_id(mut self, id: i64) -> Self {
        self.entity_id = Some(id);
        self
    }

    pub fn event_type(mut self, et: impl Into<String>) -> Self {
        self.event_type = Some(et.into());
        self
    }

    pub fn actor(mut self, a: impl Into<String>) -> Self {
        self.actor = Some(a.into());
        self
    }

    pub fn from_time(mut self, t: DateTime<Utc>) -> Self {
        self.from_time = Some(t);
        self
    }

    pub fn to_time(mut self, t: DateTime<Utc>) -> Self {
        self.to_time = Some(t);
        self
    }

    pub fn from_sequence(mut self, s: i64) -> Self {
        self.from_sequence = Some(s);
        self
    }

    pub fn to_sequence(mut self, s: i64) -> Self {
        self.to_sequence = Some(s);
        self
    }

    pub fn limit(mut self, l: usize) -> Self {
        self.limit = Some(l);
        self
    }

    /// Filter an in-memory event list using this query's criteria.
    pub fn filter(&self, events: &[Event]) -> Vec<Event> {
        events
            .iter()
            .filter(|e| {
                if let Some(ref et) = self.entity_type {
                    if e.entity_type != *et { return false; }
                }
                if let Some(id) = self.entity_id {
                    if e.entity_id != id { return false; }
                }
                if let Some(ref et) = self.event_type {
                    if e.event_type != *et { return false; }
                }
                if let Some(ref a) = self.actor {
                    if e.actor != *a { return false; }
                }
                if let Some(from) = self.from_time {
                    if e.timestamp < from { return false; }
                }
                if let Some(to) = self.to_time {
                    if e.timestamp > to { return false; }
                }
                if let Some(from) = self.from_sequence {
                    if e.sequence < from { return false; }
                }
                if let Some(to) = self.to_sequence {
                    if e.sequence > to { return false; }
                }
                true
            })
            .take(self.limit.unwrap_or(usize::MAX))
            .cloned()
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_event(seq: i64, entity_type: &str, event_type: &str, actor: &str) -> Event {
        Event {
            id: seq, entity_type: entity_type.into(), entity_id: 1,
            event_type: event_type.into(), payload: serde_json::json!({}),
            actor: actor.into(), timestamp: chrono::Utc::now(),
            prev_hash: [0u8; 32], hash: [0u8; 32], sequence: seq,
        }
    }

    #[test]
    fn filter_by_entity_type() {
        let events = vec![
            make_event(1, "Feature", "created", "a"),
            make_event(2, "WP", "created", "a"),
        ];
        let result = EventQuery::new().entity_type("Feature").filter(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].entity_type, "Feature");
    }

    #[test]
    fn filter_by_actor() {
        let events = vec![
            make_event(1, "F", "c", "alice"),
            make_event(2, "F", "c", "bob"),
        ];
        let result = EventQuery::new().actor("bob").filter(&events);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn limit_works() {
        let events = vec![
            make_event(1, "F", "c", "a"),
            make_event(2, "F", "c", "a"),
            make_event(3, "F", "c", "a"),
        ];
        let result = EventQuery::new().limit(2).filter(&events);
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn sequence_range() {
        let events = vec![
            make_event(1, "F", "c", "a"),
            make_event(2, "F", "c", "a"),
            make_event(3, "F", "c", "a"),
        ];
        let result = EventQuery::new().from_sequence(2).to_sequence(2).filter(&events);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].sequence, 2);
    }
}
