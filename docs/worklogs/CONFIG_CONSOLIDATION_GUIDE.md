# Phenotype Config Loading Consolidation Guide

## Overview

This document describes the consolidation of configuration loading functionality across the Phenotype workspace into the unified `phenotype-config-core` crate.

## Problem Statement

The LOC audit (2026-03-29) identified 1,200+ LOC of scattered configuration loading patterns across the workspace:

| Project | Pattern | LOC |
|---------|---------|-----|
| heliosCLI/codex-rs/core/src/config/ | Multi-file config system | ~400 |
| platforms/thegent/crates/thegent-hooks/src/config.rs | Custom YAML/JSON loader | ~200 |
| heliosCLI/codex-rs/otel/src/config.rs | Telemetry config | ~150 |
| heliosCLI/codex-rs/network-proxy/src/config.rs | Network config | ~100 |
| Other scattered patterns | Various | ~350 |
| **Total** | | **~1,200 LOC** |

Each implementation varied in:
- Supported formats (JSON-only vs. multi-format)
- Error handling patterns
- Type conversion approaches
- Environment variable handling

## Solution: Enhanced phenotype-config-core

### Architecture

```
ConfigBuilder (fluent API)
    ├── with_file() ─→ FileLoader
    ├── with_env_prefix() ─→ EnvLoader
    └── with_inline_value() ─→ InlineLoader
         ↓
    Override Chain Merging
    (env > file > defaults)
         ↓
    Config (type-safe access)
         ↓
    ConfigValidator (optional)
```

### Key Features

#### 1. Multi-Format Support
```rust
// Auto-detect from extension
let config = ConfigBuilder::new()
    .with_file("config.json")
    .build()?;

// Or explicit format
let config = ConfigBuilder::new()
    .with_file_format("config.txt", ConfigFormat::Json)
    .build()?;
```

Supported formats:
- JSON (serde_json)
- TOML (toml)
- YAML (serde_yaml)

#### 2. Override Chain Pattern
Later sources override earlier ones:

```rust
let config = ConfigBuilder::new()
    .with_inline_value("port", json!(3000))        // Default
    .with_file("config.toml")                       // Override
    .with_env_prefix("APP_")                        // Override again
    .build()?;

// Priority: environment > file > defaults
```

This allows safe production deployments:
- Code defaults ensure app starts
- Config files override for environments
- Environment variables override for secrets/ops

#### 3. Type-Safe Access

```rust
// Optional access
let port = config.get_i64("port").unwrap_or(3000);
let debug = config.get_bool("debug").unwrap_or(false);
let name = config.get_string("app_name").unwrap_or("default");

// Required access (fails if missing or wrong type)
let db_url = config.get_string_required("database_url")?;
let port = config.get_i64_required("port")?;
let enabled = config.get_bool_required("feature_enabled")?;

// Generic access
let value = config.get_value("custom_field");

// Nested sections
let db_config = config.get_section("database")?;
let host = db_config.get_string("host")?;
```

#### 4. Configuration Validation

```rust
use phenotype_config_core::validator::ValueType;

let validator = ConfigValidator::new()
    .require_keys(vec!["port", "database_url"])
    .require_type("port", ValueType::Number)
    .require_type("enabled", ValueType::Boolean);

validator.validate(&config.to_json_value())?;
```

#### 5. Environment Variable Loading

```rust
// Load all env vars prefixed with "APP_"
let config = ConfigBuilder::new()
    .with_env_prefix("APP_")
    .build()?;

// Then access by key (without prefix, lowercased)
let port = config.get_string("port"); // from APP_PORT

// Combine with files for proper override order
let config = ConfigBuilder::new()
    .with_inline_value("port", json!(3000))    // defaults
    .with_file("config.toml")                   // dev/staging override
    .with_env_prefix("APP_")                    // production override
    .build()?;
```

## Migration Guide

### Phase 1: Dependency Addition

Add to `Cargo.toml`:
```toml
[dependencies]
phenotype-config-core = { path = "../../crates/phenotype-config-core" }
serde_json = "1.0"  # For json! macro
```

### Phase 2: Replace Loader

**Before:**
```rust
let config = ConfigLoader::load("config.toml")?;
let port = config.port.unwrap_or(3000);
```

**After:**
```rust
let config = ConfigBuilder::new()
    .with_file("config.toml")
    .build()?;
let port = config.get_i64("port").unwrap_or(3000);
```

### Phase 3: Add Environment Variable Support

**Before:**
```rust
// Manual env var handling scattered throughout code
let port = std::env::var("PORT")
    .ok()
    .and_then(|p| p.parse::<i64>().ok())
    .unwrap_or(3000);
```

**After:**
```rust
let config = ConfigBuilder::new()
    .with_file("config.toml")
    .with_env_prefix("APP_")
    .build()?;
let port = config.get_i64("port").unwrap_or(3000);
```

### Phase 4: Add Validation (Optional)

```rust
use phenotype_config_core::ConfigValidator;
use phenotype_config_core::validator::ValueType;

let validator = ConfigValidator::new()
    .require_keys(vec!["database_url", "port", "secret_key"])
    .require_type("port", ValueType::Number);

validator.validate(&config.to_json_value())?;
```

## Example: Complete Migration

### heliosCLI Network Proxy Config

**Before:** `heliosCLI/codex-rs/network-proxy/src/config.rs`

