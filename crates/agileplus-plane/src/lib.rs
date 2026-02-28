//! AgilePlus Plane.so sync adapter.
//!
//! One-way sync (AgilePlus → Plane.so): features map to issues,
//! work packages map to sub-issues. Conflict detection prevents
//! overwriting user edits on the Plane.so side.
//!
//! Traceability: FR-051 / WP18

pub mod client;
pub mod sync;

pub use client::PlaneClient;
pub use sync::{PlaneSyncAdapter, SyncState};
