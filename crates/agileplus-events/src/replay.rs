//! Event replay engine with Aggregate pattern.

use agileplus_domain::domain::event::Event;
use async_trait::async_trait;

#[derive(Debug, thiserror::Error)]
pub enum ReplayError {
    #[error("Aggregate error: {0}")]
    AggregateError(String),
    #[error("Invalid state: {0}")]
    InvalidState(String),
}

/// Aggregate trait: any entity that can be reconstructed from events.
#[async_trait]
pub trait Aggregate: Send + Sync {
    /// Apply an event to update aggregate state. Must be idempotent for same event sequence.
    async fn apply(&mut self, event: &Event) -> Result<(), ReplayError>;

    /// Current version (latest applied event sequence).
    fn version(&self) -> i64;

    /// Set version after loading from snapshot.
    fn set_version(&mut self, version: i64);
}

/// Replay a sequence of events onto an aggregate.
pub async fn replay_events<A: Aggregate>(
    aggregate: &mut A,
    events: &[Event],
) -> Result<(), ReplayError> {
    if !events.is_empty() {
        let first_id = events[0].entity_id;
        for event in events {
            if event.entity_id != first_id {
                return Err(ReplayError::InvalidState(
                    "Events from different entities in replay".into(),
                ));
            }
        }
    }

    for event in events {
        aggregate.apply(event).await?;
    }

    if let Some(last) = events.last() {
        aggregate.set_version(last.sequence);
    }

    Ok(())
}

/// Replay only events after a snapshot sequence.
pub async fn replay_events_since<A: Aggregate>(
    aggregate: &mut A,
    snapshot_sequence: i64,
    events: &[Event],
) -> Result<(), ReplayError> {
    let filtered: Vec<_> = events
        .iter()
        .filter(|e| e.sequence > snapshot_sequence)
        .cloned()
        .collect();
    replay_events(aggregate, &filtered).await
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAggregate {
        version: i64,
        state: serde_json::Value,
    }

    #[async_trait]
    impl Aggregate for TestAggregate {
        async fn apply(&mut self, event: &Event) -> Result<(), ReplayError> {
            self.state = event.payload.clone();
            self.version = event.sequence;
            Ok(())
        }
        fn version(&self) -> i64 { self.version }
        fn set_version(&mut self, v: i64) { self.version = v; }
    }

    fn make_event(seq: i64, entity_id: i64, payload: serde_json::Value) -> Event {
        Event {
            id: seq, entity_type: "T".into(), entity_id, event_type: "U".into(),
            payload, actor: "t".into(), timestamp: chrono::Utc::now(),
            prev_hash: [0u8; 32], hash: [0u8; 32], sequence: seq,
        }
    }

    #[tokio::test]
    async fn replay_applies_events() {
        let mut agg = TestAggregate { version: 0, state: serde_json::json!({}) };
        let events = vec![
            make_event(1, 1, serde_json::json!({"v": 1})),
            make_event(2, 1, serde_json::json!({"v": 2})),
        ];
        replay_events(&mut agg, &events).await.unwrap();
        assert_eq!(agg.version, 2);
        assert_eq!(agg.state, serde_json::json!({"v": 2}));
    }

    #[tokio::test]
    async fn replay_rejects_mixed_entities() {
        let mut agg = TestAggregate { version: 0, state: serde_json::json!({}) };
        let events = vec![
            make_event(1, 1, serde_json::json!({})),
            make_event(2, 2, serde_json::json!({})),
        ];
        assert!(replay_events(&mut agg, &events).await.is_err());
    }

    #[tokio::test]
    async fn replay_since_filters() {
        let mut agg = TestAggregate { version: 0, state: serde_json::json!({}) };
        let events = vec![
            make_event(1, 1, serde_json::json!({"v": 1})),
            make_event(2, 1, serde_json::json!({"v": 2})),
            make_event(3, 1, serde_json::json!({"v": 3})),
        ];
        replay_events_since(&mut agg, 2, &events).await.unwrap();
        assert_eq!(agg.version, 3);
        assert_eq!(agg.state, serde_json::json!({"v": 3}));
    }
}
