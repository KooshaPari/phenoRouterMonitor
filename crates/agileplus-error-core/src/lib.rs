//! Unified error enums for AgilePlus, bridgeable to [`phenotype_error_core::ErrorKind`].

pub mod api;
pub mod domain;
pub mod serialization;
pub mod storage;
pub mod sync;
pub mod traits;

pub use api::ApiError;
pub use domain::DomainError;
pub use serialization::SerializationError;
pub use storage::StorageError;
pub use sync::SyncError;
pub use traits::NotFoundMarker;
