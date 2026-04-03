//! Resource management utilities

use std::collections::HashMap;

/// Resource handle
pub struct ResourceHandle {
    pub id: String,
    pub path: String,
}

/// Resource manager
pub struct ResourceManager {
    resources: HashMap<String, ResourceHandle>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            resources: HashMap::new(),
        }
    }

    /// Register a resource
    pub fn register(&mut self, id: impl Into<String>, path: impl Into<String>) {
        let id = id.into();
        let path = path.into();
        self.resources.insert(
            id.clone(),
            ResourceHandle { id, path },
        );
    }

    /// Get a resource by ID
    pub fn get(&self, id: &str) -> Option<&ResourceHandle> {
        self.resources.get(id)
    }
}

impl Default for ResourceManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_resource() {
        let mut manager = ResourceManager::new();
        manager.register("test", "/path/to/resource");
        assert!(manager.get("test").is_some());
    }
}
