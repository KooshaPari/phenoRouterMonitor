//! T052: Sync Queue with Retry — bounded in-memory queue with exponential backoff.
//!
//! Traceability: WP08-T052

use std::time::Duration;

use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Maximum capacity of the in-memory sync queue.
pub const QUEUE_CAPACITY: usize = 1000;

/// Maximum backoff duration (5 minutes).
pub const MAX_BACKOFF: Duration = Duration::from_secs(300);

/// Base backoff duration (1 second).
pub const BASE_BACKOFF: Duration = Duration::from_secs(1);

/// Errors from queue operations.
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("sync queue is full (capacity {0})")]
    Full(usize),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("serialization error: {0}")]
    Serde(#[from] serde_json::Error),
}

/// Type of operation to retry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SyncOpKind {
    CreateIssue,
    UpdateIssue,
    CreateLabel,
    DeleteIssue,
}

/// A pending sync operation in the retry queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncQueueItem {
    pub id: u64,
    pub kind: SyncOpKind,
    /// Serialized payload (JSON).
    pub payload: String,
    pub attempt: u32,
    pub next_attempt_at: chrono::DateTime<chrono::Utc>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl SyncQueueItem {
    /// Compute the next retry time using exponential backoff.
    ///
    /// Delay = BASE_BACKOFF * 2^attempt, capped at MAX_BACKOFF.
    pub fn next_backoff_delay(attempt: u32) -> Duration {
        let exp = 2u64.saturating_pow(attempt);
        let delay = BASE_BACKOFF.saturating_mul(exp as u32);
        delay.min(MAX_BACKOFF)
    }

    /// Return a new item with incremented attempt and updated next-attempt timestamp.
    pub fn with_next_attempt(&self) -> Self {
        let delay = Self::next_backoff_delay(self.attempt + 1);
        let next = chrono::Utc::now()
            + chrono::Duration::from_std(delay).unwrap_or(chrono::Duration::seconds(300));
        Self {
            attempt: self.attempt + 1,
            next_attempt_at: next,
            ..self.clone()
        }
    }

    pub fn is_ready(&self) -> bool {
        chrono::Utc::now() >= self.next_attempt_at
    }
}

/// Bounded in-memory sync queue with exponential backoff.
#[derive(Debug)]
pub struct SyncQueue {
    items: std::collections::VecDeque<SyncQueueItem>,
    next_id: u64,
}

impl SyncQueue {
    pub fn new() -> Self {
        Self {
            items: std::collections::VecDeque::new(),
            next_id: 1,
        }
    }

    /// Enqueue a new sync operation.
    ///
    /// Returns `QueueError::Full` if the queue has reached `QUEUE_CAPACITY`.
    pub fn enqueue(&mut self, kind: SyncOpKind, payload: String) -> Result<u64, QueueError> {
        if self.items.len() >= QUEUE_CAPACITY {
            return Err(QueueError::Full(QUEUE_CAPACITY));
        }
        let id = self.next_id;
        self.next_id += 1;
        self.items.push_back(SyncQueueItem {
            id,
            kind,
            payload,
            attempt: 0,
            next_attempt_at: chrono::Utc::now(),
            created_at: chrono::Utc::now(),
        });
        Ok(id)
    }

    /// Pop the next item that is ready to be retried.
    pub fn pop_ready(&mut self) -> Option<SyncQueueItem> {
        let pos = self.items.iter().position(|item| item.is_ready())?;
        self.items.remove(pos)
    }

    /// Re-enqueue an item after a failed attempt (with incremented backoff).
    pub fn requeue(&mut self, item: SyncQueueItem) -> Result<(), QueueError> {
        if self.items.len() >= QUEUE_CAPACITY {
            return Err(QueueError::Full(QUEUE_CAPACITY));
        }
        self.items.push_back(item.with_next_attempt());
        Ok(())
    }

    /// Number of items currently in the queue.
    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    /// Drain all items (for persistence on shutdown).
    pub fn drain(&mut self) -> Vec<SyncQueueItem> {
        self.items.drain(..).collect()
    }

    /// Load items back from persistence (on startup).
    pub fn reload(&mut self, items: Vec<SyncQueueItem>) {
        for item in items {
            if self.items.len() < QUEUE_CAPACITY {
                self.items.push_back(item);
            }
        }
    }
}

impl Default for SyncQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// SQLite persistence for the sync queue.
pub struct SyncQueueStore {
    conn: rusqlite::Connection,
}

impl SyncQueueStore {
    /// Open or create the SQLite database.
    pub fn open(path: &str) -> Result<Self, QueueError> {
        let conn = rusqlite::Connection::open(path)?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY,
                kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                attempt INTEGER NOT NULL DEFAULT 0,
                next_attempt_at TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    /// Open an in-memory store (for tests).
    pub fn open_in_memory() -> Result<Self, QueueError> {
        let conn = rusqlite::Connection::open_in_memory()?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS sync_queue (
                id INTEGER PRIMARY KEY,
                kind TEXT NOT NULL,
                payload TEXT NOT NULL,
                attempt INTEGER NOT NULL DEFAULT 0,
                next_attempt_at TEXT NOT NULL,
                created_at TEXT NOT NULL
            );",
        )?;
        Ok(Self { conn })
    }

