---
audience: [developers]
---

# Extending AgilePlus

AgilePlus uses a port-based architecture that makes it straightforward to add new integrations.

## Extension Points

| Extension | Trait | Purpose |
|-----------|-------|---------|
| Storage backend | `StoragePort` | Where specs/plans/tasks are persisted |
| VCS integration | `VcsPort` | Git operations, branch management |
| Tracker sync | `SyncPort` | Issue tracker integration |
| Agent harness | `AgentHarness` | AI agent communication |

## Adding a Storage Backend

Implement the `StoragePort` trait:

```rust
pub trait StoragePort {
    fn read_spec(&self, feature: &FeatureId) -> Result<Spec>;
    fn write_spec(&self, feature: &FeatureId, spec: &Spec) -> Result<()>;
    fn list_features(&self) -> Result<Vec<FeatureId>>;
    fn read_plan(&self, feature: &FeatureId) -> Result<Plan>;
    fn write_plan(&self, feature: &FeatureId, plan: &Plan) -> Result<()>;
}
```

Register it in the dependency container:

```rust
container.register::<dyn StoragePort>(MyStorageBackend::new(config));
```

## Adding a VCS Provider

Implement `VcsPort`:

```rust
pub trait VcsPort {
    fn create_branch(&self, name: &str, base: &str) -> Result<()>;
    fn create_worktree(&self, path: &Path, branch: &str) -> Result<()>;
    fn commit(&self, message: &str, files: &[PathBuf]) -> Result<CommitId>;
    fn merge(&self, source: &str, target: &str) -> Result<MergeResult>;
}
```

## Adding a Tracker Integration

Implement `SyncPort`:

```rust
pub trait SyncPort {
    fn push_issues(&self, issues: &[Issue]) -> Result<()>;
    fn pull_issues(&self) -> Result<Vec<Issue>>;
    fn update_status(&self, id: &IssueId, status: Status) -> Result<()>;
}
```

## Plugin Discovery

AgilePlus discovers extensions at startup from:

1. Built-in adapters (filesystem, git, plane, github)
2. Config-specified adapters in `.kittify/config.toml`
3. Crate features (compile-time selection)
