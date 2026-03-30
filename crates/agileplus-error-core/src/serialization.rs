use thiserror::Error;

/// Serialization and deserialization failures for AgilePlus crates.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum SerializationError {
    #[error("serialization error: {0}")]
    Serialize(String),

    #[error("deserialization error: {0}")]
    Deserialize(String),
}

impl From<SerializationError> for phenotype_error_core::ErrorKind {
    fn from(e: SerializationError) -> Self {
        match e {
            SerializationError::Serialize(m) | SerializationError::Deserialize(m) => {
                Self::serialization(m)
            }
        }
    }
}

impl From<serde_json::Error> for SerializationError {
    fn from(e: serde_json::Error) -> Self {
        Self::Deserialize(e.to_string())
    }
}
