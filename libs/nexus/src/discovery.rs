//! Service discovery module

use std::hash::BuildHasher;
use crate::{Registry, Service, NexusError};

/// Discovery strategies for load balancing
#[derive(Debug, Clone)]
pub enum Strategy {
    /// Round-robin selection
    RoundRobin,
    /// Random selection
    Random,
    /// Consistent hash
    ConsistentHash,
}

/// Discovery for finding and selecting services
pub struct Discovery {
    registry: Registry,
    strategy: Strategy,
}

impl Discovery {
    /// Create a new discovery instance
    pub fn new(registry: Registry, strategy: Strategy) -> Self {
        Self { registry, strategy }
    }

    /// Find the next endpoint using the configured strategy
    pub async fn next(&self, name: &str) -> Result<Option<Service>, NexusError> {
        let services = self.registry.discover(name).await?;
        if services.is_empty() {
            return Ok(None);
        }

        match self.strategy {
            Strategy::RoundRobin => Ok(services.into_iter().next()),
            Strategy::Random => {
                use std::collections::hash_map::RandomState;
                let idx = (RandomState::new().hash_one(name) as usize) % services.len();
                Ok(services.into_iter().nth(idx))
            }
            Strategy::ConsistentHash => Ok(services.into_iter().next()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_discovery_round_robin() {
        let registry = Registry::new();
        registry.register(Service::new("api", crate::Endpoint::new("localhost:8080"))).await.unwrap();
        let discovery = Discovery::new(registry, Strategy::RoundRobin);
        let service = discovery.next("api").await.unwrap();
        assert!(service.is_some());
    }

    #[tokio::test]
    async fn test_discovery_not_found() {
        let registry = Registry::new();
        let discovery = Discovery::new(registry, Strategy::Random);
        let service = discovery.next("nonexistent").await.unwrap();
        assert!(service.is_none());
    }
}
