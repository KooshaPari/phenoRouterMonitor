---
audience: [sdk, developers]
---

# Storage Port API

The `StoragePort` trait defines how AgilePlus persists specifications, plans, tasks, and metadata.

## Trait Definition

```rust
pub trait StoragePort: Send + Sync {
    /// Read a feature specification
    fn read_spec(&self, feature: &FeatureId) -> Result<Spec>;

    /// Write or update a feature specification
    fn write_spec(&self, feature: &FeatureId, spec: &Spec) -> Result<()>;

    /// List all features in the project
    fn list_features(&self) -> Result<Vec<FeatureSummary>>;

    /// Read the implementation plan for a feature
    fn read_plan(&self, feature: &FeatureId) -> Result<Plan>;

    /// Write or update an implementation plan
    fn write_plan(&self, feature: &FeatureId, plan: &Plan) -> Result<()>;

    /// Read work package tasks
    fn read_tasks(&self, feature: &FeatureId) -> Result<Vec<WorkPackage>>;

    /// Write work package tasks
    fn write_tasks(&self, feature: &FeatureId, tasks: &[WorkPackage]) -> Result<()>;

    /// Read project metadata
    fn read_meta(&self, feature: &FeatureId) -> Result<FeatureMeta>;

    /// Write project metadata
    fn write_meta(&self, feature: &FeatureId, meta: &FeatureMeta) -> Result<()>;
}
```

## Built-in Implementations

### FilesystemStorage

The default implementation stores everything as files:

```
kitty-specs/
└── 001-feature-name/
    ├── meta.json
    ├── spec.md
    ├── plan.md
    └── tasks/
        ├── WP01.md
        └── WP02.md
```

### Key Types

```rust
pub struct FeatureId(pub String);  // e.g., "001-user-login"

pub struct FeatureSummary {
    pub id: FeatureId,
    pub title: String,
    pub state: FeatureState,
    pub created_at: DateTime<Utc>,
}

pub struct FeatureMeta {
    pub feature_number: String,
    pub slug: String,
    pub friendly_name: String,
    pub mission: String,
    pub target_branch: String,
    pub vcs: String,
}
```

## Custom Implementations

To add a custom storage backend (e.g., database, S3):

1. Implement the `StoragePort` trait
2. Register it in the dependency container
3. Configure it in `.kittify/config.toml`

See [Extending AgilePlus](/developers/extending) for details.
