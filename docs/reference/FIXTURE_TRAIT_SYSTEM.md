# Fixture Trait System & Architectural Patterns

**Purpose**: Define trait abstractions, builder patterns, and factory design for test-fixtures-shared
**Audience**: Implementers, architects, maintainers
**Status**: SPECIFICATION DOCUMENT (implements existing design docs)

---

## Part 1: Core Trait System

### 1.1 Generic FixtureBuilder Trait

```rust
/// Universal trait for all fixture builders.
/// 
/// Enables:
/// - Generic testing frameworks to work with any builder
/// - Macro-driven builder generation in Phase 2+
/// - Plugin systems to register custom builders
/// - Reflection/introspection of builder capabilities
pub trait FixtureBuilder<T>: Sized {
    /// Build and return the constructed object.
    fn build(self) -> T;
    
    /// Get a display name for debugging.
    fn fixture_name() -> &'static str;
}

/// Optional: extended trait for builders with ID fields.
pub trait WithId: Sized {
    fn with_id(self, id: i64) -> Self;
}

/// Optional: extended trait for builders with state/status fields.
pub trait WithState<S>: Sized {
    fn with_state(self, state: S) -> Self;
}
```

### 1.2 Generic FixtureFactory Trait

```rust
/// Universal trait for test data factories.
/// 
/// Enables:
/// - Simple, consistent API for creating test data
/// - Batch creation for load testing
/// - Named fixture creation for readability
pub trait FixtureFactory<T> {
    /// Create a single test data item with default name.
    fn create(name: &str) -> T;
    
    /// Create multiple items in batch.
    fn create_batch(count: usize) -> Vec<T> {
        (0..count)
            .map(|i| Self::create(&format!("item-{}", i)))
            .collect()
    }
    
    /// Create from template with overrides.
    fn create_with(name: &str, overrides: std::collections::HashMap<&str, &str>) -> T {
        let _ = overrides; // implement in concrete types
        Self::create(name)
    }
}
```

### 1.3 MockStorage Port Trait

```rust
/// Trait for mocking storage backends.
pub trait StoragePort: Send + Sync {
    type Feature;
    type WorkPackage;
    
    fn store_feature(&self, feature: Self::Feature) -> Result<i64>;
    fn get_feature(&self, id: i64) -> Result<Option<Self::Feature>>;
    fn list_features(&self) -> Result<Vec<Self::Feature>>;
}

/// Implementation for testing.
pub struct MockStorage {
    pub features: Arc<Mutex<Vec<Feature>>>,
    pub work_packages: Arc<Mutex<Vec<WorkPackage>>>,
}

impl StoragePort for MockStorage {
    type Feature = Feature;
    type WorkPackage = WorkPackage;
    
    fn store_feature(&self, feature: Self::Feature) -> Result<i64> {
        let id = feature.id;
        self.features.lock().unwrap().push(feature);
        Ok(id)
    }
    
    // ... other trait methods
}
```

---

## Part 2: Builder Pattern Implementations

### 2.1 Builder Pattern Structure (Template)

All builders follow this structure:

```rust
/// Builder for TargetType domain objects.
///
/// # Examples
///
/// ```ignore
/// let obj = TargetTypeFixture::new(required_field)
///     .id(42)
///     .with_state(SomeState::Active)
///     .build();
/// ```
pub struct TargetTypeFixture {
    // One field per TargetType field (required + optional)
    id: i64,
    name: String,
    state: StateEnum,
    optional_field: Option<String>,
    // ...
}

impl TargetTypeFixture {
    /// Create with required fields only.
    pub fn new(name: &str) -> Self {
        Self {
            id: 1,                           // default
            name: name.to_string(),          // required
            state: StateEnum::Default,       // default state
            optional_field: None,            // optional
        }
    }
    
    /// Set id (fluent API).
    pub fn id(mut self, id: i64) -> Self {
        self.id = id;
        self
    }
    
    /// Common state transitions (convenience methods).
    pub fn with_active(mut self) -> Self {
        self.state = StateEnum::Active;
        self
    }
    
    /// Build the final object.
    pub fn build(self) -> TargetType {
        TargetType {
            id: self.id,
            name: self.name,
            state: self.state,
            optional_field: self.optional_field,
        }
    }
}

