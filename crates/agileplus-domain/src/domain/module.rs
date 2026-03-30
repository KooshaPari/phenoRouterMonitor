//! Module domain entity and related types.
//!
//! Traces to: FR-M01, FR-M02, FR-M04, FR-M07

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// A Module groups Features into logical product areas and supports hierarchical organisation.
///
/// Traces to: FR-M01, FR-M02
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Module {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_module_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Module {
    /// Create a new Module with `id = 0` (populated by storage layer on insert).
    pub fn new(friendly_name: &str, parent_module_id: Option<i64>) -> Self {
        let now = Utc::now();
        Self {
            id: 0,
            slug: Self::slug_from_name(friendly_name),
            friendly_name: friendly_name.to_string(),
            description: None,
            parent_module_id,
            created_at: now,
            updated_at: now,
        }
    }

    /// Derive a kebab-case slug from a display name using the same logic as `Feature::slug_from_name`.
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

    /// Update the module's display name, re-derive the slug, and touch `updated_at`.
    pub fn update_name(&mut self, new_name: &str) {
        self.friendly_name = new_name.to_string();
        self.slug = Self::slug_from_name(new_name);
        self.updated_at = Utc::now();
    }
}

/// Many-to-many tagging join between a Module and a Feature.
///
/// Traces to: FR-M04
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleFeatureTag {
    pub module_id: i64,
    pub feature_id: i64,
    pub created_at: DateTime<Utc>,
}

impl ModuleFeatureTag {
    pub fn new(module_id: i64, feature_id: i64) -> Self {
        Self {
            module_id,
            feature_id,
            created_at: Utc::now(),
        }
    }
}

/// View struct carrying a Module together with its associated features and children.
/// Populated by the storage/query layer in WP02.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleWithFeatures {
    pub module: Module,
    /// Features where `Feature.module_id == this module's id` (strict ownership).
    pub owned_features: Vec<crate::domain::feature::Feature>,
    /// Features linked via `module_feature_tags` (many-to-many tagging).
    pub tagged_features: Vec<crate::domain::feature::Feature>,
    /// Direct child modules (non-recursive).
    pub child_modules: Vec<Module>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_module_defaults() {
        let m = Module::new("My Module", None);
        assert_eq!(m.id, 0);
        assert_eq!(m.slug, "my-module");
        assert!(m.parent_module_id.is_none());
        assert!(m.description.is_none());
    }

    #[test]
    fn new_module_with_parent() {
        let m = Module::new("Child", Some(42));
        assert_eq!(m.parent_module_id, Some(42));
    }

    #[test]
    fn slug_derivation() {
        assert_eq!(Module::slug_from_name("OAuth Providers"), "oauth-providers");
        assert_eq!(Module::slug_from_name("Hello World"), "hello-world");
        assert_eq!(Module::slug_from_name("  Foo  Bar  "), "foo-bar");
        assert_eq!(Module::slug_from_name("a--b"), "a-b");
    }

    #[test]
    fn update_name_re_slugs() {
        let mut m = Module::new("Old Name", None);
        assert_eq!(m.slug, "old-name");
        let before = m.updated_at;
        // Ensure at least 1ns difference on fast systems.
        std::thread::sleep(std::time::Duration::from_millis(1));
        m.update_name("New Name");
        assert_eq!(m.friendly_name, "New Name");
        assert_eq!(m.slug, "new-name");
        assert!(m.updated_at >= before);
    }

    #[test]
    fn tag_new_stamps_created_at() {
        let before = Utc::now();
        let tag = ModuleFeatureTag::new(1, 2);
        let after = Utc::now();
        assert_eq!(tag.module_id, 1);
        assert_eq!(tag.feature_id, 2);
        assert!(tag.created_at >= before);
        assert!(tag.created_at <= after);
    }
}
