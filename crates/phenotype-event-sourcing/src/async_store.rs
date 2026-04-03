//! Async event store trait for async runtime compatibility.

use async_trait::async_trait;

use crate::error::EventSourcingError;
use crate::event::EventEnvelope;

#[async_trait]
pub trait AsyncEventStore<T>: Send + Sync
where
    T: Send + Sync + serde::Serialize + for<'de> serde::Deserialize<'de> + 'static,
{
    async fn append(&self, event: &EventEnvelope<T>) -> Result<i64, EventSourcingError>;

    async fn get(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, EventSourcingError>;

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Option<i64>, EventSourcingError>;

    async fn verify_chain(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<bool, EventSourcingError>;

    async fn count(&self, entity_type: &str) -> Result<usize, EventSourcingError>;
}
