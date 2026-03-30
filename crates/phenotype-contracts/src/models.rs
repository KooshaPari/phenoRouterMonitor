//! Domain models, aggregates, and value objects.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A domain entity with identity.
pub trait DomainEntity: Send + Sync {
    type Id: Clone + Send + Sync;
    fn id(&self) -> Self::Id;
}

/// A value object - immutable, identified by its values.
pub trait ValueObject: Clone + PartialEq + Send + Sync {}

/// An aggregate root - boundary for transactional consistency.
pub trait AggregateRoot: Send + Sync {
    type Id: Clone + Send + Sync;
    fn id(&self) -> Self::Id;
}

/// A domain event.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DomainEvent {
    pub id: Uuid,
    pub aggregate_id: String,
    pub event_type: String,
    pub timestamp: DateTime<Utc>,
    pub data: serde_json::Value,
}

impl DomainEvent {
    pub fn new(aggregate_id: String, event_type: String, data: serde_json::Value) -> Self {
        Self {
            id: Uuid::new_v4(),
            aggregate_id,
            event_type,
            timestamp: Utc::now(),
            data,
        }
    }
}
