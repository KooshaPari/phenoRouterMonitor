# Config Consolidation: 5-Phase Migration Plan

**Target:** Merge 4+ config implementations into unified phenotype-config-core
**Timeline:** ~5-7 working days
**Risk Level:** Low (minimal breaking changes)

---

## Phase Overview

```
Phase 1: Prepare Core (1-2 days)
  ├─ Expand phenotype-config-core with traits
  └─ No breaking changes

Phase 2: Migrate Config Loader (2-3 days)
  ├─ Refactor AppConfigLoader
  └─ Minor breaking change (error type)

Phase 3: Migrate Policy Engine (1-2 days)
  ├─ Use ConfigError from core
  └─ No breaking changes

Phase 4: Consolidate Telemetry & Event-Sourcing (1 day)
  ├─ Add validators
  └─ No breaking changes

Phase 5: Align Contracts (1 day)
  ├─ Update trait definitions
  └─ Minor breaking change (re-export)
```

---

## Phase 1: Prepare phenotype-config-core (1-2 days)

### Deliverables

1. Expand ConfigError with all variants
2. Implement ConfigLoader & ConfigLoaderSync traits
3. Enhance ConfigSource trait
4. Implement ConfigValidator trait
5. Add ConfigProvider trait
6. Error conversions from standard types

### Files to Create/Modify

```
crates/phenotype-config-core/src/
├── lib.rs (update re-exports)
├── error.rs (expand)
├── loader.rs (NEW, 100 LOC)
├── source.rs (enhance)
├── validator.rs (NEW, 80 LOC)
└── provider.rs (NEW, 50 LOC)
```

### Key Code Changes

**error.rs (expanded):**
```rust
#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("io error: {0}")] Io(#[from] std::io::Error),
    #[error("parse error: {0}")] Parse(String),
    #[error("toml error: {0}")] TomlError(String),
    #[error("json error: {0}")] JsonError(String),
    #[error("figment error: {0}")] FigmentError(String),
    #[error("validation error: {0}")] Validation(String),
    #[error("invalid configuration: {0}")] Invalid(String),
    #[error("invalid value: {field} — {reason}")]
    InvalidValue { field: String, reason: String },
    #[error("not found: {0}")] NotFound(String),
    #[error("key not found: {key} in {source}")]
    KeyNotFound { key: String, source: String },
    #[error("other error: {0}")] Other(String),
}

impl From<toml::de::Error> for ConfigError { ... }
impl From<serde_json::Error> for ConfigError { ... }
```

**loader.rs (NEW):**
```rust
#[async_trait]
pub trait ConfigLoader: Send + Sync + 'static {
    type Config: Debug + Send + Sync + serde::de::DeserializeOwned + 'static;
    async fn load(&self) -> Result<Self::Config, ConfigError>;
    async fn reload(&self) -> Result<Self::Config, ConfigError> { self.load().await }
}

pub trait ConfigLoaderSync: Send + Sync + 'static {
    type Config: Debug + Send + Sync + serde::de::DeserializeOwned + 'static;
    fn load(&self) -> Result<Self::Config, ConfigError>;
    fn parse(input: &str) -> Result<Self::Config, ConfigError>;
}
```

**validator.rs (NEW):**
```rust
pub trait ConfigValidator: Send + Sync + 'static {
    type Config: Debug + Send + Sync + 'static;
    fn validate(&self, config: &Self::Config) -> Result<(), ConfigError>;
}

pub struct ChainValidator<V1, V2>(pub V1, pub V2);
impl<C, V1, V2> ConfigValidator for ChainValidator<V1, V2> { ... }
```

**provider.rs (NEW):**
```rust
pub trait ConfigProvider: Send + Sync + 'static {
    type Config: Debug + Send + Sync + 'static;
    fn config(&self) -> &Self::Config;
}

pub struct DefaultConfigProvider<C> { config: Arc<C> }
```

### Testing

- 50+ unit tests for core traits
- Error conversion tests (20+ tests)
- Fixture tests for validators (15+ tests)

### Verification Checklist

- [ ] All new modules compile without warnings
- [ ] `cargo test --package phenotype-config-core` passes
- [ ] `cargo clippy --package phenotype-config-core` passes
- [ ] No unsafe code introduced
- [ ] Error conversions tested
- [ ] Documentation complete

### Breaking Changes

None — entirely additive.

---

## Phase 2: Migrate phenotype-config-loader (2-3 days)

### Scope

Refactor AppConfigLoader to use new core traits and error types while maintaining backward compatibility.

### Deliverables

1. Create FigmentConfigLoader implementing ConfigLoader trait
2. Create FigmentConfigLoaderSync implementing ConfigLoaderSync trait
3. Move helper structs (DatabaseConfig, CacheConfig, ServerConfig) to shared module
4. Update error handling to use ConfigError
5. Keep AppConfigLoader as deprecated alias

