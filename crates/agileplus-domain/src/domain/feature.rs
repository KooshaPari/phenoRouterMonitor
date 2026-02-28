use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::state_machine::{FeatureState, TransitionResult};
use crate::error::DomainError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub state: FeatureState,
    #[serde(with = "hex_bytes")]
    pub spec_hash: [u8; 32],
    pub target_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Feature {
    pub fn new(
        slug: &str,
        friendly_name: &str,
        spec_hash: [u8; 32],
        target_branch: Option<&str>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash,
            target_branch: target_branch.unwrap_or("main").to_string(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn transition(&mut self, target: FeatureState) -> Result<TransitionResult, DomainError> {
        let result = self.state.transition(target)?;
        self.state = target;
        self.updated_at = Utc::now();
        Ok(result)
    }

    pub fn slug_from_name(name: &str) -> String {
        name.chars()
            .map(|c| {
                if c.is_alphanumeric() {
                    c.to_ascii_lowercase()
                } else {
                    '-'
                }
            })
            .collect::<String>()
            .split('-')
            .filter(|s| !s.is_empty())
            .collect::<Vec<_>>()
            .join("-")
    }
}

pub mod hex_bytes {
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(bytes: &[u8; 32], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex_string: String = bytes.iter().map(|b| format!("{b:02x}")).collect();
        serializer.serialize_str(&hex_string)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 32], D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.len() != 64 {
            return Err(serde::de::Error::custom(
                "expected 64 hex characters for [u8; 32]",
            ));
        }
        let mut bytes = [0u8; 32];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte =
                u8::from_str_radix(&s[i * 2..i * 2 + 2], 16).map_err(serde::de::Error::custom)?;
        }
        Ok(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_feature_defaults() {
        let f = Feature::new("my-feat", "My Feature", [0u8; 32], None);
        assert_eq!(f.id, 0);
        assert_eq!(f.state, FeatureState::Created);
        assert_eq!(f.target_branch, "main");
    }

    #[test]
    fn feature_transition_updates_state() {
        let mut f = Feature::new("f", "F", [1u8; 32], Some("dev"));
        f.transition(FeatureState::Specified).unwrap();
        assert_eq!(f.state, FeatureState::Specified);
    }

    #[test]
    fn feature_transition_backward_fails() {
        let mut f = Feature::new("f", "F", [0u8; 32], None);
        f.transition(FeatureState::Specified).unwrap();
        assert!(f.transition(FeatureState::Created).is_err());
    }

    #[test]
    fn slug_from_name() {
        assert_eq!(Feature::slug_from_name("Hello World"), "hello-world");
        assert_eq!(Feature::slug_from_name("  Foo  Bar  "), "foo-bar");
        assert_eq!(Feature::slug_from_name("a--b"), "a-b");
    }

    #[test]
    fn hex_bytes_serde_roundtrip() {
        let f = Feature::new("s", "S", [0xab; 32], None);
        let json = serde_json::to_string(&f).unwrap();
        assert!(json.contains("abababab"));
        let f2: Feature = serde_json::from_str(&json).unwrap();
        assert_eq!(f2.spec_hash, [0xab; 32]);
    }
}