// Implement trait for generic test frameworks
impl FixtureBuilder<TargetType> for TargetTypeFixture {
    fn build(self) -> TargetType {
        TargetTypeFixture::build(self)
    }
    
    fn fixture_name() -> &'static str {
        "TargetType"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn new_has_defaults() {
        let obj = TargetTypeFixture::new("test").build();
        assert_eq!(obj.name, "test");
        assert_eq!(obj.state, StateEnum::Default);
    }
    
    #[test]
    fn fluent_api_works() {
        let obj = TargetTypeFixture::new("test")
            .id(42)
            .with_active()
            .build();
        assert_eq!(obj.id, 42);
        assert_eq!(obj.state, StateEnum::Active);
    }
}
```

### 2.2 Concrete Builder Examples

#### FeatureFixture
```rust
pub struct FeatureFixture {
    id: i64,
    slug: String,
    friendly_name: String,
    state: FeatureState,
    spec_hash: [u8; 32],
    target_branch: String,
    plane_issue_id: Option<String>,
    plane_state_id: Option<String>,
    labels: Vec<String>,
    module_id: Option<i64>,
    project_id: Option<i64>,
}

impl FeatureFixture {
    pub fn new(slug: &str, friendly_name: &str) -> Self { /* ... */ }
    pub fn id(mut self, id: i64) -> Self { self.id = id; self }
    pub fn state(mut self, state: FeatureState) -> Self { self.state = state; self }
    pub fn with_shipped(mut self) -> Self { self.state = FeatureState::Shipped; self }
    pub fn with_implementing(mut self) -> Self { self.state = FeatureState::Implementing; self }
    pub fn with_label(mut self, label: &str) -> Self { 
        self.labels.push(label.to_string()); 
        self 
    }
    pub fn with_project_id(mut self, project_id: i64) -> Self { 
        self.project_id = Some(project_id); 
        self 
    }
    pub fn build(self) -> Feature { /* ... */ }
}

impl FixtureBuilder<Feature> for FeatureFixture {
    fn build(self) -> Feature { FeatureFixture::build(self) }
    fn fixture_name() -> &'static str { "Feature" }
}
```

#### WorkPackageFixture
```rust
pub struct WorkPackageFixture {
    id: i64,
    feature_id: i64,
    title: String,
    state: WpState,
    sequence: i32,
    file_scope: Vec<String>,
    acceptance_criteria: String,
    agent_id: Option<String>,
    pr_url: Option<String>,
    pr_state: Option<String>,
    worktree_path: Option<String>,
    plane_sub_issue_id: Option<String>,
}

impl WorkPackageFixture {
    pub fn new(feature_id: i64, title: &str) -> Self { /* ... */ }
    pub fn id(mut self, id: i64) -> Self { self.id = id; self }
    pub fn state(mut self, state: WpState) -> Self { self.state = state; self }
    pub fn done(mut self) -> Self { self.state = WpState::Done; self }
    pub fn in_progress(mut self) -> Self { self.state = WpState::InProgress; self }
    pub fn with_pr(mut self, pr_url: &str) -> Self { 
        self.pr_url = Some(pr_url.to_string()); 
        self.pr_state = Some("merged".to_string()); 
        self 
    }
    pub fn with_agent_id(mut self, agent_id: &str) -> Self { 
        self.agent_id = Some(agent_id.to_string()); 
        self 
    }
    pub fn build(self) -> WorkPackage { /* ... */ }
}

impl FixtureBuilder<WorkPackage> for WorkPackageFixture {
    fn build(self) -> WorkPackage { WorkPackageFixture::build(self) }
    fn fixture_name() -> &'static str { "WorkPackage" }
}
```

---

## Part 3: Factory Pattern Implementations

### 3.1 Factory Pattern Structure (Template)

```rust
/// Factory for creating test data of a specific type.
///
/// # Examples
///
/// ```ignore
/// let event = EventFactory::order_event(100.0, "pending");
/// let batch = EventFactory::create_batch(10);
/// ```
pub struct TargetTypeFactory;

impl TargetTypeFactory {
    /// Create a single instance with minimal defaults.
    pub fn create(name: &str) -> TargetType {
        TargetType {
            name: name.to_string(),
            // ... sensible defaults
        }
    }
    
