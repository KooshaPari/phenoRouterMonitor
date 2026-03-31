//! AgilePlus contract tests — trait-based contract verification across crate boundaries.
//!
//! This crate houses contract tests verifying that provider implementations
//! satisfy the contracts expected by consumers at each crate boundary:
//!
//! - T112: agileplus-events ↔ agileplus-sqlite (EventStore trait contract)
//! - T113: agileplus-sync ↔ agileplus-plane (PlaneClient / state-mapper contract)
//! - T114: agileplus-api ↔ agileplus-dashboard (API response shape contract)
//! - T115: agileplus-api ↔ agileplus-events (EventQuery / EventStore consumer contract)
//!
//! Traceability: WP20 / T112, T113, T114, T115
