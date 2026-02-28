# Port Traits

Port traits define the boundary between domain logic and infrastructure.

## StoragePort

```rust
pub trait StoragePort {
    fn save_feature(&self, feature: &Feature) -> Result<()>;
    fn load_feature(&self, id: &str) -> Result<Option<Feature>>;
    fn list_features(&self) -> Result<Vec<Feature>>;
    fn save_audit_entry(&self, entry: &AuditEntry) -> Result<()>;
    fn load_audit_trail(&self, feature_id: &str) -> Result<Vec<AuditEntry>>;
    // ... work package operations
}
```

Implemented by: `SqliteStorageAdapter`

## VcsPort

```rust
pub trait VcsPort {
    fn create_branch(&self, name: &str, base: &str) -> Result<()>;
    fn current_branch(&self) -> Result<String>;
    fn merge_branch(&self, source: &str, target: &str) -> Result<()>;
    fn list_branches(&self, prefix: &str) -> Result<Vec<String>>;
    // ... worktree operations
}
```

Implemented by: `GitVcsAdapter`

## AgentPort

```rust
pub trait AgentPort {
    fn dispatch(&self, task: &AgentTask) -> Result<AgentResult>;
    fn status(&self, task_id: &str) -> Result<AgentStatus>;
}
```

Implemented by: `StubAgentAdapter` (real adapters in future WPs)
