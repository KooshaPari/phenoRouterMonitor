//! Synchronized connection pool and locking primitives.
//!
//! Manages single write-serialized connection protected by Mutex.
//! WAL mode allows concurrent reads; all writes are serialized.
//! Traceability: WP06

use std::path::Path;
use std::sync::{Arc, Mutex};

use rusqlite::Connection;

use agileplus_domain::error::DomainError;

use crate::migrations::MigrationRunner;

/// SQLite-backed storage adapter.
///
/// Uses a single write-serialized connection protected by a Mutex.
/// WAL mode is enabled to allow concurrent reads; all writes are serialized.
pub struct SqliteStorageAdapter {
    conn: Arc<Mutex<Connection>>,
}

/// Alias for connection pool (currently 1-connection impl).
pub type SqliteConnectionPool = Arc<Mutex<Connection>>;

impl SqliteStorageAdapter {
    /// Open a file-backed database, enable WAL + FK pragma, and run all migrations.
    pub fn new(db_path: &Path) -> Result<Self, DomainError> {
        let conn = Connection::open(db_path)
            .map_err(|e| DomainError::Storage(format!("failed to open db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    /// Open an in-memory database (for tests).
    pub fn in_memory() -> Result<Self, DomainError> {
        let conn = Connection::open_in_memory()
            .map_err(|e| DomainError::Storage(format!("failed to open in-memory db: {e}")))?;
        Self::configure_and_migrate(conn)
    }

    /// Get the inner connection pool for advanced access.
    pub fn pool(&self) -> SqliteConnectionPool {
        Arc::clone(&self.conn)
    }

    /// Configure database pragmas and run migrations.
    fn configure_and_migrate(conn: Connection) -> Result<Self, DomainError> {
        Self::apply_pragmas(&conn)?;
        Self::run_migrations(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    /// Apply SQLite pragmas for performance and safety.
    fn apply_pragmas(conn: &Connection) -> Result<(), DomainError> {
        // Enable WAL mode for concurrent reads
        conn.execute_batch("PRAGMA journal_mode=WAL;")
            .map_err(|e| DomainError::Storage(format!("WAL pragma failed: {e}")))?;

        // Enable foreign key enforcement
        conn.execute_batch("PRAGMA foreign_keys=ON;")
            .map_err(|e| DomainError::Storage(format!("FK pragma failed: {e}")))?;

        Ok(())
    }

    /// Run all migrations.
    fn run_migrations(conn: &Connection) -> Result<(), DomainError> {
        let runner = MigrationRunner::new(conn);
        runner.run_all()?;
        Ok(())
    }

    /// Get a locked guard to the connection.
    pub fn lock(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.conn
            .lock()
            .map_err(|e| DomainError::Storage(format!("mutex poisoned: {e}")))
    }

    /// Expose a locked connection guard for benchmarks and test helpers.
    ///
    /// This method is intentionally public so that benchmark crates can access
    /// the underlying rusqlite `Connection` to call repository functions directly
    /// without going through the async `StoragePort` trait.
    pub fn conn_for_bench(&self) -> Result<std::sync::MutexGuard<'_, Connection>, DomainError> {
        self.lock()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_in_memory_creation() {
        let adapter = SqliteStorageAdapter::in_memory();
        assert!(adapter.is_ok());
        let adapter = adapter.unwrap();
        let _lock = adapter.lock();
        assert!(_lock.is_ok());
    }

    #[test]
    fn adapter_lock_serialization() {
        let adapter = SqliteStorageAdapter::in_memory().unwrap();
        let _lock1 = adapter.lock().unwrap();
        // Lock is held; trying to lock again would block (test validates no deadlock)
        drop(_lock1);
        let _lock2 = adapter.lock();
        assert!(_lock2.is_ok());
    }
}
