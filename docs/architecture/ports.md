---
audience: [developers, sdk]
---

# Port Traits

Port traits define the **contract** between domain logic and external systems. They are defined in `crates/agileplus-domain/src/ports/` and must be implemented by adapters.

This design allows:
- **Testability** — mock implementations for unit tests
- **Flexibility** — swap implementations without changing domain code
- **Decoupling** — domain doesn't depend on specific libraries (git2, sqlx, etc.)

## StoragePort

Abstracts all **persistent storage** operations. The primary implementation is SQLite, but adapters for PostgreSQL, DynamoDB, etc. can be added without changing domain code.

**Defined in**: `crates/agileplus-domain/src/ports/storage.rs`

```rust
pub trait StoragePort: Send + Sync {
    // --- Feature CRUD ---

    /// Create a new feature, returning its assigned ID.
    fn create_feature(
        &self,
        feature: &Feature,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a feature by its unique slug.
    fn get_feature_by_slug(
        &self,
        slug: &str,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send;

    /// Look up a feature by its primary key.
    fn get_feature_by_id(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<Feature>, DomainError>> + Send;

    /// Update only the state field of an existing feature.
    fn update_feature_state(
        &self,
        id: i64,
        state: FeatureState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// List all features currently in the given state.
    fn list_features_by_state(
        &self,
        state: FeatureState,
    ) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    /// List every feature in the system.
    fn list_all_features(&self) -> impl Future<Output = Result<Vec<Feature>, DomainError>> + Send;

    // --- Work Package CRUD ---

    /// Create a new work package, returning its assigned ID.
    fn create_work_package(
        &self,
        wp: &WorkPackage,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a work package by primary key.
    fn get_work_package(
        &self,
        id: i64,
    ) -> impl Future<Output = Result<Option<WorkPackage>, DomainError>> + Send;

    /// Update only the state field of a work package.
    fn update_wp_state(
        &self,
        id: i64,
        state: WpState,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// List all work packages belonging to a feature.
    fn list_wps_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    /// Record a dependency between two work packages.
    fn add_wp_dependency(
        &self,
        dep: &WpDependency,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Get all dependencies for a given work package.
    fn get_wp_dependencies(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<WpDependency>, DomainError>> + Send;

    /// Get work packages whose dependencies are all in `Done` state.
    fn get_ready_wps(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<WorkPackage>, DomainError>> + Send;

    // --- Audit CRUD ---

    /// Append an audit entry to the immutable log, returning its ID.
    fn append_audit_entry(
        &self,
        entry: &AuditEntry,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Retrieve the full audit trail for a feature, ordered by timestamp.
    fn get_audit_trail(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<AuditEntry>, DomainError>> + Send;

    /// Get the most recent audit entry for a feature.
    fn get_latest_audit_entry(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<AuditEntry>, DomainError>> + Send;

    // --- Evidence + Policy + Metric CRUD ---

    /// Store a new piece of evidence, returning its ID.
    fn create_evidence(
        &self,
        evidence: &Evidence,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Get all evidence associated with a work package.
    fn get_evidence_by_wp(
        &self,
        wp_id: i64,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    /// Get all evidence satisfying a given functional requirement.
    fn get_evidence_by_fr(
        &self,
        fr_id: &str,
    ) -> impl Future<Output = Result<Vec<Evidence>, DomainError>> + Send;

    /// Create a new policy rule, returning its ID.
    fn create_policy_rule(
        &self,
        rule: &PolicyRule,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// List all currently active policy rules.
    fn list_active_policies(
        &self,
    ) -> impl Future<Output = Result<Vec<PolicyRule>, DomainError>> + Send;

    /// Record a command-execution metric, returning its ID.
    fn record_metric(
        &self,
        metric: &Metric,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Get all metrics associated with a feature.
    fn get_metrics_by_feature(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Vec<Metric>, DomainError>> + Send;

    // --- Governance ---

    /// Store a governance contract, returning its ID.
    fn create_governance_contract(
        &self,
        contract: &GovernanceContract,
    ) -> impl Future<Output = Result<i64, DomainError>> + Send;

    /// Look up a specific version of a governance contract for a feature.
    fn get_governance_contract(
        &self,
        feature_id: i64,
        version: i32,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;

    /// Get the latest governance contract for a feature.
    fn get_latest_governance_contract(
        &self,
        feature_id: i64,
    ) -> impl Future<Output = Result<Option<GovernanceContract>, DomainError>> + Send;
}
```

