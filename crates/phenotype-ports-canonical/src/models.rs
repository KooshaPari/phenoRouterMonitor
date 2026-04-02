//! Domain models for hexagonal architecture

use serde::{Deserialize, Serialize};

pub trait Entity: Send + Sync {
    fn id(&self) -> EntityId;
}

pub trait ValueObject: Send + Sync + Clone + PartialEq {}

pub trait AggregateRoot: Entity + Send + Sync {
    fn version(&self) -> u64;
}

pub trait DomainEvent: Send + Sync + Clone {
    fn event_type(&self) -> &'static str;
    fn aggregate_id(&self) -> &str;
    fn timestamp(&self) -> chrono::DateTime<chrono::Utc>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct EntityId(pub String);

impl EntityId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
}
