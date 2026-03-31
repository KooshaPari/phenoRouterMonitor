//! API key domain type — authentication for dashboard and API.
//!
//! Traceability: FR-028, FR-029 / WP01-T006

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::feature::hex_bytes;

/// An API key for authenticating requests to the AgilePlus API and dashboard.
///
/// The plaintext key is never stored — only its SHA-256 hash.
/// The plaintext is shown to the user once on generation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKey {
    pub id: i64,
    #[serde(with = "hex_bytes")]
    pub key_hash: [u8; 32],
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked: bool,
}

impl ApiKey {
    pub fn new(key_hash: [u8; 32], name: impl Into<String>) -> Self {
        Self {
            id: 0,
            key_hash,
            name: name.into(),
            created_at: Utc::now(),
            last_used_at: None,
            revoked: false,
        }
    }

    /// Check if this key is valid (not revoked).
    pub fn is_valid(&self) -> bool {
        !self.revoked
    }

    /// Mark this key as used (update last_used_at).
    pub fn touch(&mut self) {
        self.last_used_at = Some(Utc::now());
    }

    /// Revoke this key (soft-delete).
    pub fn revoke(&mut self) {
        self.revoked = true;
    }
}

impl std::fmt::Display for ApiKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ApiKey({}, name={}, revoked={})",
            self.id, self.name, self.revoked
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_api_key() {
        let k = ApiKey::new([0xab; 32], "default");
        assert_eq!(k.name, "default");
        assert!(k.is_valid());
        assert!(k.last_used_at.is_none());
    }

    #[test]
    fn api_key_lifecycle() {
        let mut k = ApiKey::new([0xff; 32], "cli");
        assert!(k.is_valid());

        k.touch();
        assert!(k.last_used_at.is_some());

        k.revoke();
        assert!(!k.is_valid());
    }

    #[test]
    fn api_key_serde_roundtrip() {
        let k = ApiKey::new([0xcd; 32], "test");
        let json = serde_json::to_string(&k).unwrap();
        let k2: ApiKey = serde_json::from_str(&json).unwrap();
        assert_eq!(k2.key_hash, [0xcd; 32]);
        assert_eq!(k2.name, "test");
    }

    #[test]
    fn api_key_display() {
        let k = ApiKey::new([0; 32], "my-key");
        assert!(k.to_string().contains("my-key"));
    }
}
