//! Service registry module

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::{Service, NexusError};

/// Service registry for managing service registrations
///
/// # Example
///
/// ```ignore
/// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
/// use nexus::{Registry, Service, Endpoint};
///
/// let registry = Registry::new();
/// registry.register(Service::new("api", Endpoint::new("localhost:8080"))).await?;
///
/// let services = registry.discover("api").await?;
/// assert_eq!(services.len(), 1);
/// # Ok(())
/// # }
/// ```
pub struct Registry {
    services: Arc<RwLock<HashMap<String, Vec<Service>>>>,
}

impl Registry {
    /// Create a new registry instance
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service
    pub async fn register(&self, service: Service) -> Result<(), NexusError> {
        let mut services = self.services.write().await;
        let entries = services.entry(service.name.clone()).or_insert_with(Vec::new);
        entries.push(service);
        Ok(())
    }

    /// Deregister a service by name and endpoint
    pub async fn deregister(&self, name: &str, endpoint: &str) -> Result<(), NexusError> {
        let mut services = self.services.write().await;
        if let Some(entries) = services.get_mut(name) {
            entries.retain(|s| s.endpoint.addr != endpoint);
        }
        Ok(())
    }

    /// Discover services by name
    pub async fn discover(&self, name: &str) -> Result<Vec<Service>, NexusError> {
        let services = self.services.read().await;
        Ok(services.get(name).cloned().unwrap_or_default())
    }
}

impl Default for Registry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_registry_register_and_discover() {
        let registry = Registry::new();
        let service = Service::new("test-svc", crate::Endpoint::new("localhost:8080"));
        registry.register(service).await.unwrap();
        let discovered = registry.discover("test-svc").await.unwrap();
        assert_eq!(discovered.len(), 1);
    }

    #[tokio::test]
    async fn test_registry_deregister() {
        let registry = Registry::new();
        registry.register(Service::new("test-svc", crate::Endpoint::new("localhost:8080"))).await.unwrap();
        registry.deregister("test-svc", "localhost:8080").await.unwrap();
        let discovered = registry.discover("test-svc").await.unwrap();
        assert!(discovered.is_empty());
    }

    #[tokio::test]
    async fn test_registry_discover_nonexistent() {
        let registry = Registry::new();
        let discovered = registry.discover("nonexistent").await.unwrap();
        assert!(discovered.is_empty());
    }
}