    /// Create with all parameters specified (no defaults).
    pub fn create_with(field1: &str, field2: u32) -> TargetType {
        TargetType {
            field1: field1.to_string(),
            field2,
            // ... required fields only
        }
    }
    
    /// Common variants (convenience constructors).
    pub fn active_variant() -> TargetType {
        Self::create_with("default", 1)
    }
    
    pub fn inactive_variant() -> TargetType {
        Self::create_with("inactive", 0)
    }
}

impl FixtureFactory<TargetType> for TargetTypeFactory {
    fn create(name: &str) -> TargetType {
        TargetTypeFactory::create(name)
    }
    
    fn create_batch(count: usize) -> Vec<TargetType> {
        (0..count)
            .map(|i| Self::create(&format!("item-{}", i)))
            .collect()
    }
}
```

### 3.2 Concrete Factory Examples

#### EventFactory
```rust
pub struct EventFactory;

impl EventFactory {
    pub fn order_event(amount: f64, status: &str) -> EventEnvelope<Order> {
        EventEnvelope::new(
            Order {
                id: uuid::Uuid::new_v4().to_string(),
                amount,
                status: status.to_string(),
            },
            "test-user",
        )
    }
    
    pub fn user_event(name: &str, email: &str) -> EventEnvelope<User> {
        EventEnvelope::new(
            User {
                name: name.to_string(),
                email: email.to_string(),
            },
            "admin",
        )
    }
    
    pub fn create_batch_orders(count: usize, amount: f64) -> Vec<EventEnvelope<Order>> {
        (0..count)
            .map(|_| Self::order_event(amount, "pending"))
            .collect()
    }
}

impl FixtureFactory<Order> for EventFactory {
    fn create(name: &str) -> Order {
        Order {
            id: name.to_string(),
            amount: 100.0,
            status: "pending".to_string(),
        }
    }
}
```

#### CacheFactory
```rust
pub struct CacheFactory;

impl CacheFactory {
    pub fn cache_entry(key: &str, ttl_secs: u64) -> CacheEntry {
        CacheEntry {
            key: key.to_string(),
            value: serde_json::json!({"cached": true}),
            ttl_secs,
            created_at: Utc::now(),
        }
    }
    
    pub fn expired_entry(key: &str) -> CacheEntry {
        Self::cache_entry(key, 0) // ttl_secs = 0 means expired
    }
    
    pub fn long_lived_entry(key: &str) -> CacheEntry {
        Self::cache_entry(key, 86400) // 24 hours
    }
}
```

---

## Part 4: Composition Patterns

### 4.1 Builder Composition for Complex Objects

```rust
// Create complex object from multiple builders

#[test]
fn test_feature_with_complete_workflow() {
    let feature = FeatureFixture::new("complete-feature", "Complete Feature")
        .id(1)
        .with_shipped()
        .build();
    
    let wps = vec![
        WorkPackageFixture::new(1, "WP01").id(1).done().build(),
        WorkPackageFixture::new(1, "WP02").id(2).done().build(),
        WorkPackageFixture::new(1, "WP03").id(3).in_progress().build(),
    ];
    
    let storage = MockStorage::new()
        .with_feature(feature)
        .with_work_packages(wps);
    
    // ... use storage in test
}
```

### 4.2 Factory Batch Creation for Load Testing

```rust
#[test]
fn test_cache_with_many_entries() {
    let entries = CacheFactory::create_batch(1000);
    let cache = MockCache::new();
    
    for entry in entries {
        cache.set(&entry.key, entry.value.clone()).unwrap();
    }
    
    assert_eq!(cache.len(), 1000);
}
```

### 4.3 Mixed Builder + Factory Patterns

```rust
#[test]
fn test_order_events_with_workpackages() {
    let orders = EventFactory::create_batch_orders(5, 100.0);
    let wps = vec![
        WorkPackageFixture::new(1, "WP01").done().build(),
        WorkPackageFixture::new(1, "WP02").in_progress().build(),
    ];
    
    let storage = MockStorage::new()
        .with_work_packages(wps);
    
    for order in orders {
        // Process order with storage
    }
}
```

---

## Part 5: Advanced Patterns

### 5.1 Builder with Defaults Trait

```rust
/// For builders that support default instance creation.
pub trait WithDefaults: Sized {
    fn defaults() -> Self;
    fn from_defaults() -> Self::Output;
}

