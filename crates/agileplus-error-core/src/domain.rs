use crate::traits::NotFoundMarker;
use thiserror::Error;

/// Domain-rule and aggregate errors for AgilePlus domain layer.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DomainError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("invalid transition: {0}")]
    InvalidTransition(String),

    #[error("internal domain error: {0}")]
    Internal(String),
}

impl NotFoundMarker for DomainError {
    fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }
}

impl From<DomainError> for phenotype_error_core::ErrorKind {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::NotFound(m) => Self::not_found(m),
            DomainError::Conflict(m) => Self::conflict(m),
            DomainError::InvalidTransition(m) => Self::validation(m),
            DomainError::Internal(m) => Self::internal(m),
        }
    }
}
