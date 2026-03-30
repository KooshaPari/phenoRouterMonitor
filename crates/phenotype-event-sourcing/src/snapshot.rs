//! Snapshot support.

use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone)]
pub struct SnapshotConfig {
    pub max_events: usize,
    pub max_age_seconds: u64,
}

impl Default for SnapshotConfig {
    fn default() -> Self {
        Self {
            max_events: 100,
            max_age_seconds: 3600,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot<T> {
    pub entity_id: String,
    pub sequence: i64,
    pub payload: T,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config() {
        let config = SnapshotConfig::default();
        assert_eq!(config.max_events, 100);
        assert_eq!(config.max_age_seconds, 3600);
    }
}