**Implementation**: `crates/agileplus-sqlite/src/adapter.rs` (SQLite)

**Characteristics**:
- Async-first (returns futures)
- All-or-nothing transactions at the domain level
- Append-only for audit entries (no updates/deletes)
- Query by state (list features in Shipped state, etc.)

## VcsPort

Abstracts **version control** operations. The primary implementation uses `git2-rs` for native git operations, but adapters for Mercurial, Pijul, etc. can be added.

**Defined in**: `crates/agileplus-domain/src/ports/vcs.rs`

```rust
pub trait VcsPort: Send + Sync {
    // --- Worktree operations ---

    /// Create a worktree for a feature work package, returning its absolute path.
    fn create_worktree(
        &self,
        feature_slug: &str,
        wp_id: &str,
    ) -> impl Future<Output = Result<PathBuf, DomainError>> + Send;

    /// List all active worktrees.
    fn list_worktrees(
        &self,
    ) -> impl Future<Output = Result<Vec<WorktreeInfo>, DomainError>> + Send;

    /// Remove a worktree at the given path.
    fn cleanup_worktree(
        &self,
        worktree_path: &Path,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    // --- Branch operations ---

    /// Create a new branch from a base ref.
    fn create_branch(
        &self,
        branch_name: &str,
        base: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Check out an existing branch.
    fn checkout_branch(
        &self,
        branch_name: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Merge source branch into target, returning the merge result.
    fn merge_to_target(
        &self,
        source: &str,
        target: &str,
    ) -> impl Future<Output = Result<MergeResult, DomainError>> + Send;

    /// Detect merge conflicts between two branches without performing the merge.
    fn detect_conflicts(
        &self,
        source: &str,
        target: &str,
    ) -> impl Future<Output = Result<Vec<ConflictInfo>, DomainError>> + Send;

    // --- Artifact operations ---

    /// Read a text artifact relative to the feature directory.
    fn read_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl Future<Output = Result<String, DomainError>> + Send;

    /// Write a text artifact relative to the feature directory.
    fn write_artifact(
        &self,
        feature_slug: &str,
        relative_path: &str,
        content: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Check whether an artifact exists.
    fn artifact_exists(
        &self,
        feature_slug: &str,
        relative_path: &str,
    ) -> impl Future<Output = Result<bool, DomainError>> + Send;

    // --- History scanning ---

    /// Scan and collect all feature artifacts from the repository.
    fn scan_feature_artifacts(
        &self,
        feature_slug: &str,
    ) -> impl Future<Output = Result<FeatureArtifacts, DomainError>> + Send;
}
```

**Related types**:

```rust
pub struct WorktreeInfo {
    pub path: PathBuf,
    pub branch: String,
    pub feature_slug: String,
    pub wp_id: String,
}

pub struct MergeResult {
    pub success: bool,
    pub conflicts: Vec<ConflictInfo>,
    pub merged_commit: Option<String>,
}

pub struct ConflictInfo {
    pub path: String,
    pub ours: Option<String>,   // Our version
    pub theirs: Option<String>, // Their version
}

pub struct FeatureArtifacts {
    pub meta_json: Option<String>,  // Metadata JSON
    pub audit_chain: Option<String>, // Audit chain JSON
    pub evidence_paths: Vec<String>, // Evidence file paths
}
```

**Implementation**: `crates/agileplus-git/src/adapter.rs` (Git)

**Characteristics**:
- Worktrees are isolated environments (no interference between WPs)
- Artifacts are markdown/JSON files in the repo
- Merge detection prevents conflicts before they occur
- History scanning collects all feature-related data

## AgentPort

Abstracts **AI agent dispatch** and communication. Allows different agent backends (Claude Code, Codex, etc.) to be swapped without changing domain code.

**Defined in**: `crates/agileplus-domain/src/ports/agent.rs`

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AgentKind {
    ClaudeCode,
    Codex,
}

pub struct AgentConfig {
    pub kind: AgentKind,
    pub max_review_cycles: u32,  // How many fix rounds?
    pub timeout_secs: u64,       // Timeout in seconds
    pub extra_args: Vec<String>, // Agent-specific flags
}

pub struct AgentTask {
    pub wp_id: String,                      // "WP01"
    pub feature_slug: String,               // "user-authentication"
    pub prompt_path: PathBuf,               // Path to generated prompt
    pub worktree_path: PathBuf,             // Path to git worktree
    pub context_files: Vec<PathBuf>,        // Additional context
}

