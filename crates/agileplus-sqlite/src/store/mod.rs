//! SQLite storage adapter modules.
//!
//! Organizes persistence layer into focused concerns:
//! - `sync.rs` — Synchronized connection management and locking
//! - `query_builder.rs` — SQL generation patterns and utilities
//! - `migrations.rs` — Schema management and versioning

pub mod sync;

// Re-exports for convenience
pub use sync::{SqliteStorageAdapter, SqliteConnectionPool};
