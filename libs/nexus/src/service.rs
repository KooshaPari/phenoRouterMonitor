//! Service types for Nexus

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// An endpoint (host:port) for a service
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Endpoint {
    /// The address (host:port)
    pub addr: String,
}

impl Endpoint {
    /// Create a new endpoint
    pub fn new(addr: impl Into<String>) -> Self {
        Self { addr: addr.into() }
    }
}

/// A registered service in the registry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service name
    pub name: String,
    /// Service endpoint
    pub endpoint: Endpoint,
    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    /// Optional tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Service {
    /// Create a new service
    pub fn new(name: impl Into<String>, endpoint: Endpoint) -> Self {
        Self {
            name: name.into(),
            endpoint,
            metadata: HashMap::new(),
            tags: Vec::new(),
        }
    }

    /// Add a tag to the service
    pub fn with_tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }

    /// Add metadata to the service
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_endpoint_creation() {
        let endpoint = Endpoint::new("localhost:8080");
        assert_eq!(endpoint.addr, "localhost:8080");
    }

    #[test]
    fn test_service_creation() {
        let service = Service::new("api", Endpoint::new("localhost:8080"));
        assert_eq!(service.name, "api");
        assert_eq!(service.endpoint.addr, "localhost:8080");
    }
}
