//! SQLite-backed persistent event store with hash chain verification.
//!
//! Stores events in a SQLite database with full hash chain integrity verification.
//! Each event is immutable and linked to its predecessor via blake3 hash.

use crate::error::{EventSourcingError, Result};
use crate::hash::{compute_hash, verify_chain};
use crate::{EventEnvelope, EventStore};
use async_trait::async_trait;
use std::path::Path;
use std::sync::Arc;

/// SQLite-backed event store with persistent storage.
pub struct SqliteEventStore {
    conn: Arc<rusqlite::Connection>,
}

impl SqliteEventStore {
    /// Open or create a SQLite event store at the given path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let conn = rusqlite::Connection::open(path)
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;

        // Create tables if they don't exist
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS events (
                id INTEGER PRIMARY KEY,
                aggregate_id TEXT NOT NULL,
                event_type TEXT NOT NULL,
                payload BLOB NOT NULL,
                hash TEXT NOT NULL UNIQUE,
                previous_hash TEXT NOT NULL,
                timestamp INTEGER NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP
            );
            CREATE INDEX IF NOT EXISTS idx_aggregate_id ON events(aggregate_id);
            CREATE INDEX IF NOT EXISTS idx_hash ON events(hash);",
        )
        .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;

        Ok(Self {
            conn: Arc::new(conn),
        })
    }

    /// Get event count for an aggregate.
    pub fn count_events(&self, aggregate_id: &str) -> Result<u64> {
        let count: u64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM events WHERE aggregate_id = ?1",
                [aggregate_id],
                |row| row.get(0),
            )
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;
        Ok(count)
    }
}

#[async_trait]
impl EventStore for SqliteEventStore {
    async fn append(&self, envelope: EventEnvelope) -> Result<()> {
        let payload = serde_json::to_vec(&envelope.payload)
            .map_err(|e| EventSourcingError::SerializationError(e.to_string()))?;

        self.conn
            .execute(
                "INSERT INTO events (aggregate_id, event_type, payload, hash, previous_hash, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                [
                    &envelope.aggregate_id,
                    &envelope.event_type,
                    &String::from_utf8_lossy(&payload),
                    &envelope.hash,
                    &envelope.previous_hash,
                    &envelope.timestamp.to_string(),
                ],
            )
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;

        Ok(())
    }

    async fn load(&self, aggregate_id: &str) -> Result<Vec<EventEnvelope>> {
        let mut stmt = self
            .conn
            .prepare("SELECT payload, hash, previous_hash, timestamp FROM events WHERE aggregate_id = ?1 ORDER BY id ASC")
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;

        let events = stmt
            .query_map([aggregate_id], |row| {
                let payload: Vec<u8> = row.get(0)?;
                let hash: String = row.get(1)?;
                let previous_hash: String = row.get(2)?;
                let timestamp: i64 = row.get(3)?;

                Ok((payload, hash, previous_hash, timestamp))
            })
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|e| EventSourcingError::StorageError(e.to_string()))?;

        // Reconstruct envelopes with verification
        let mut result = Vec::new();
        for (payload, hash, previous_hash, timestamp) in events {
            let event_payload: serde_json::Value = serde_json::from_slice(&payload)
                .map_err(|e| EventSourcingError::SerializationError(e.to_string()))?;

            result.push(EventEnvelope {
                aggregate_id: aggregate_id.to_string(),
                event_type: event_payload
                    .get("type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown")
                    .to_string(),
                payload: event_payload,
                hash,
                previous_hash,
                timestamp,
            });
        }

        Ok(result)
    }

    async fn verify_integrity(&self, aggregate_id: &str) -> Result<bool> {
        let events = self.load(aggregate_id).await?;
        Ok(verify_chain(&events))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_sqlite_append_and_load() {
        let dir = TempDir::new().unwrap();
        let store = SqliteEventStore::open(dir.path().join("events.db")).unwrap();

        let envelope = EventEnvelope {
            aggregate_id: "test-1".to_string(),
            event_type: "Created".to_string(),
            payload: json!({"type": "Created", "name": "Test"}),
            hash: "abc123".to_string(),
            previous_hash: "0".to_string(),
            timestamp: 1000,
        };

        store.append(envelope.clone()).await.unwrap();

        let loaded = store.load("test-1").await.unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].aggregate_id, "test-1");
    }

    #[tokio::test]
    async fn test_sqlite_count_events() {
        let dir = TempDir::new().unwrap();
        let store = SqliteEventStore::open(dir.path().join("events.db")).unwrap();

        for i in 0..3 {
            let envelope = EventEnvelope {
                aggregate_id: "test-2".to_string(),
                event_type: "Updated".to_string(),
                payload: json!({"type": "Updated", "value": i}),
                hash: format!("hash-{}", i),
                previous_hash: if i == 0 { "0".to_string() } else { format!("hash-{}", i - 1) },
                timestamp: 1000 + i as i64,
            };
            store.append(envelope).await.unwrap();
        }

        let count = store.count_events("test-2").unwrap();
        assert_eq!(count, 3);
    }
}