impl WithDefaults for FeatureFixture {
    fn defaults() -> Self {
        Self::new("default-feature", "Default Feature")
    }
    
    fn from_defaults() -> Feature {
        Self::defaults().build()
    }
}

#[test]
fn test_with_defaults() {
    let feature = Feature::from_defaults();
    assert_eq!(feature.slug, "default-feature");
}
```

### 5.2 Builder with Clone + Modify Pattern

```rust
pub struct FeatureFixture { /* ... */ }

impl FeatureFixture {
    /// Clone this builder and modify.
    pub fn clone_and_modify<F>(self, f: F) -> Self 
    where 
        F: FnOnce(Self) -> Self 
    {
        f(self)
    }
}

#[test]
fn test_clone_and_modify() {
    let base = FeatureFixture::new("test", "Test Feature");
    let modified = base.clone_and_modify(|b| b.id(42).with_shipped());
    let feature = modified.build();
    assert_eq!(feature.id, 42);
}
```

### 5.3 Builder Validation Before Build

```rust
pub struct AuditChainFixture {
    chain: Vec<AuditEntry>,
}

impl AuditChainFixture {
    pub fn build_validated(self) -> Result<Vec<AuditEntry>, ValidationError> {
        // Verify chain integrity
        if self.chain.is_empty() {
            return Err(ValidationError::EmptyChain);
        }
        
        // Verify hash linkage
        for i in 1..self.chain.len() {
            if self.chain[i].prev_hash != self.chain[i-1].hash {
                return Err(ValidationError::BrokenHashChain { at: i });
            }
        }
        
        Ok(self.chain)
    }
}
```

---

## Part 6: Testing the Fixtures Themselves

### 6.1 Builder Unit Tests

```rust
#[cfg(test)]
mod feature_fixture_tests {
    use super::*;
    
    #[test]
    fn new_feature_has_sensible_defaults() {
        let f = FeatureFixture::new("my-feature", "My Feature").build();
        assert_eq!(f.slug, "my-feature");
        assert_eq!(f.friendly_name, "My Feature");
        assert_eq!(f.state, FeatureState::Created);
        assert_eq!(f.target_branch, "main");
    }
    
    #[test]
    fn id_method_sets_id() {
        let f = FeatureFixture::new("test", "Test")
            .id(42)
            .build();
        assert_eq!(f.id, 42);
    }
    
    #[test]
    fn with_shipped_sets_state() {
        let f = FeatureFixture::new("test", "Test")
            .with_shipped()
            .build();
        assert_eq!(f.state, FeatureState::Shipped);
    }
    
    #[test]
    fn with_label_adds_multiple_labels() {
        let f = FeatureFixture::new("test", "Test")
            .with_label("platform")
            .with_label("infrastructure")
            .with_label("critical")
            .build();
        assert_eq!(f.labels.len(), 3);
        assert!(f.labels.contains(&"platform".to_string()));
    }
    
    #[test]
    fn chaining_multiple_methods_works() {
        let f = FeatureFixture::new("test", "Test")
            .id(1)
            .with_shipped()
            .with_label("urgent")
            .with_project_id(100)
            .build();
        
        assert_eq!(f.id, 1);
        assert_eq!(f.state, FeatureState::Shipped);
        assert_eq!(f.project_id, Some(100));
    }
}
```

### 6.2 Factory Unit Tests

```rust
#[cfg(test)]
mod event_factory_tests {
    use super::*;
    
    #[test]
    fn order_event_has_valid_structure() {
        let event = EventFactory::order_event(100.0, "pending");
        assert_eq!(event.data.amount, 100.0);
        assert_eq!(event.data.status, "pending");
        assert!(!event.data.id.is_empty());
    }
    
    #[test]
    fn create_batch_generates_n_items() {
        let items = EventFactory::create_batch(10);
        assert_eq!(items.len(), 10);
    }
    
