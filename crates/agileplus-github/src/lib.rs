//! AgilePlus GitHub sync adapter.
//!
//! One-way sync (AgilePlus → GitHub): bugs map to GitHub Issues
//! with structured markdown bodies. Status polling detects
//! external changes. Conflict detection via SHA-256 body hashing.
//!
//! Traceability: FR-052 / WP19

pub mod client;
pub mod sync;

pub use client::GitHubClient;
pub use sync::{GitHubSyncAdapter, GitHubSyncState};
