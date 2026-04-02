//! Validation context utilities

use crate::types::ValidationContext;

impl ValidationContext {
    /// Create a new context with initial data
    pub fn with_data(data: impl IntoIterator<Item = (String, String)>) -> Self {
        let mut ctx = Self::new();
        for (k, v) in data {
            ctx.set(k, v);
        }
        ctx
    }

    /// Check if context has a key
    pub fn has(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Remove a key from context
    pub fn remove(&mut self, key: &str) -> Option<String> {
        self.data.remove(key)
    }

    /// Clear all data from context
    pub fn clear(&mut self) {
        self.data.clear();
    }
}
