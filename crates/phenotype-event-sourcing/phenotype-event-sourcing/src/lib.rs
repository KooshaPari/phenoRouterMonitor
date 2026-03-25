//! Generic event sourcing engine for Phenotype.
//!
//! Provides append-only event storage with SHA-256 hash chain verification,
//! snapshot management, and flexible event handling for any serializable event type.
//!
//! # Examples
//!
//! ```ignore
//! use phenotype_event_sourcing::{EventEnvelope, EventStore};
//! use phenotype_event_sourcing::memory::InMemoryEventStore;
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Serialize, Deserialize)]
//! struct UserCreated {
//!     user_id: String,
//!     email: String,
//! }
//!
//! let store = InMemoryEventStore::new();
//! let event = EventEnvelope::new(
//!     UserCreated {
//!         user_id: "user-123".to_string(),
//!         email: "user@example.com".to_string(),
//!     },
//!     "system",
//! );
//! let sequence = store.append(&event, "UserCreated").unwrap();
//! ```

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, EventStoreError, HashError, Result};
pub use event::EventEnvelope;
pub use hash::{compute_hash, detect_gaps, verify_chain};
pub use memory::InMemoryEventStore;
pub use snapshot::{should_snapshot, Snapshot, SnapshotConfig};
pub use store::EventStore;
