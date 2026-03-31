use thiserror::Error;

#[derive(Debug, Error)]
pub enum EventSourcingError {
    #[error("store error: {0}")]
    Store(String),
    #[error("hash mismatch: expected {expected}, got {actual}")]
    HashMismatch { expected: String, actual: String },
    #[error("invalid hash length: {0}")]
    InvalidHashLength(usize),
    #[error("hex decode error: {0}")]
    HexDecode(String),
    #[error("event not found: {0}")]
    EventNotFound(String),
    #[error("sequence error: {0}")]
    Sequence(String),
    #[error("serialization error: {0}")]
    Serialization(String),
}

impl From<serde_json::Error> for EventSourcingError {
    fn from(err: serde_json::Error) -> Self {
        Self::Serialization(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = EventSourcingError::Store("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_hash_mismatch() {
        let err = EventSourcingError::HashMismatch {
            expected: "abc".to_string(),
            actual: "def".to_string(),
        };
        assert!(err.to_string().contains("abc"));
        assert!(err.to_string().contains("def"));
    }
}
