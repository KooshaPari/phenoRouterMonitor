//! Step definitions and context for BDD tests

use std::any::Any;
use std::collections::HashMap;

/// Type of step (Given, When, Then, etc.)
pub use crate::parser::StepType;

/// Context passed to step definitions
#[derive(Debug)]
pub struct StepContext {
    data: HashMap<String, Box<dyn Any>>,
}

impl StepContext {
    /// Create a new step context
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Insert a value into the context
    pub fn insert<T: Any + Send + Sync>(&mut self, key: &str, value: T) {
        self.data.insert(key.to_string(), Box::new(value));
    }

    /// Get a value from the context
    pub fn get<T: Any + Clone>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>()).cloned()
    }

    /// Get a reference to a value in the context
    pub fn get_ref<T: Any>(&self, key: &str) -> Option<&T> {
        self.data.get(key).and_then(|v| v.downcast_ref::<T>())
    }

    /// Check if a key exists in the context
    pub fn contains(&self, key: &str) -> bool {
        self.data.contains_key(key)
    }

    /// Remove a value from the context
    pub fn remove<T: Any>(&mut self, key: &str) -> Option<T> {
        self.data
            .remove(key)
            .and_then(|v| v.downcast::<T>().ok().map(|v| *v))
    }

    /// Clear all values from the context
    pub fn clear(&mut self) {
        self.data.clear();
    }
}

impl Default for StepContext {
    fn default() -> Self {
        Self::new()
    }
}

/// Function type for step definitions
type StepFn = Box<dyn Fn(&mut StepContext) + Send + Sync>;

/// A step definition with pattern and handler
pub struct StepDefinition {
    pub step_type: StepType,
    pub pattern: String,
    pub handler: StepFn,
}

impl std::fmt::Debug for StepDefinition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StepDefinition")
            .field("step_type", &self.step_type)
            .field("pattern", &self.pattern)
            .finish_non_exhaustive()
    }
}

/// Registry for step definitions
pub struct StepRegistry {
    steps: Vec<StepDefinition>,
}

impl StepRegistry {
    /// Create a new step registry
    pub fn new() -> Self {
        Self { steps: Vec::new() }
    }

    /// Register a step definition
    pub fn register<F>(&mut self, step_type: StepType, pattern: &str, func: F)
    where
        F: Fn(&mut StepContext) + Send + Sync + 'static,
    {
        self.steps.push(StepDefinition {
            step_type,
            pattern: pattern.to_string(),
            handler: Box::new(func),
        });
    }

    /// Find a matching step definition
    pub fn find_step(&self, step_type: StepType, text: &str) -> Option<&StepDefinition> {
        self.steps.iter().find(|step| {
            step.step_type == step_type
                && (text.contains(&step.pattern) || step.pattern.contains(text))
        })
    }

    /// Get all registered steps
    pub fn all_steps(&self) -> &[StepDefinition] {
        &self.steps
    }

    /// Clear all steps
    pub fn clear(&mut self) {
        self.steps.clear();
    }
}

impl Default for StepRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_insert_and_get() {
        let mut ctx = StepContext::new();
        ctx.insert("name", "Alice".to_string());
        ctx.insert("age", 30u32);

        assert_eq!(ctx.get::<String>("name"), Some("Alice".to_string()));
        assert_eq!(ctx.get::<u32>("age"), Some(30));
    }

    #[test]
    fn test_context_get_ref() {
        let mut ctx = StepContext::new();
        ctx.insert("name", "Bob".to_string());

        assert_eq!(ctx.get_ref::<String>("name"), Some(&"Bob".to_string()));
    }

    #[test]
    fn test_context_contains() {
        let mut ctx = StepContext::new();
        ctx.insert("key", "value");
        assert!(ctx.contains("key"));
        assert!(!ctx.contains("missing"));
    }

    #[test]
    fn test_step_registry() {
        let mut registry = StepRegistry::new();

        registry.register(StepType::Given, "a user", |_ctx| {});
        registry.register(StepType::When, "they login", |_ctx| {});

        assert_eq!(registry.all_steps().len(), 2);

        let step = registry.find_step(StepType::Given, "a user");
        assert!(step.is_some());
    }
}
