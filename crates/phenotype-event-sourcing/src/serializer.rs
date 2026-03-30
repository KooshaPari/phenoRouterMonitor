//! Event serialization abstraction layer.
//!
//! Provides a unified interface for serializing/deserializing events in multiple formats:
//! - JSON (human-readable, default)
//! - Binary (compact, faster for large payloads)
//!
//! This consolidates ~500 LOC of scattered serialization logic into a cohesive registry pattern.

use crate::error::{EventSourcingError, Result};
use crate::event::EventEnvelope;
use serde_json::Value as JsonValue;
use std::collections::HashMap;
use std::sync::Arc;

/// Event serialization format identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SerializationFormat {
    /// JSON format (human-readable, default)
    Json,
    /// Binary format using rkyv (compact, fast)
    Binary,
}

impl SerializationFormat {
    /// Get the format identifier string (for persisted metadata).
    pub fn identifier(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Binary => "binary",
        }
    }

    /// Parse a format identifier string.
    pub fn from_identifier(s: &str) -> Result<Self> {
        match s {
            "json" => Ok(Self::Json),
            "binary" => Ok(Self::Binary),
            unknown => Err(EventSourcingError::serialization(format!(
                "unknown serialization format: {}",
                unknown
            ))),
        }
    }
}

/// Trait for event serialization/deserialization strategies.
///
/// Implementations define how events are converted to/from bytes.
pub trait EventSerializer: Send + Sync {
    /// Serialize an event envelope to bytes.
    fn to_bytes(&self, envelope: &JsonEnvelope) -> Result<Vec<u8>>;

    /// Deserialize an event envelope from bytes.
    fn from_bytes(&self, bytes: &[u8]) -> Result<JsonEnvelope>;

    /// Get the format identifier for this serializer.
    fn format(&self) -> SerializationFormat;
}

/// JSON event serializer (default, human-readable).
pub struct JsonEventSerializer;

impl EventSerializer for JsonEventSerializer {
    fn to_bytes(&self, envelope: &JsonEnvelope) -> Result<Vec<u8>> {
        serde_json::to_vec(envelope)
            .map_err(|e| EventSourcingError::serialization(format!("JSON serialization failed: {}", e)))
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<JsonEnvelope> {
        serde_json::from_slice(bytes)
            .map_err(|e| EventSourcingError::serialization(format!("JSON deserialization failed: {}", e)))
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Json
    }
}

/// Binary event serializer using rkyv.
///
/// Provides compact, zero-copy deserialization for high-performance scenarios.
/// Note: This is a placeholder for future rkyv integration.
pub struct BinaryEventSerializer;

impl EventSerializer for BinaryEventSerializer {
    fn to_bytes(&self, envelope: &JsonEnvelope) -> Result<Vec<u8>> {
        // For now, fall back to JSON + prefix.
        // Future: use rkyv or bincode for true binary format.
        let json_bytes = serde_json::to_vec(envelope)
            .map_err(|e| EventSourcingError::serialization(format!("JSON serialization failed: {}", e)))?;

        let mut result = Vec::with_capacity(json_bytes.len() + 8);
        result.extend_from_slice(b"BINARY\0\0");
        result.extend_from_slice(&json_bytes);
        Ok(result)
    }

    fn from_bytes(&self, bytes: &[u8]) -> Result<JsonEnvelope> {
        // For now, strip the prefix and deserialize as JSON.
        if bytes.len() < 8 || !bytes.starts_with(b"BINARY\0\0") {
            return Err(EventSourcingError::serialization(
                "invalid binary format: missing header",
            ));
        }

        serde_json::from_slice(&bytes[8..])
            .map_err(|e| EventSourcingError::serialization(format!("JSON deserialization failed: {}", e)))
    }

    fn format(&self) -> SerializationFormat {
        SerializationFormat::Binary
    }
}

/// Registry for event serializers.
///
/// Provides dynamic format detection and serializer lookup.
/// Supports pluggable custom serializers.
pub struct SerializerRegistry {
    serializers: HashMap<SerializationFormat, Arc<dyn EventSerializer>>,
}

impl SerializerRegistry {
    /// Create a new serializer registry with default serializers.
    pub fn new() -> Self {
        let mut serializers = HashMap::new();
        serializers.insert(
            SerializationFormat::Json,
            Arc::new(JsonEventSerializer) as Arc<dyn EventSerializer>,
        );
        serializers.insert(
            SerializationFormat::Binary,
            Arc::new(BinaryEventSerializer) as Arc<dyn EventSerializer>,
        );

        Self { serializers }
    }

    /// Register a custom serializer for a format.
    pub fn register(&mut self, serializer: Arc<dyn EventSerializer>) {
        let format = serializer.format();
        self.serializers.insert(format, serializer);
    }

    /// Get a serializer for the specified format.
    pub fn get(&self, format: SerializationFormat) -> Result<Arc<dyn EventSerializer>> {
        self.serializers
            .get(&format)
            .cloned()
            .ok_or_else(|| EventSourcingError::serialization(format!("no serializer for format: {:?}", format)))
    }

