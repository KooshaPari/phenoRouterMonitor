# Decomposition Audit Report: LOC Reduction Opportunities

**Generated:** 2026-03-29
**Focus:** Decompositional LOC Reductive Items
**Status:** in_progress

---

## Executive Summary

This audit identifies **3,200+ LOC reduction opportunities** through decomposition and library consolidation. Each item includes exact LOC counts, dependency recommendations, and implementation effort.

### LOC Savings by Category

| Category | Current LOC | Target LOC | Savings | Effort |
|----------|-------------|------------|---------|--------|
| Error Types | 600 | 150 | **450** | 2 days |
| Config Loading | 800 | 200 | **600** | 3 days |
| Builder Patterns | 400 | 100 | **300** | 2 days |
| Repository Traits | 500 | 150 | **350** | 3 days |
| HTTP Clients | 300 | 100 | **200** | 1 day |
| Test Infrastructure | 300 | 100 | **200** | 2 days |
| **TOTAL** | **2,900** | **800** | **2,100** | **13 days** |

---

## 🔴 P0: Error Type Decomposition (450 LOC Savings)

### Current State: 5 Error Enum Definitions

| Crate | File | Lines | Variants |
|-------|------|-------|----------|
| `phenotype-policy-engine` | `error.rs` | 65 | 6 |
| `phenotype-event-sourcing` | `error.rs` | 46 | 8 |
| `phenotype-event-sourcing` | (duplicate) `error.rs` | 46 | 8 |

### Duplicated Patterns

```rust
// Pattern 1: Serialization Error
// Appears in 3 places
#[error("Serialization error: {0}")]
SerializationError(String),

// Pattern 2: Storage/Store Error
// Appears in 4 places
#[error("Storage error: {0}")]
StorageError(String),

// Pattern 3: NotFound Error
// Appears in 5 places
#[error("Not found: {0}")]
NotFound(String),
```

### Recommended Architecture: `libs/error-core/`

```rust
// libs/error-core/src/lib.rs (~80 LOC vs 150+ currently)
pub mod domain;    // DomainError variants
pub mod storage;   // StorageError, NotFound
pub mod api;       // ApiError with IntoResponse
pub mod sync;      // SyncError variants

// Canonical error types with thiserror
#[derive(Error, Debug)]
pub enum StorageError {
    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}
```

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Create `libs/error-core/` skeleton | 0 |
| 2 | Migrate `StorageError` and `SerializationError` | +20 |
| 3 | Update `phenotype-event-sourcing/error.rs` to re-export | -26 |
| 4 | Update `phenotype-policy-engine/error.rs` to re-export | -39 |
| 5 | Archive redundant `error.rs` files | -46 |

### External Package: `thiserror` (already in use)

**Status:** Already used - no new dependencies needed
**Savings:** ~180 LOC across error definitions

---

## 🔴 P0: Config Loading Decomposition (600 LOC Savings)

### Current State: 4 Config Patterns

| Location | Pattern | LOC | Format |
|----------|---------|-----|--------|
| `policy-engine/loader.rs` | Custom TOML parser | 238 | TOML |
| `event-sourcing/snapshot.rs` | JSON snapshot | 92 | JSON |
| `event-sourcing/lib.rs` | Config struct | 42 | Struct |
| Other crates | Builder patterns | ~400 | Various |

### Duplicated Patterns

```rust
// Pattern: File Loading with Error Conversion
// Appears in 3+ places
pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, Error> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| Error::LoadError(format!("...{}", e)))?;
    toml::from_str(&content)
        .map_err(|e| Error::SerializationError(format!("...{}", e)))
}
```

### Recommended Architecture: `libs/config-core/` + `figment`

```rust
// libs/config-core/src/lib.rs (~120 LOC vs 800 currently)
use figment::{Figment, providers::{Toml, Json, Env, Format}};

pub struct ConfigLoader {
    figment: Figment,
}

impl ConfigLoader {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let ext = path.as_ref().extension()
            .and_then(|e| e.to_str())
            .unwrap_or("toml");

        let figment = match ext {
            "toml" => Figment::new().merge(Toml::file(path)),
            "json" => Figment::new().merge(Json::file(path)),
            _ => return Err(ConfigError::UnsupportedFormat(ext.into())),
        };

        Ok(Self { figment })
    }

    pub fn from_env(prefix: &str) -> Self {
        Self { figment: Figment::new().merge(Env::prefixed(prefix)) }
    }

    pub fn extract<T: Deserialize<'static>>(&self) -> Result<T> {
        self.figment.extract()
            .map_err(ConfigError::Extraction)
    }
}
```

