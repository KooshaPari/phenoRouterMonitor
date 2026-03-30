//! Event store trait.

use async_trait::async_trait;
use serde::{de::DeserializeOwned, Serialize};

use crate::error::Error;
use crate::event::EventEnvelope;

#[async_trait]
pub trait EventStore<T: Clone + Send + Sync + Serialize + DeserializeOwned + 'static>:
    Send + Sync
{
    async fn append(
        &self,
        entity_type: &str,
        entity_id: &str,
        event: EventEnvelope<T>,
    ) -> Result<i64, Error>;

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Vec<EventEnvelope<T>>, Error>;

    async fn get_sequence(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<i64, Error>;
}