    /// Persist all items from the queue.
    pub fn save_all(&self, items: &[SyncQueueItem]) -> Result<(), QueueError> {
        self.conn.execute("DELETE FROM sync_queue", [])?;
        for item in items {
            self.conn.execute(
                "INSERT INTO sync_queue (id, kind, payload, attempt, next_attempt_at, created_at) VALUES (?1,?2,?3,?4,?5,?6)",
                rusqlite::params![
                    item.id,
                    serde_json::to_string(&item.kind)?,
                    item.payload,
                    item.attempt,
                    item.next_attempt_at.to_rfc3339(),
                    item.created_at.to_rfc3339(),
                ],
            )?;
        }
        Ok(())
    }

    /// Load all items from the database.
    pub fn load_all(&self) -> Result<Vec<SyncQueueItem>, QueueError> {
        let mut stmt = self.conn.prepare(
            "SELECT id, kind, payload, attempt, next_attempt_at, created_at FROM sync_queue",
        )?;
        let items = stmt
            .query_map([], |row| {
                let id: u64 = row.get::<_, i64>(0)? as u64;
                let kind_str: String = row.get(1)?;
                let payload: String = row.get(2)?;
                let attempt: u32 = row.get::<_, i32>(3)? as u32;
                let next_str: String = row.get(4)?;
                let created_str: String = row.get(5)?;
                Ok((id, kind_str, payload, attempt, next_str, created_str))
            })?
            .filter_map(|r| {
                let (id, kind_str, payload, attempt, next_str, created_str) = r.ok()?;
                let kind: SyncOpKind = serde_json::from_str(&kind_str).ok()?;
                let next_attempt_at = chrono::DateTime::parse_from_rfc3339(&next_str)
                    .ok()?
                    .with_timezone(&chrono::Utc);
                let created_at = chrono::DateTime::parse_from_rfc3339(&created_str)
                    .ok()?
                    .with_timezone(&chrono::Utc);
                Some(SyncQueueItem {
                    id,
                    kind,
                    payload,
                    attempt,
                    next_attempt_at,
                    created_at,
                })
            })
            .collect();
        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enqueue_and_pop() {
        let mut q = SyncQueue::new();
        let id = q.enqueue(SyncOpKind::CreateIssue, "{}".into()).unwrap();
        assert_eq!(q.len(), 1);
        let item = q.pop_ready().unwrap();
        assert_eq!(item.id, id);
        assert_eq!(q.len(), 0);
    }

    #[test]
    fn queue_full_error() {
        let mut q = SyncQueue::new();
        for i in 0..QUEUE_CAPACITY {
            q.enqueue(SyncOpKind::CreateIssue, format!("{}", i)).unwrap();
        }
        let err = q.enqueue(SyncOpKind::CreateIssue, "overflow".into());
        assert!(matches!(err, Err(QueueError::Full(_))));
    }

    #[test]
    fn requeue_increments_attempt() {
        let mut q = SyncQueue::new();
        q.enqueue(SyncOpKind::UpdateIssue, "data".into()).unwrap();
        let item = q.pop_ready().unwrap();
        assert_eq!(item.attempt, 0);
        q.requeue(item).unwrap();
        // The requeued item has attempt=1 and is not immediately ready.
        assert_eq!(q.len(), 1);
    }

    #[test]
    fn backoff_doubles() {
        assert_eq!(SyncQueueItem::next_backoff_delay(0), Duration::from_secs(1));
        assert_eq!(SyncQueueItem::next_backoff_delay(1), Duration::from_secs(2));
        assert_eq!(SyncQueueItem::next_backoff_delay(2), Duration::from_secs(4));
        assert_eq!(SyncQueueItem::next_backoff_delay(3), Duration::from_secs(8));
    }

    #[test]
    fn backoff_capped_at_max() {
        // Attempt 10 → 2^10 = 1024 seconds → capped at 300
        assert_eq!(SyncQueueItem::next_backoff_delay(10), MAX_BACKOFF);
    }

    #[test]
    fn sqlite_roundtrip() {
        let store = SyncQueueStore::open_in_memory().unwrap();
        let mut q = SyncQueue::new();
        q.enqueue(SyncOpKind::CreateLabel, r#"{"name":"bug"}"#.into()).unwrap();
        let items = q.drain();
        store.save_all(&items).unwrap();

        let loaded = store.load_all().unwrap();
        assert_eq!(loaded.len(), 1);
        assert_eq!(loaded[0].kind, SyncOpKind::CreateLabel);
    }

    #[test]
    fn drain_and_reload() {
        let mut q = SyncQueue::new();
        q.enqueue(SyncOpKind::DeleteIssue, "x".into()).unwrap();
        let items = q.drain();
        assert!(q.is_empty());
        q.reload(items);
        assert_eq!(q.len(), 1);
    }
}