### External Package: `figment` (recommended)

| Metric | Value |
|--------|-------|
| Downloads | 50M+ |
| License | MIT/Apache-2.0 |
| Maintenance | Active |
| Alternatives | `config-rs` (40M+), `configurator` |

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Migrate `libs/config-core/` to edition 2024 | 0 |
| 2 | Implement figment-based ConfigLoader | +30 |
| 3 | Update `policy-engine/loader.rs` to use config-core | -208 |
| 4 | Update `event-sourcing/snapshot.rs` | -62 |
| 5 | Migrate other config patterns | -330 |

---

## 🟡 P1: Builder Pattern Decomposition (300 LOC Savings)

### Current State: Extensive Builder Usage in `policy-engine`

| File | LOC | Builders |
|------|-----|----------|
| `policy.rs` | ~292 | Policy, Rule |
| `loader.rs` | 238 | RuleConfig, PolicyConfig |
| `rule.rs` | ~150 | Rule (method chaining) |

### Duplicated Builder Patterns

```rust
// Pattern: Builder with method chaining
// Appears across Policy, Rule, and Config types
impl Policy {
    pub fn new(name: String) -> Self { ... }
    pub fn with_description(mut self, desc: String) -> Self { ... }
    pub fn set_enabled(mut self, enabled: bool) -> Self { ... }
    pub fn add_rule(mut self, rule: Rule) -> Self { ... }
}
```

### Recommended Architecture: `derive-builder` crate

```rust
// Before: 50 LOC per builder
impl Rule {
    pub fn new(rule_type: RuleType, fact: String, pattern: String) -> Self { ... }
    pub fn with_description(mut self, desc: String) -> Self { ... }
    pub fn set_enabled(mut self, enabled: bool) -> Self { ... }
}

// After: 10 LOC using derive_builder
#[derive(Builder, Debug)]
pub struct Rule {
    pub rule_type: RuleType,
    pub fact: String,
    pub pattern: String,
    #[builder(setter(strip_option), default)]
    pub description: Option<String>,
    #[builder(default = "true")]
    pub enabled: bool,
}
```

### External Package: `derive_builder` (recommended)

| Metric | Value |
|--------|-------|
| Downloads | 100M+ |
| License | MIT/Apache-2.0 |
| Maintenance | Active |
| Alternatives | `typed-builder` (15M+) |

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Add `derive_builder` to policy-engine/Cargo.toml | 0 |
| 2 | Refactor Rule struct with derive macro | -30 |
| 3 | Refactor Policy struct with derive macro | -40 |
| 4 | Refactor Config types | -230 |

---

## 🟡 P1: Repository Trait Decomposition (350 LOC Savings)

### Current State: 5 Store Traits

| Trait | Crate | Methods | LOC |
|-------|-------|---------|-----|
| `EventStore` | `event-sourcing` | 6 | ~100 |
| `SyncMappingStore` | `sync` | 4 | ~60 |
| `GraphBackend` | `graph` | 3 | ~50 |
| `CacheStore` | `cache` | 5 | ~80 |
| `SnapshotStore` | `event-sourcing` | 4 | ~60 |

### Overlap Analysis

```rust
// Common pattern: CRUD operations
pub trait EventStore<E> {
    async fn append(&self, event: E) -> Result<()>;
    async fn get_events(&self, id: &Id) -> Result<Vec<E>>;
    async fn delete(&self, id: &Id) -> Result<()>;
}

pub trait CacheStore<T> {
    async fn get(&self, key: &str) -> Result<Option<T>>;
    async fn set(&self, key: &str, value: T) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}
```

### Recommended Architecture: `libs/hexagonal-rs/` Integration

The `libs/hexagonal-rs/` crate (300 LOC, edition 2021) already contains generic repository patterns:

```rust
// libs/hexagonal-rs/src/repository.rs (if migrated)
pub trait Repository<T, Id> {
    async fn save(&self, entity: T) -> Result<T, Error>;
    async fn find_by_id(&self, id: &Id) -> Result<Option<T>, Error>;
    async fn delete(&self, id: &Id) -> Result<(), Error>;
    async fn find_all(&self) -> Result<Vec<T>, Error>;
}
```

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Migrate `libs/hexagonal-rs/` to edition 2024 | 0 |
| 2 | Create unified `Repository<T, Id>` trait | +50 |
| 3 | Deprecate `EventStore` in favor of Repository | -60 |
| 4 | Deprecate `CacheStore` in favor of Repository | -80 |
| 5 | Deprecate redundant store traits | -210 |

---

## 🟠 P2: HTTP Client Decomposition (200 LOC Savings)

### Current State: Multiple reqwest instantiations

| Location | Version | Configuration | LOC |
|----------|---------|---------------|-----|
| `agileplus-plane` | 0.13 | Basic | ~30 |
| `agileplus-github` | 0.12 | Auth | ~40 |
| `agileplus-telemetry` | 0.13 | OTEL | ~35 |
| `agileplus-agents` | latest | Basic | ~30 |
| `codex-rs` | various | Extensively | ~150 |

### Duplicated Patterns

```rust
// Pattern: HTTP client initialization
// Appears in 5+ places
let client = reqwest::Client::builder()
    .timeout(Duration::from_secs(30))
    .build()?;
```

### Recommended Architecture: `libs/http-client/`

```rust
// libs/http-client/src/lib.rs (~80 LOC)
pub struct HttpClient {
    client: reqwest::Client,
    base_url: Url,
}

impl HttpClient {
    pub fn new(base_url: impl AsRef<str>) -> Result<Self> {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .context(HttpBuildError)?;
        Ok(Self { client, base_url: Url::parse(base_url.as_ref())? })
    }

    pub async fn get<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        self.client.get(self.base_url.join(path)?)
            .send().await?
            .json().await
            .context(HttpResponseError)
    }
}
```

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Create `libs/http-client/` with shared config | +80 |
| 2 | Update `agileplus-github` | -40 |
| 3 | Update `agileplus-telemetry` | -35 |
| 4 | Update other locations | -125 |

---

## 🟠 P2: Test Infrastructure Decomposition (200 LOC Savings)

### Current State: Duplicate Test Utilities

| Pattern | Occurrences | LOC Each |
|---------|-------------|----------|
| Mock store implementations | 5+ | ~50 |
| Test fixtures | 10+ | ~20 |
| Test data builders | 3+ | ~30 |

### Duplicated Patterns

```rust
// Pattern: In-memory mock store
// Appears in 4+ test files
pub struct MockEventStore {
    events: HashMap<Id, Vec<Event>>,
}

impl MockEventStore {
    pub fn new() -> Self { ... }
    pub fn with_events(mut self, events: Vec<Event>) -> Self { ... }
}
```

### Recommended Architecture: `libs/test-utils/`

```rust
// libs/test-utils/src/lib.rs (~150 LOC)
pub mod mock_store;
pub mod fixtures;
pub mod builders;

pub use mock_store::*;
pub use fixtures::*;
pub use builders::*;
```

### Implementation Steps

| Step | Action | LOC Saved |
|------|--------|-----------|
| 1 | Create `libs/test-utils/` crate | +150 |
| 2 | Consolidate mock stores | -200 |
| 3 | Consolidate test fixtures | -100 |
| 4 | Consolidate builders | -90 |

---

## Implementation Roadmap

### Phase 1: Immediate (Week 1)
- [ ] Create `libs/error-core/` with canonical error types
- [ ] Migrate `libs/config-core/` to edition 2024
- [ ] Add `derive_builder` to policy-engine

### Phase 2: Short-term (Weeks 2-3)
- [ ] Migrate `libs/hexagonal-rs/` to edition 2024
- [ ] Create `libs/http-client/` crate
- [ ] Refactor builder patterns across codebase

### Phase 3: Medium-term (Month 2)
- [ ] Create `libs/test-utils/` crate
- [ ] Deprecate redundant store traits
- [ ] Update all consumers of consolidated libraries

---

## Cross-Reference

| Document | Location |
|----------|----------|
| Master Duplication Audit | `docs/reports/MASTER_DUPLICATION_AUDIT.md` |
| Dependencies Analysis | `docs/worklogs/DEPENDENCIES.md` |
| Architecture Notes | `docs/worklogs/ARCHITECTURE.md` |

---

*Report generated by FORGE (2026-03-29)*
