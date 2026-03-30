//! Unified error types for AgilePlus crates.
//!
//! Shapes follow `docs/worklogs/PLANS/ErrorCoreExtraction.md`. Cross-ecosystem
//! [`ErrorKind`](phenotype_error_core::ErrorKind) lives in **`phenotype-error-core`**; use `Into` /
//! `From` below to bridge AgilePlus errors into shared handlers and telemetry.

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

/// Re-export of Phenotype-wide structured error kind.
pub use phenotype_error_core::ErrorKind;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn storage_maps_to_error_kind() {
        let e: ErrorKind = StorageError::not_found("x").into();
        assert_eq!(e.kind(), "NotFound");
    }

    #[test]
    fn serde_json_maps_to_serialization_error() {
        let err = serde_json::from_str::<serde_json::Value>("not-json").unwrap_err();
        let se: SerializationError = err.into();
        assert!(matches!(se, SerializationError::Deserialize(_)));
    }
}
