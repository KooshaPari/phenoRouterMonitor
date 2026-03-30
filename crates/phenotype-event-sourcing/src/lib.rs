//! # Phenotype Event Sourcing
//!
//! Event sourcing primitives for phenotype-infrakit.
//!
//! ## Serialization
//!
//! Events can be serialized to multiple formats via the [`serializer`] module:
//! - **JSON** (default, human-readable)
//! - **Binary** (compact, fast)
//!
//! Use the [`serializer::SerializerRegistry`] for format-agnostic serialization:
//!
//! ```ignore
//! let registry = SerializerRegistry::new();
//! let bytes = registry.serialize(&envelope, SerializationFormat::Json)?;
//! let restored = registry.deserialize_auto(&bytes)?;
//! ```

pub mod error;
pub mod event;
pub mod hash;
pub mod memory;
pub mod serializer;
pub mod snapshot;
pub mod store;

pub use error::{EventSourcingError, EventStoreError, HashError, Result};
pub use event::EventEnvelope;
pub use hash::{compute_hash, verify_chain};
pub use memory::InMemoryEventStore;
pub use serializer::{EventSerializer, SerializationFormat, SerializerRegistry};
pub use store::{EventStore, JsonEnvelope};