    #[test]
    fn create_batch_ids_are_unique() {
        let events = EventFactory::create_batch_orders(5, 100.0);
        let ids: Vec<_> = events.iter().map(|e| &e.data.id).collect();
        let unique_ids: std::collections::HashSet<_> = ids.iter().collect();
        assert_eq!(ids.len(), unique_ids.len());
    }
}
```

---

## Part 7: Integration with Testing Frameworks

### 7.1 pytest/tokio Integration

```rust
#[tokio::test]
async fn test_with_fixture_builder() {
    let feature = FeatureFixture::new("async-test", "Async Test")
        .id(1)
        .build();
    
    let server = TestServerFixture::new().await;
    let resp = server
        .post("/api/v1/features")
        .json(&feature)
        .await;
    
    assert_eq!(resp.status(), StatusCode::CREATED);
}
```

### 7.2 Parameterized Testing with Factories

```rust
#[test]
fn test_cache_with_multiple_ttls() {
    for ttl in [0, 3600, 86400] {
        let entry = CacheFactory::cache_entry("key", ttl);
        let cache = MockCache::new();
        cache.set(&entry.key, entry.value.clone()).unwrap();
        
        if ttl > 0 {
            assert!(cache.get("key").is_some());
        }
    }
}
```

---

## Part 8: Documentation & Conventions

### 8.1 Builder Method Naming Conventions

| Pattern | Usage | Example |
|---------|-------|---------|
| `with_*` | Optional single-value field | `with_label()`, `with_project_id()` |
| `set_*` | Required field modification | `set_state()` (rare, prefer state-specific) |
| `*_*` | Common state transitions | `with_shipped()`, `done()`, `in_progress()` |
| `add_*` | Collections | `add_label()` (alternative to `with_label()`) |

### 8.2 Factory Method Naming Conventions

| Pattern | Usage | Example |
|---------|-------|---------|
| `<type>_<variant>` | Named variants | `order_event()`, `user_event()` |
| `create` | Default/simple | `create(name)` |
| `create_*` | Specific | `create_batch()`, `create_with()` |
| `<state>_variant` | State-specific | `active_variant()`, `expired_entry()` |

### 8.3 Documentation Template

```rust
/// Builder for creating Feature test fixtures with fluent API.
///
/// # Examples
///
/// Basic usage:
/// ```ignore
/// let feature = FeatureFixture::new("my-feature", "My Feature")
///     .id(1)
///     .with_shipped()
///     .build();
/// ```
///
/// Multiple labels:
/// ```ignore
/// let feature = FeatureFixture::new("complex", "Complex")
///     .with_label("platform")
///     .with_label("critical")
///     .with_project_id(100)
///     .build();
/// ```
///
/// # Defaults
/// - `id`: 1
/// - `state`: FeatureState::Created
/// - `target_branch`: "main"
/// - `labels`: empty vec
/// - `plane_issue_id`: None
pub struct FeatureFixture { /* ... */ }
```

---

## Summary: Trait System Architecture

```
┌─────────────────────────────────────────────┐
│  Generic Traits (trait system foundation)  │
├─────────────────────────────────────────────┤
│ - FixtureBuilder<T>                         │
│ - FixtureFactory<T>                         │
│ - WithId / WithState (extensions)           │
│ - WithDefaults / WithValidation (advanced) │
└─────────────────────────────────────────────┘
                       △
                       │ implements
                       │
         ┌─────────────┼─────────────┐
         │             │             │
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │  Builders  │  │ Factories  │  │    Mocks   │
    ├────────────┤  ├────────────┤  ├────────────┤
    │ Feature    │  │ Event      │  │ Storage    │
    │ WorkPackage│  │ Cache      │  │ TestServer │
    │ Audit      │  │ Policy     │  │            │
    │ Project    │  │            │  │            │
    │ Cycle      │  │            │  │            │
    │ Module     │  │            │  │            │
    └────────────┘  └────────────┘  └────────────┘
         │                │                │
         └────────────────┼────────────────┘
                          │
                          ▼
         ┌─────────────────────────────────┐
         │  Integration with Test Suites  │
         │  (via test_fixtures_shared)    │
         └─────────────────────────────────┘
```

---

**Document**: FIXTURE_TRAIT_SYSTEM.md
**Status**: SPECIFICATION & REFERENCE (implements existing work)
**Ready**: Yes, aligns with CODE_EXAMPLES.md implementations
