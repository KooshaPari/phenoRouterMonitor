use agileplus_domain::domain::event::Event;
use agileplus_events::{EventError, EventStore};

use crate::adapter::SqliteStorageAdapter;
use crate::repository::events;

#[async_trait::async_trait]
impl EventStore for SqliteStorageAdapter {
    async fn append(&self, event: &Event) -> Result<i64, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::append_event(&conn, event).map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events_since(
        &self,
        entity_type: &str,
        entity_id: i64,
        sequence: i64,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events_since(&conn, entity_type, entity_id, sequence)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_events_by_range(
        &self,
        entity_type: &str,
        entity_id: i64,
        from: chrono::DateTime<chrono::Utc>,
        to: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<Event>, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_events_by_range(
            &conn,
            entity_type,
            entity_id,
            &from.to_rfc3339(),
            &to.to_rfc3339(),
        )
        .map_err(|e| EventError::StorageError(e.to_string()))
    }

    async fn get_latest_sequence(
        &self,
        entity_type: &str,
        entity_id: i64,
    ) -> Result<i64, EventError> {
        let conn = self
            .lock()
            .map_err(|e| EventError::StorageError(e.to_string()))?;
        events::get_latest_sequence(&conn, entity_type, entity_id)
            .map_err(|e| EventError::StorageError(e.to_string()))
    }
}