pub struct AgentResult {
    pub success: bool,
    pub pr_url: Option<String>,             // GitHub PR if created
    pub commits: Vec<String>,               // Commit hashes
    pub stdout: String,                     // Full output
    pub stderr: String,                     // Error output
    pub exit_code: i32,                     // Exit code
}

pub enum AgentStatus {
    Pending,
    Running { pid: u32 },
    WaitingForReview { pr_url: String },
    Completed { result: AgentResult },
    Failed { error: String },
}

pub trait AgentPort: Send + Sync {
    /// Spawn an agent and wait for completion.
    fn dispatch(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> impl Future<Output = Result<AgentResult, DomainError>> + Send;

    /// Spawn an agent without blocking; returns a job ID for later polling.
    fn dispatch_async(
        &self,
        task: AgentTask,
        config: &AgentConfig,
    ) -> impl Future<Output = Result<String, DomainError>> + Send;

    /// Query the current status of a previously dispatched job.
    fn query_status(
        &self,
        job_id: &str,
    ) -> impl Future<Output = Result<AgentStatus, DomainError>> + Send;

    /// Cancel a running or pending job.
    fn cancel(&self, job_id: &str) -> impl Future<Output = Result<(), DomainError>> + Send;

    /// Send a follow-up instruction to a running agent.
    fn send_instruction(
        &self,
        job_id: &str,
        instruction: &str,
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
```

**Implementation**: Dispatcher adapter (Claude Code implementation in MVP; Codex in WP08)

**Characteristics**:
- Both sync and async dispatch supported
- Status polling for long-running tasks
- Structured prompts from domain context
- Result validation against acceptance criteria

## ObservabilityPort

Abstracts **logging, tracing, and metrics**.

**Defined in**: `crates/agileplus-domain/src/ports/observability.rs`

```rust
pub trait ObservabilityPort: Send + Sync {
    fn log(&self, level: LogLevel, message: &str, context: Option<serde_json::Value>);
    fn start_span(&self, name: &str) -> SpanGuard;
    fn record_metric(&self, name: &str, value: f64, labels: Option<serde_json::Value>);
    fn record_error(&self, error: &dyn std::error::Error);
}

pub enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}
```

**Implementation**: Tracing adapter (uses `tracing` and `tracing-subscriber` crates)

## ReviewPort

Abstracts **code review and approval** workflows (GitHub, GitLab, Gerrit, etc.).

**Defined in**: `crates/agileplus-domain/src/ports/review.rs`

```rust
pub trait ReviewPort: Send + Sync {
    fn submit_for_review(
        &self,
        wp_id: i64,
        branch: &str,
        description: &str,
    ) -> impl Future<Output = Result<ReviewResult, DomainError>> + Send;

    fn get_review_status(
        &self,
        review_id: &str,
    ) -> impl Future<Output = Result<ReviewStatus, DomainError>> + Send;

    fn request_approval(
        &self,
        review_id: &str,
        reviewers: &[String],
    ) -> impl Future<Output = Result<(), DomainError>> + Send;
}
```

**Implementation**: GitHub API adapter (planned WP12)

## Implementation Guidelines

When implementing a port:

1. **Implement the trait**: Create a struct and implement all trait methods
2. **Handle errors**: Return `Result<T, DomainError>` with semantic errors
3. **Be async-first**: Use `impl Future<Output = ...> + Send`
4. **Log operations**: Use ObservabilityPort for debugging
5. **Test with mocks**: Provide a mock implementation for testing

Example mock implementation:

```rust
pub struct MockStoragePort {
    features: Arc<Mutex<HashMap<i64, Feature>>>,
}

#[async_trait::async_trait]
impl StoragePort for MockStoragePort {
    async fn create_feature(&self, feature: &Feature) -> Result<i64> {
        let mut f = self.features.lock().unwrap();
        let id = (f.len() + 1) as i64;
        f.insert(id, feature.clone());
        Ok(id)
    }

    async fn get_feature_by_id(&self, id: i64) -> Result<Option<Feature>> {
        let f = self.features.lock().unwrap();
        Ok(f.get(&id).cloned())
    }

    // ... implement other methods
}
```

## Related Pages

- [Architecture Overview](overview.md) — Crate structure and port relationships
- [Domain Model](domain-model.md) — Entity types used by ports
- [Agent Dispatch](../concepts/agent-dispatch.md) — AgentPort usage details
- [Governance & Audit](../concepts/governance.md) — StoragePort audit operations
