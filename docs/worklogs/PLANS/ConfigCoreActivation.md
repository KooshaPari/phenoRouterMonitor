# Config Core Activation Plan

**Created:** 2026-03-29
**Status:** Planned
**Priority:** P0 (Critical)
**Estimated LOC Savings:** ~350 lines

## Problem Statement

3 independent config loading implementations exist, all sharing the same pattern:
1. File existence check
2. Read file contents
3. Parse to struct (TOML/YAML/JSON)
4. Apply environment variable overrides
5. Validate
6. Return or default

Meanwhile, `libs/config-core/` exists with a complete implementation but is not being used.

## Current Config Loaders

| Location | Format | Lines | Pattern |
|----------|--------|-------|---------|
| `agileplus-domain/src/config/loader.rs` | TOML | ~80 | TOML + env overrides |
| `agileplus-telemetry/src/config.rs` | YAML | ~120 | YAML + env overrides |
| `vibe-kanban/backend/src/models/config.rs` | JSON | ~150 | JSON + defaults merge |

**Total:** ~350 lines of config loading code

## Existing config-core Implementation

```rust
// libs/config-core/src/lib.rs (existing)
pub struct ConfigLoader<T: DeserializeOwned> {
    path: PathBuf,
    env_prefix: String,
    validator: Option<Box<dyn Fn(&T) -> Result<(), ConfigError>>>,
}

impl<T: DeserializeOwned + Default> ConfigLoader<T> {
    pub fn new(path: PathBuf) -> Self { ... }
    pub fn with_env_prefix(self, prefix: &str) -> Self { ... }
    pub fn with_validator(self, f: impl Fn(&T) -> Result<(), ConfigError>) -> Self { ... }
    pub fn load(&self) -> Result<T, ConfigError> { ... }
}
```

## Proposed Architecture

```
config-core/ (activated)
├── src/
│   ├── lib.rs
│   ├── traits.rs      # ConfigLoader trait
│   ├── toml.rs        # TOML backend
│   ├── yaml.rs        # YAML backend
│   ├── json.rs        # JSON backend
│   └── env.rs         # Env override helpers
└── Cargo.toml
```

## Implementation Plan

### Phase 1: Upgrade config-core (2 hours)

1. Update `edition = "2021"` → `"2024"` in `libs/config-core/Cargo.toml`
2. Add missing dependencies:
   - `serde_yaml` for YAML support
   - `serde_json` (already present)
3. Verify `cargo check` passes

### Phase 2: Extend config-core (4 hours)

```rust
// src/toml.rs
pub struct TomlLoader<T: DeserializeOwned> {
    base: ConfigLoader<T>,
}

impl<T: DeserializeOwned> TomlLoader<T> {
    pub fn new(path: PathBuf) -> Self {
        Self { base: ConfigLoader::new(path) }
    }
    pub fn load(&self) -> Result<T, ConfigError> {
        let content = std::fs::read_to_string(&self.base.path)?;
        let parsed: T = toml::from_str(&content)?;
        self.base.apply_env_overrides(&parsed)
    }
}

// src/yaml.rs
pub struct YamlLoader<T: DeserializeOwned> { ... }
pub struct JsonLoader<T: DeserializeOwned> { ... }
```

### Phase 3: Add to Workspace (1 hour)

Add `config-core` to main workspace members (if not already):

```toml
# Already in workspace - just verify
members = [
  ...
  "libs/config-core",
  ...
]
```

### Phase 4: Integrate with Crates (6 hours)

**agileplus-domain:**
```rust
// Before
use crate::config::loader::ConfigLoader;
let config = ConfigLoader::new().load()?;

// After
use config_core::TomlLoader;
let config = TomlLoader::new(home_dir() / ".agileplus/config.toml")
    .with_env_prefix("AGILEPLUS")
    .load()?;
```

**agileplus-telemetry:**
```rust
// Before
use crate::config::TelemetryConfig;
let config = TelemetryConfig::load()?;

// After
use config_core::YamlLoader;
let config = YamlLoader::new(home_dir() / ".agileplus/otel-config.yaml")
    .with_env_prefix("OTEL")
    .load()?;
```

**vibe-kanban:**
```rust
// Before
use crate::models::config::Config;
let config = Config::load()?;

// After
use config_core::JsonLoader;
let config = JsonLoader::new(config_path)
    .with_defaults(Config::default())
    .load()?;
```

### Phase 5: Remove Duplicated Code (2 hours)

- Delete `agileplus-domain/src/config/loader.rs`
- Remove config loading from `agileplus-telemetry/src/config.rs`
- Remove config loading from `vibe-kanban/backend/src/models/config.rs`

## Effort Estimate

| Phase | Effort | Risk |
|-------|--------|------|
| Upgrade config-core edition | 2 hours | Low |
| Extend with TOML/YAML/JSON backends | 4 hours | Low |
| Add to workspace | 1 hour | Low |
| Integrate with 3 crates | 6 hours | Medium |
| Remove duplicated code | 2 hours | Low |
| **Total** | **15 hours** | — |

## LOC Impact

| Metric | Before | After | Savings |
|--------|--------|-------|---------|
| Config loading LOC | 350 | ~100 | **250 lines (71%)** |
| config-core LOC | 0 (unused) | ~200 | 200 lines |
| **Net Savings** | — | — | **~150 lines** |

## Action Items

- [ ] 🔴 **CRITICAL** Update `libs/config-core/Cargo.toml` edition to 2024
- [ ] 🟡 **HIGH** Add `serde_yaml` dependency to config-core
- [ ] 🟡 **HIGH** Implement `YamlLoader` backend in config-core
- [ ] 🟡 **HIGH** Implement `JsonLoader` backend in config-core
- [ ] 🟠 **MEDIUM** Migrate `agileplus-domain` to config-core
- [ ] 🟠 **MEDIUM** Migrate `agileplus-telemetry` to config-core
- [ ] 🟠 **MEDIUM** Migrate `vibe-kanban` to config-core
- [ ] 🟢 **LOW** Delete duplicated loader code
- [ ] 🟢 **LOW** Update worklogs/ARCHITECTURE.md

## Related

- worklogs/ARCHITECTURE.md (libs/ Directory Analysis)
- worklogs/DUPLICATION.md (Config Loading Duplication)