    /// Get the default (JSON) serializer.
    pub fn default_serializer(&self) -> Result<Arc<dyn EventSerializer>> {
        self.get(SerializationFormat::Json)
    }

    /// Serialize an event to bytes, auto-detecting the best format.
    ///
    /// Default behavior: use JSON.
    pub fn serialize(&self, envelope: &JsonEnvelope, format: SerializationFormat) -> Result<Vec<u8>> {
        let serializer = self.get(format)?;
        serializer.to_bytes(envelope)
    }

    /// Deserialize an event from bytes with explicit format.
    pub fn deserialize(&self, bytes: &[u8], format: SerializationFormat) -> Result<JsonEnvelope> {
        let serializer = self.get(format)?;
        serializer.from_bytes(bytes)
    }

    /// Attempt to auto-detect the format and deserialize.
    ///
    /// Tries Binary first (checks header), falls back to JSON.
    pub fn deserialize_auto(&self, bytes: &[u8]) -> Result<JsonEnvelope> {
        // Try binary format first
        if bytes.len() >= 8 && bytes.starts_with(b"BINARY\0\0") {
            if let Ok(envelope) = self.deserialize(bytes, SerializationFormat::Binary) {
                return Ok(envelope);
            }
        }

        // Fall back to JSON
        self.deserialize(bytes, SerializationFormat::Json)
    }
}

impl Default for SerializerRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_envelope() -> JsonEnvelope {
        EventEnvelope::new(
            json!({
                "user_id": "user-123",
                "action": "login",
                "timestamp": "2024-01-01T00:00:00Z"
            }),
            "system",
        )
    }

    #[test]
    fn format_identifier_roundtrip() {
        let json_fmt = SerializationFormat::Json;
        let id = json_fmt.identifier();
        assert_eq!(id, "json");
        assert_eq!(SerializationFormat::from_identifier(id).unwrap(), json_fmt);

        let bin_fmt = SerializationFormat::Binary;
        let id = bin_fmt.identifier();
        assert_eq!(id, "binary");
        assert_eq!(SerializationFormat::from_identifier(id).unwrap(), bin_fmt);
    }

    #[test]
    fn json_serializer_roundtrip() {
        let serializer = JsonEventSerializer;
        let envelope = create_test_envelope();

        let bytes = serializer.to_bytes(&envelope).unwrap();
        assert!(!bytes.is_empty());

        let restored = serializer.from_bytes(&bytes).unwrap();
        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn binary_serializer_roundtrip() {
        let serializer = BinaryEventSerializer;
        let envelope = create_test_envelope();

        let bytes = serializer.to_bytes(&envelope).unwrap();
        assert!(bytes.starts_with(b"BINARY\0\0"));

        let restored = serializer.from_bytes(&bytes).unwrap();
        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn registry_default_serializer() {
        let registry = SerializerRegistry::new();
        let serializer = registry.default_serializer().unwrap();
        assert_eq!(serializer.format(), SerializationFormat::Json);
    }

    #[test]
    fn registry_serialize_json() {
        let registry = SerializerRegistry::new();
        let envelope = create_test_envelope();

        let bytes = registry.serialize(&envelope, SerializationFormat::Json).unwrap();
        let restored = registry.deserialize(&bytes, SerializationFormat::Json).unwrap();

        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn registry_serialize_binary() {
        let registry = SerializerRegistry::new();
        let envelope = create_test_envelope();

        let bytes = registry.serialize(&envelope, SerializationFormat::Binary).unwrap();
        let restored = registry.deserialize(&bytes, SerializationFormat::Binary).unwrap();

        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn registry_auto_detect_binary() {
        let registry = SerializerRegistry::new();
        let envelope = create_test_envelope();

        let bytes = registry.serialize(&envelope, SerializationFormat::Binary).unwrap();
        let restored = registry.deserialize_auto(&bytes).unwrap();

        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn registry_auto_detect_json() {
        let registry = SerializerRegistry::new();
        let envelope = create_test_envelope();

        let bytes = registry.serialize(&envelope, SerializationFormat::Json).unwrap();
        let restored = registry.deserialize_auto(&bytes).unwrap();

        assert_eq!(restored.actor, envelope.actor);
        assert_eq!(restored.payload, envelope.payload);
    }

    #[test]
    fn invalid_format_identifier() {
        let result = SerializationFormat::from_identifier("unknown");
        assert!(result.is_err());
    }

    #[test]
    fn custom_serializer_registration() {
        let mut registry = SerializerRegistry::new();
        let json_serializer = Arc::new(JsonEventSerializer);
        registry.register(json_serializer);

        let retrieved = registry.get(SerializationFormat::Json).unwrap();
        assert_eq!(retrieved.format(), SerializationFormat::Json);
    }

    #[test]
    fn all_serializers_available() {
        let registry = SerializerRegistry::new();
        assert!(registry.get(SerializationFormat::Json).is_ok());
        assert!(registry.get(SerializationFormat::Binary).is_ok());
    }
}
