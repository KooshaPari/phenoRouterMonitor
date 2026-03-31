//! AgilePlus SQLite adapter — persistence layer.
//!
//! Implements `StoragePort` using rusqlite with WAL mode and foreign keys.
//! Traceability: WP06

pub mod migrations;
pub mod rebuild;
pub mod repository;
pub mod event_store;

pub mod lib {
    pub mod adapter;
    pub mod storage_port;
    pub mod content_storage;
    pub mod tests;
}

pub use adapter::SqliteStorageAdapter;