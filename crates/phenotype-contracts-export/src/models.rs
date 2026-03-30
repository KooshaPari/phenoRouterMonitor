//! Data models for contract metadata and versioning.

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

/// Contract metadata and versioning information.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    /// Semantic version of the contract specification
    pub contract_version: String,

    /// Format of the schema (openapi/3.0.0, protobuf, json-schema)
    pub schema_format: String,

    /// Timestamp when contracts were generated
    pub generated_at: DateTime<Utc>,

    /// Number of ports exported
    pub ports_count: usize,

    /// Number of models exported
    pub models_count: usize,

    /// List of supported formats
    pub formats: Vec<String>,

    /// List of inbound ports
    pub inbound_ports: Vec<String>,

    /// List of outbound ports
    pub outbound_ports: Vec<String>,
}

impl ContractMetadata {
    /// Creates a new ContractMetadata instance with defaults.
    pub fn new() -> Self {
        Self {
            contract_version: crate::CONTRACT_VERSION.to_string(),
            schema_format: format!("openapi/{}", crate::OPENAPI_VERSION),
            generated_at: Utc::now(),
            ports_count: 8, // 4 inbound + 5 outbound
            models_count: 1, // DomainEvent
            formats: vec![
                "openapi/3.0.0".to_string(),
                "protobuf/3".to_string(),
                "json-schema/draft-07".to_string(),
            ],
            inbound_ports: vec![
                "UseCase".to_string(),
                "CommandHandler".to_string(),
                "QueryHandler".to_string(),
                "EventHandler".to_string(),
            ],
            outbound_ports: vec![
                "Repository".to_string(),
                "CachePort".to_string(),
                "EventBus".to_string(),
                "SecretManager".to_string(),
                "ConfigLoader".to_string(),
            ],
        }
    }
}

impl Default for ContractMetadata {
    fn default() -> Self {
        Self::new()
    }
}

/// Contract artifact representing a single exported contract in a specific format.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractArtifact {
    /// Name/identifier of the artifact
    pub name: String,

    /// File path where the artifact should be stored
    pub path: String,

    /// MIME type of the artifact content
    pub content_type: String,

    /// The actual contract content
    pub content: String,

    /// Version of this artifact
    pub version: String,

    /// Checksum for integrity verification
    pub checksum: String,
}

impl ContractArtifact {
    /// Creates a new ContractArtifact.
    pub fn new(name: String, path: String, content_type: String, content: String) -> Self {
        let checksum = Self::compute_checksum(&content);
        Self {
            name,
            path,
            content_type,
            content,
            version: crate::CONTRACT_VERSION.to_string(),
            checksum,
        }
    }

    fn compute_checksum(content: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }
}

/// Collection of all generated contract artifacts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractBundle {
    /// Metadata about the contracts
    pub metadata: ContractMetadata,

    /// All generated artifacts
    pub artifacts: Vec<ContractArtifact>,
}

impl ContractBundle {
    /// Creates a new ContractBundle.
    pub fn new() -> Self {
        Self {
            metadata: ContractMetadata::new(),
            artifacts: Vec::new(),
        }
    }

    /// Adds an artifact to the bundle.
    pub fn add_artifact(&mut self, artifact: ContractArtifact) {
        self.artifacts.push(artifact);
    }

    /// Gets the number of artifacts in the bundle.
    pub fn artifact_count(&self) -> usize {
        self.artifacts.len()
    }
}

impl Default for ContractBundle {
    fn default() -> Self {
        Self::new()
    }
}
