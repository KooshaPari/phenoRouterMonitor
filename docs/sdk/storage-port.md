---
audience: [sdk, developers]
---

# Storage Port API

The `StoragePort` trait defines the persistence abstraction for all domain entities. It provides async CRUD operations for features, work packages, audit entries, evidence, policy rules, and governance contracts. The trait has zero external dependencies and is implemented by `agileplus-sqlite` using SQLite.

## Trait Definition

```rust
pub trait StoragePort: Send + Sync {
    // -- Feature CRUD --

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

    // -- Work Package CRUD --

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

    // -- Audit CRUD --

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

    // -- Evidence + Policy + Metric CRUD --

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

    // -- Governance --

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

## Key Domain Types

All types use strongly-typed IDs and enums to prevent invalid states.

### Feature

```rust
pub struct Feature {
    pub id: i64,
    pub slug: String,
    pub friendly_name: String,
    pub state: FeatureState,
    pub target_branch: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub enum FeatureState {
    SPECIFY,
    PLAN,
    IMPLEMENT,
    REVIEW,
    DONE,
}
```

### WorkPackage

```rust
pub struct WorkPackage {
    pub id: i64,
    pub feature_id: i64,
    pub sequence: u32,           // 1, 2, 3...
    pub title: String,
    pub state: WpState,
    pub agent_id: Option<String>,
    pub pr_url: Option<String>,
    pub file_scope: Vec<String>, // Authorized files
}

pub enum WpState {
    PLANNED,
    DOING,
    FOR_REVIEW,
    DONE,
}

pub struct WpDependency {
    pub wp_id: i64,
    pub depends_on: i64,
}
```

### Audit Trail (Immutable)

```rust
pub struct AuditEntry {
    pub id: i64,
    pub feature_id: i64,
    pub wp_sequence: Option<u32>,
    pub timestamp: DateTime<Utc>,
    pub actor: String,           // "claude-code", "user@example.com", etc.
    pub transition: String,      // "SPECIFY -> PLAN", "WP01: PLANNED -> DOING"
    pub evidence_refs: Vec<String>, // ["pr/42", "commit/abc123"]
    pub prev_hash: Vec<u8>,      // SHA256 of previous entry
    pub hash: Vec<u8>,           // SHA256(entry) — cryptographic chain
}
```

### Governance

```rust
pub struct GovernanceContract {
    pub id: i64,
    pub feature_id: i64,
    pub version: i32,
    pub rules: Vec<PolicyRule>,
    pub created_at: DateTime<Utc>,
}

pub struct PolicyRule {
    pub id: i64,
    pub rule_id: String,         // "FR-REVIEW-001"
    pub description: String,
    pub is_active: bool,
}

pub struct Evidence {
    pub id: i64,
    pub wp_id: i64,
    pub fr_id: String,           // Requirement this satisfies
    pub artifact_url: String,    // PR, commit, test report URL
    pub confidence: f64,         // 0.0 to 1.0
}
```

### Metrics

```rust
pub struct Metric {
    pub id: i64,
    pub feature_id: i64,
    pub wp_id: i64,
    pub metric_type: String,     // "lines_changed", "test_coverage", "review_time"
    pub value: f64,
    pub unit: String,            // "lines", "%", "minutes"
    pub recorded_at: DateTime<Utc>,
}
```

## Built-in Implementation: SqliteStorageAdapter

The default implementation uses SQLite with async queries via `tokio-rusqlite`.

```rust
use agileplus_sqlite::SqliteStorageAdapter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Create or open database
    let storage = SqliteStorageAdapter::new("agileplus.db")?;

    // Create a feature
    let feature = Feature {
        id: 0,
        slug: "001-login".to_string(),
        friendly_name: "User Login System".to_string(),
        state: FeatureState::SPECIFY,
        target_branch: "main".to_string(),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let feature_id = storage.create_feature(&feature).await?;
    println!("Created feature with ID: {}", feature_id);

    // Query features by state
    let implementing = storage
        .list_features_by_state(FeatureState::IMPLEMENT)
        .await?;
    println!("Features in IMPLEMENT: {}", implementing.len());

    Ok(())
}
```

## Custom Implementations

To implement a custom storage backend (database, S3, Firestore):

### 1. Implement the Trait

```rust
use agileplus_domain::ports::StoragePort;
use agileplus_domain::domain::feature::Feature;
use agileplus_domain::error::DomainError;

pub struct MyCustomStorage {
    // Your backend fields
}

#[async_trait::async_trait]
impl StoragePort for MyCustomStorage {
    async fn create_feature(
        &self,
        feature: &Feature,
    ) -> Result<i64, DomainError> {
        // Implement persistence logic
        // Return the assigned feature ID
        Ok(42)
    }

    // Implement all other methods...
}
```

### 2. Wire into CLI

In `main.rs`:

```rust
let storage: Box<dyn StoragePort> = match backend_type {
    "sqlite" => Box::new(SqliteStorageAdapter::new(db_path)?),
    "my-custom" => Box::new(MyCustomStorage::new(config)?),
    _ => panic!("unknown backend"),
};
```

### 3. Configure

```toml
# .kittify/config.toml
[storage]
backend = "my-custom"
url = "postgresql://localhost/agileplus"
```

## Transaction Semantics

All operations are atomic at the database level:

- Feature state transitions are serialized (no concurrent updates)
- Audit entries are append-only with cryptographic chaining
- Work package state changes log an audit entry automatically
- Governance contract versions are immutable (new versions create new records)

## Async Runtime

All trait methods return `impl Future` with `Send` bound, making them compatible with `tokio` and other async runtimes:

```rust
// Tokio runtime
#[tokio::main]
async fn main() -> Result<()> {
    let features = storage.list_all_features().await?;
    Ok(())
}

// Custom runtime
futures::executor::block_on(async {
    let features = storage.list_all_features().await?;
})?;
```
