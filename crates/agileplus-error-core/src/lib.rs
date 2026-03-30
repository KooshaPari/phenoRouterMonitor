<<<<<<< HEAD
//! Unified error enums for AgilePlus, bridgeable to [`phenotype_error_core::ErrorKind`].
=======
//! Unified error handling for the AgilePlus ecosystem.
>>>>>>> origin/main

pub mod api;
pub mod domain;
pub mod serialization;
pub mod storage;
pub mod sync;
pub mod traits;

pub use api::ApiError;
pub use domain::DomainError;
<<<<<<< HEAD
=======
pub use phenotype_error_core::ErrorKind;
>>>>>>> origin/main
pub use serialization::SerializationError;
pub use storage::StorageError;
pub use sync::SyncError;
pub use traits::NotFoundMarker;
<<<<<<< HEAD
=======

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_error_maps_to_error_kind() {
        let e = StorageError::NotFound("wp/1".into());
        let k: ErrorKind = e.into();
        assert_eq!(k.kind(), "NotFound");
    }

    #[test]
    fn serde_json_error_maps_via_serialization_error() {
        let j = serde_json::from_str::<i32>("not-json").unwrap_err();
        let s: SerializationError = j.into();
        let k: ErrorKind = s.into();
        assert_eq!(k.kind(), "Serialization");
    }
}
>>>>>>> origin/main