### Files to Modify

```
crates/phenotype-config-loader/src/
├── lib.rs (refactor)
├── figment_loader.rs (NEW, ~150 LOC)
├── figment_sync_loader.rs (NEW, ~100 LOC)
├── helpers/
│   ├── mod.rs (NEW, re-exports)
│   ├── database.rs (NEW, ~30 LOC)
│   ├── cache.rs (NEW, ~30 LOC)
│   └── server.rs (NEW, ~30 LOC)
└── tests/
    ├── figment_loader_tests.rs (NEW, ~80 LOC)
    └── backward_compat_tests.rs (NEW, ~50 LOC)
```

### Key Code Changes

**figment_loader.rs (NEW):**
```rust
#[async_trait]
impl<T: DeserializeOwned + Send + 'static> ConfigLoader for FigmentConfigLoader {
    type Config = T;

    async fn load(&self) -> Result<T, ConfigError> {
        let figment = self.build_figment()?;
        tokio::task::spawn_blocking(move || {
            figment.extract::<T>()
                .map_err(|e| ConfigError::FigmentError(e.to_string()))
        })
        .await
        .map_err(|e| ConfigError::Other(e.to_string()))?
    }
}
```

**lib.rs (update):**
```rust
pub use figment_loader::FigmentConfigLoader;

#[deprecated(since = "0.3.0", note = "use FigmentConfigLoader instead")]
pub type AppConfigLoader = FigmentConfigLoader;
```

### Testing

- All existing tests updated to use new error type
- New tests for ConfigLoader trait implementation
- Backward compatibility tests with AppConfigLoader alias
- 80+ tests total

### Breaking Changes

| Change | Impact | Migration |
|--------|--------|-----------|
| ConfigLoaderError → ConfigError | Direct dependents | 0 crates depend on it |
| AppConfigLoader → FigmentConfigLoader | Code using it | Use new name (old deprecated) |
| Sync load() becomes async | Downstream | Update error handling |

### Deprecation Timeline

v0.3.0: AppConfigLoader deprecated, new name required for future versions
v0.4.0: AppConfigLoader removed, ConfigLoaderError removed

---

## Phase 3: Migrate phenotype-policy-engine (1-2 days)

### Scope

Update policy engine to use ConfigError from core and implement ConfigLoader trait.

### Deliverables

1. Refactor to use ConfigError
2. Implement ConfigLoader trait for PolicyConfigLoader
3. Add ConfigValidator for policy validation
4. Keep PolicyEngineError for policy execution errors

### Files to Modify

```
crates/phenotype-policy-engine/src/
├── loader.rs (refactor)
├── validator.rs (NEW)
└── tests/ (update)
```

### Key Code Changes

**loader.rs (update):**
```rust
pub struct PolicyConfigLoader {
    path: std::path::PathBuf,
}

#[async_trait]
impl ConfigLoader for PolicyConfigLoader {
    type Config = Vec<Policy>;

    async fn load(&self) -> Result<Vec<Policy>, ConfigError> {
        let path = self.path.clone();
        let policies_config = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| ConfigError::Io(e))?;

        let file: PoliciesConfigFile = toml::from_str(&policies_config)
            .map_err(|e| ConfigError::TomlError(e.to_string()))?;

        file.to_policies()
            .map_err(|e| ConfigError::Other(e.to_string()))
    }
}
```

**validator.rs (NEW):**
```rust
pub struct PolicyValidator;

impl ConfigValidator for PolicyValidator {
    type Config = Vec<Policy>;

    fn validate(&self, policies: &Vec<Policy>) -> Result<(), ConfigError> {
        if policies.is_empty() {
            return Err(ConfigError::validation("at least one policy required"));
        }
        // ... additional validation ...
        Ok(())
    }
}
```

### Breaking Changes

None — PolicyEngineError still exists for policy execution.

### Testing

- New tests for PolicyConfigLoader as ConfigLoader
- Validator tests
- 40+ tests total

---

## Phase 4: Consolidate Telemetry & Event-Sourcing (1 day)

### Scope

Add validators for telemetry and event sourcing configs.

### Files to Create

```
crates/phenotype-telemetry/src/
├── validator.rs (NEW)
└── tests/validator_tests.rs (NEW)

crates/phenotype-event-sourcing/src/
├── validator.rs (NEW)
└── tests/validator_tests.rs (NEW)
```

### Code Changes

**phenotype-telemetry/src/validator.rs:**
```rust
pub struct TelemetryConfigValidator;

impl ConfigValidator for TelemetryConfigValidator {
    type Config = TelemetryConfig;

    fn validate(&self, config: &TelemetryConfig) -> Result<(), ConfigError> {
        if config.service_name.is_empty() {
            return Err(ConfigError::invalid_value("service_name", "cannot be empty"));
        }
        Ok(())
    }
}
```