```rust
pub struct ProxyConfig {
    pub listen_port: u16,
    pub target_host: String,
    pub tls_enabled: bool,
}

impl ProxyConfig {
    pub fn load() -> Result<Self> {
        let config_file = "proxy.toml";
        let content = std::fs::read_to_string(config_file)?;
        let table: toml::Table = toml::from_str(&content)?;
        
        Ok(ProxyConfig {
            listen_port: table.get("listen_port")
                .and_then(|v| v.as_integer())
                .ok_or(ConfigError::MissingKey("listen_port"))?
                as u16,
            target_host: table.get("target_host")
                .and_then(|v| v.as_str())
                .ok_or(ConfigError::MissingKey("target_host"))?
                .to_string(),
            tls_enabled: table.get("tls_enabled")
                .and_then(|v| v.as_bool())
                .unwrap_or(false),
        })
    }
}
```

**After:**

```rust
use phenotype_config_core::{Config, ConfigBuilder, ConfigValidator};
use phenotype_config_core::validator::ValueType;

pub struct ProxyConfig(Config);

impl ProxyConfig {
    pub fn load() -> phenotype_config_core::Result<Self> {
        let config = ConfigBuilder::new()
            .with_file("proxy.toml")
            .with_env_prefix("PROXY_")
            .build()?;
        
        // Validate required fields
        ConfigValidator::new()
            .require_keys(vec!["listen_port", "target_host"])
            .require_type("listen_port", ValueType::Number)
            .require_type("target_host", ValueType::String)
            .require_type("tls_enabled", ValueType::Boolean)
            .validate(&config.to_json_value())?;
        
        Ok(ProxyConfig(config))
    }
    
    pub fn listen_port(&self) -> u16 {
        self.0.get_i64("listen_port")
            .unwrap_or(3000) as u16
    }
    
    pub fn target_host(&self) -> String {
        self.0.get_string("target_host")
            .unwrap_or_default()
    }
    
    pub fn tls_enabled(&self) -> bool {
        self.0.get_bool("tls_enabled")
            .unwrap_or(false)
    }
}
```

**Savings:** 50-70 LOC removed, better error handling, env var support added

## Testing

All functionality is covered by 18 integration tests:

```bash
# Run all tests
cargo test -p phenotype-config-core

# Run specific test
cargo test -p phenotype-config-core test_file_loader_json

# Run with output
cargo test -p phenotype-config-core -- --nocapture
```

## Project-Specific Migration Priority

### High Priority (Ready Now)
1. **heliosCLI/codex-rs/core/src/config/** - Complex, high LOC impact
2. **platforms/thegent/crates/thegent-hooks** - New format support needed

### Medium Priority (Next Sprint)
3. **heliosCLI/codex-rs/otel/src/config.rs** - Telemetry config
4. **heliosCLI/codex-rs/network-proxy/** - Network proxying

### Low Priority (Future)
5. **agileplus** - When domain models mature
6. **phench** - Python config patterns (may need adapter layer)

## API Reference

### ConfigBuilder Methods

```rust
ConfigBuilder::new()
    .with_file(path)                          // Auto-detect format
    .with_file_format(path, format)           // Explicit format
    .with_env()                               // All env vars
    .with_env_prefix(prefix)                  // Prefixed env vars
    .with_inline_value(key, value)            // Defaults
    .build()? -> Config
```

### Config Methods

```rust
config.get_string(key) -> Option<String>
config.get_string_required(key) -> Result<String>
config.get_i64(key) -> Option<i64>
config.get_i64_required(key) -> Result<i64>
config.get_bool(key) -> Option<bool>
config.get_bool_required(key) -> Result<bool>
config.get_value(key) -> Option<&Value>
config.get_section(key) -> Option<Config>
config.set_value(key, value) -> ()
config.contains_key(key) -> bool
config.len() -> usize
config.is_empty() -> bool
config.keys() -> Iterator<&str>
config.to_json_value() -> Value
```

### ConfigValidator Methods

```rust
ConfigValidator::new()
    .require_key(key)
    .require_keys(keys)
    .require_type(key, type)
    .validate(value) -> Result<()>
```

## FAQ

**Q: What if my app is already using a custom config system?**
A: Use a gradual migration approach. Start with new modules using phenotype-config-core, gradually refactor existing ones.

**Q: Can I use this with async code?**
A: Yes, all loaders are sync but the returned Config is thread-safe and can be wrapped in Arc<Config> for async contexts.

**Q: What about complex nested config?**
A: Use `get_section()` to get nested Config objects, or `get_value()` to access raw JSON for custom deserialization.

**Q: How do I handle secrets in config?**
A: Use environment variables with the override chain - keep secrets in env, non-sensitive config in files.

**Q: Can I extend the loader types?**
A: Yes, the ConfigLoaderFn trait is internal, but you can create wrapper functions that return Result<Value> and add them via ConfigBuilder.

## Support & Issues

For bugs or enhancements:
1. Check `crates/phenotype-config-core/README.md`
2. See test examples in `crates/phenotype-config-core/tests/`
3. Review `docs/worklogs/CONFIG_CONSOLIDATION_REPORT.md`

## Related Documents

- `docs/worklogs/CONFIG_CONSOLIDATION_REPORT.md` - Detailed implementation report
- `crates/phenotype-config-core/Cargo.toml` - Dependencies and versioning
- `crates/phenotype-config-core/src/lib.rs` - Core module documentation
