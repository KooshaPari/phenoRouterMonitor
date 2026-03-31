//! Builder patterns for constructing test fixtures.
//!
//! Provides fluent builders for creating features, work packages, and related
//! domain objects for testing. All builders produce valid, deterministic objects.

use agileplus_domain::domain::feature::Feature;
use agileplus_domain::domain::state_machine::FeatureState;
use agileplus_domain::domain::work_package::{WorkPackage, WpState};
use chrono::Utc;

/// Builder for constructing Feature test objects.
#[derive(Clone)]
pub struct FeatureBuilder {
    id: i64,
    slug: String,
    friendly_name: String,
    state: FeatureState,
    spec_hash: [u8; 32],
    target_branch: String,
    labels: Vec<String>,
    project_id: Option<i64>,
}

impl Default for FeatureBuilder {
    fn default() -> Self {
        Self::new("test-feature", "Test Feature")
    }
}

impl FeatureBuilder {
    /// Create a new feature builder with slug and friendly_name.
    pub fn new(slug: &str, friendly_name: &str) -> Self {
        Self {
            id: 1,
            slug: slug.to_string(),
            friendly_name: friendly_name.to_string(),
            state: FeatureState::Created,
            spec_hash: [0u8; 32],
            target_branch: "main".to_string(),
            labels: Vec::new(),
            project_id: None,
        }
    }

    /// Set the feature ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the feature state.
    pub fn state(mut self, state: FeatureState) -> Self {
        self.state = state;
        self
    }

    /// Add a label to the feature.
    pub fn with_label(mut self, label: &str) -> Self {
        self.labels.push(label.to_string());
        self
    }

    /// Set multiple labels.
    pub fn with_labels(mut self, labels: Vec<String>) -> Self {
        self.labels = labels;
        self
    }

    /// Set the project ID.
    pub fn project_id(mut self, project_id: i64) -> Self {
        self.project_id = Some(project_id);
        self
    }

    /// Set the spec hash.
    pub fn spec_hash(mut self, hash: [u8; 32]) -> Self {
        self.spec_hash = hash;
        self
    }

    /// Build the Feature.
    pub fn build(self) -> Feature {
        Feature {
            id: self.id,
            slug: self.slug,
            friendly_name: self.friendly_name,
            state: self.state,
            spec_hash: self.spec_hash,
            target_branch: self.target_branch,
            plane_issue_id: None,
            plane_state_id: None,
            labels: self.labels,
            module_id: None,
            project_id: self.project_id,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            created_at_commit: None,
            last_modified_commit: None,
        }
    }
}

/// Builder for constructing WorkPackage test objects.
#[derive(Clone)]
pub struct WorkPackageBuilder {
    id: i64,
    feature_id: i64,
    title: String,
    sequence: i32,
    summary: String,
    state: WpState,
    file_scope: Vec<String>,
}

impl WorkPackageBuilder {
    /// Create a new work package builder.
    pub fn new(feature_id: i64, title: &str, sequence: i32) -> Self {
        Self {
            id: 1,
            feature_id,
            title: title.to_string(),
            sequence,
            summary: String::new(),
            state: WpState::Planned,
            file_scope: Vec::new(),
        }
    }

    /// Set the work package ID.
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }

    /// Set the work package state.
    pub fn state(mut self, state: WpState) -> Self {
        self.state = state;
        self
    }

    /// Set the summary/description.
    pub fn summary(mut self, summary: &str) -> Self {
        self.summary = summary.to_string();
        self
    }

    /// Add a file to the scope.
    pub fn with_file(mut self, file: &str) -> Self {
        self.file_scope.push(file.to_string());
        self
    }

    /// Set multiple files in scope.
    pub fn with_files(mut self, files: Vec<String>) -> Self {
        self.file_scope = files;
        self
    }

    /// Build the WorkPackage.
    pub fn build(self) -> WorkPackage {
        WorkPackage {
            id: self.id,
            feature_id: self.feature_id,
            title: self.title,
            sequence: self.sequence,
            summary: self.summary,
            state: self.state,
            file_scope: self.file_scope,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_builder_creates_valid_feature() {
        let feature = FeatureBuilder::new("test-slug", "Test Name")
            .id(1)
            .state(FeatureState::Specified)
            .with_label("test")
            .build();

        assert_eq!(feature.id, 1);
        assert_eq!(feature.slug, "test-slug");
        assert_eq!(feature.friendly_name, "Test Name");
        assert_eq!(feature.state, FeatureState::Specified);
        assert_eq!(feature.labels, vec!["test"]);
    }

    #[test]
    fn feature_builder_default_values() {
        let feature = FeatureBuilder::default().build();

        assert_eq!(feature.state, FeatureState::Created);
        assert_eq!(feature.target_branch, "main");
        assert!(feature.labels.is_empty());
        assert!(feature.project_id.is_none());
    }

    #[test]
    fn work_package_builder_creates_valid_wp() {
        let wp = WorkPackageBuilder::new(1, "Test WP", 1)
            .id(100)
            .state(WpState::Done)
            .summary("This is a test")
            .with_file("src/lib.rs")
            .build();

        assert_eq!(wp.id, 100);
        assert_eq!(wp.feature_id, 1);
        assert_eq!(wp.title, "Test WP");
        assert_eq!(wp.state, WpState::Done);
        assert_eq!(wp.summary, "This is a test");
        assert_eq!(wp.file_scope, vec!["src/lib.rs"]);
    }

    #[test]
    fn work_package_builder_multiple_files() {
        let files = vec!["src/lib.rs".to_string(), "tests/unit.rs".to_string()];
        let wp = WorkPackageBuilder::new(1, "Multi-file WP", 2)
            .with_files(files.clone())
            .build();

        assert_eq!(wp.file_scope, files);
    }
}
