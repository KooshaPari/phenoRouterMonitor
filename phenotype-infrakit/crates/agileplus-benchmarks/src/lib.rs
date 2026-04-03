//! AgilePlus performance benchmarks.
//!
//! This crate contains Criterion benchmarks for all major subsystems:
//! - T116: Event append throughput (SQLite WAL, sequential appends)
//! - T117: Event replay and snapshot rebuild performance
//! - T118: API response time benchmarks (in-process handler calls)
//! - T119: Sync vector / round-trip operations
//! - T120: Graph query performance (in-memory backend)

pub mod helpers;