**phenotype-event-sourcing/src/validator.rs:**
```rust
pub struct SnapshotConfigValidator;

impl ConfigValidator for SnapshotConfigValidator {
    type Config = SnapshotConfig;

    fn validate(&self, config: &SnapshotConfig) -> Result<(), ConfigError> {
        if config.max_events == 0 {
            return Err(ConfigError::invalid_value("max_events", "must be > 0"));
        }
        Ok(())
    }
}
```

### Breaking Changes

None — purely additive.

### Testing

- Unit tests for each validator
- 20+ tests total

---

## Phase 5: Align phenotype-contracts (1 day)

### Scope

Update ConfigLoader trait in contracts to use ConfigError and re-export from core.

### Files to Modify

```
crates/phenotype-contracts/src/
├── outbound.rs (refactor ConfigLoader)
└── lib.rs (update re-exports)
```

### Code Changes

**outbound.rs (update):**
```rust
// Re-export from config-core
pub use phenotype_config_core::ConfigLoader;

// Provide conversion for old code
impl From<ConfigError> for ContractError {
    fn from(err: ConfigError) -> Self {
        match err {
            ConfigError::Validation(msg) => ContractError::InvalidConfiguration(msg),
            ConfigError::Invalid(msg) => ContractError::InvalidConfiguration(msg),
            ConfigError::NotFound(msg) => ContractError::InvalidConfiguration(msg),
            _ => ContractError::Internal(err.to_string()),
        }
    }
}
```

### Breaking Changes

- ConfigLoader trait re-exported from config-core (import path change)

### Testing

- Update existing contract tests
- New tests for error conversion
- 30+ tests total

---

## Execution Checklist

### Phase 1

- [ ] Create error.rs with all variants
- [ ] Add From impls for standard errors
- [ ] Create loader.rs with trait definitions
- [ ] Create validator.rs with trait
- [ ] Create provider.rs with trait
- [ ] Update lib.rs re-exports
- [ ] Run cargo test --package phenotype-config-core
- [ ] Run cargo clippy --package phenotype-config-core
- [ ] Commit: "feat(config-core): add v2 traits and error types"

### Phase 2

- [ ] Create figment_loader.rs
- [ ] Create figment_sync_loader.rs
- [ ] Move helpers to helpers/ submodule
- [ ] Update lib.rs with deprecation alias
- [ ] Update all tests
- [ ] Add backward compat tests
- [ ] Run cargo test --package phenotype-config-loader
- [ ] Verify clippy
- [ ] Commit: "refactor(config-loader): implement ConfigLoader trait"

### Phases 3-5

- [ ] Follow similar patterns per phase
- [ ] Run full test suite after each phase
- [ ] Verify no clippy warnings
- [ ] Commit per phase

### Final Verification

- [ ] All tests pass: cargo test --workspace
- [ ] All docs build: cargo doc --workspace --no-deps
- [ ] No clippy warnings: cargo clippy --workspace
- [ ] Feature flags work: cargo test --no-default-features
- [ ] Create release PR with summary
- [ ] Update CHANGELOG.md
- [ ] Tag v0.3.0 with release notes

---

## Rollback Plan

If critical issues arise:

**Phase 1:** Revert config-core changes (no dependencies, zero impact)
**Phase 2:** Revert to ConfigLoaderError in config-loader
**Phase 3:** Keep policy engine as-is
**Phase 4:** No config validator changes
**Phase 5:** Keep contracts trait as-is

```bash
git revert <phase-commit-hash>
cargo test --workspace
```

---

## Success Criteria

| Criterion | Measure | Status |
|-----------|---------|--------|
| Phase 1 complete | config-core compiles, tests pass | Pending |
| Phase 2 complete | config-loader migrated, backward compat verified | Pending |
| Phase 3 complete | policy-engine updated | Pending |
| Phase 4 complete | telemetry/event-sourcing validators added | Pending |
| Phase 5 complete | contracts aligned | Pending |
| Total LOC reduction | 1,200-1,500 LOC saved | Pending |
| Test coverage | >85% config-related code | Pending |
| Documentation | Examples for each loader type | Pending |
| Zero clippy warnings | Workspace clean | Pending |

---

## Timeline Estimate

| Phase | Days | Effort |
|-------|------|--------|
| 1 | 1-2 | 12-15h |
| 2 | 2-3 | 16-20h |
| 3 | 1-2 | 8-12h |
| 4 | 1 | 6-8h |
| 5 | 1 | 4-6h |
| **Total** | **6-9** | **46-61h** |

Parallel opportunities: Phases 3, 4, 5 can overlap with Phase 2 testing.

---

**Plan Version:** 1.0
**Status:** Ready for execution
**Next Step:** Begin Phase 1
