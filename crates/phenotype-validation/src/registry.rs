//! Validator registry for plugin-based discovery.
//!
//! The `ValidatorRegistry` allows registering and retrieving validator factories
//! at runtime, enabling a plugin architecture for custom validators.

use crate::traits::field_validator::FieldValidator;
use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// Factory function type for creating validators.
pub type ValidatorFactory = fn() -> FieldValidator;

/// Global validator registry.
static VALIDATOR_REGISTRY: Lazy<Mutex<HashMap<String, ValidatorFactory>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

/// Manages registration and discovery of validators.
pub struct ValidatorRegistry;

impl ValidatorRegistry {
    /// Register a validator factory by name.
    ///
    /// # Arguments
    /// * `name` - Identifier for the validator (e.g., "email", "strong_password")
    /// * `factory` - Function that creates a fresh validator instance
    ///
    /// # Example
    /// ```ignore
    /// ValidatorRegistry::register("corporate_email", || {
    ///     FieldValidator::new()
    ///         .with_rule(EmailRule::new())
    ///         .with_rule(CustomDomainRule::new())
    /// });
    /// ```
    pub fn register(name: &str, factory: ValidatorFactory) {
        let mut registry = VALIDATOR_REGISTRY.lock().unwrap();
        registry.insert(name.to_string(), factory);
    }

    /// Get a validator by name.
    ///
    /// # Arguments
    /// * `name` - Identifier for the validator
    ///
    /// # Returns
    /// A fresh validator instance, or None if not registered.
    pub fn get(name: &str) -> Option<FieldValidator> {
        let registry = VALIDATOR_REGISTRY.lock().unwrap();
        registry.get(name).map(|factory| factory())
    }

    /// List all registered validator names.
    pub fn list() -> Vec<String> {
        let registry = VALIDATOR_REGISTRY.lock().unwrap();
        let mut names: Vec<_> = registry.keys().cloned().collect();
        names.sort();
        names
    }

    /// Check if a validator is registered.
    pub fn exists(name: &str) -> bool {
        let registry = VALIDATOR_REGISTRY.lock().unwrap();
        registry.contains_key(name)
    }

    /// Clear all registered validators (useful for testing).
    #[cfg(test)]
    pub fn clear() {
        let mut registry = VALIDATOR_REGISTRY.lock().unwrap();
        registry.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traits::rule::RequiredRule;

    #[test]
    fn test_registry_register_and_get() {
        ValidatorRegistry::clear();

        ValidatorRegistry::register("test_validator", || {
            FieldValidator::new().with_rule(RequiredRule::new())
        });

        let validator = ValidatorRegistry::get("test_validator");
        assert!(validator.is_some());
        assert_eq!(validator.unwrap().rule_count(), 1);
    }

    #[test]
    fn test_registry_get_nonexistent() {
        ValidatorRegistry::clear();

        let validator = ValidatorRegistry::get("nonexistent");
        assert!(validator.is_none());
    }

    #[test]
    fn test_registry_list() {
        ValidatorRegistry::clear();

        ValidatorRegistry::register("validator_a", || FieldValidator::new());
        ValidatorRegistry::register("validator_b", || FieldValidator::new());
        ValidatorRegistry::register("validator_c", || FieldValidator::new());

        let list = ValidatorRegistry::list();
        assert_eq!(list.len(), 3);
        assert!(list.contains(&"validator_a".to_string()));
        assert!(list.contains(&"validator_b".to_string()));
        assert!(list.contains(&"validator_c".to_string()));
    }

    #[test]
    fn test_registry_exists() {
        ValidatorRegistry::clear();

        ValidatorRegistry::register("existing", || FieldValidator::new());

        assert!(ValidatorRegistry::exists("existing"));
        assert!(!ValidatorRegistry::exists("nonexistent"));
    }

    #[test]
    fn test_registry_factory_creates_fresh_instances() {
        ValidatorRegistry::clear();

        let call_count = std::sync::Arc::new(std::sync::Mutex::new(0));
        let call_count_clone = call_count.clone();

        ValidatorRegistry::register("counter", move || {
            let mut count = call_count_clone.lock().unwrap();
            *count += 1;
            FieldValidator::new()
        });

        let _ = ValidatorRegistry::get("counter");
        let _ = ValidatorRegistry::get("counter");
        let _ = ValidatorRegistry::get("counter");

        // Factory was called 3 times
        assert_eq!(*call_count.lock().unwrap(), 3);
    }
}
