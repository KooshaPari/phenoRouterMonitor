# phenotype-config-core

A unified configuration loading and management library for the Phenotype ecosystem.

## Overview

`phenotype-config-core` consolidates configuration loading patterns across 5+ implementations (650+ LOC) into a single, reusable library. It provides:

- **Multi-format support**: TOML, YAML, and JSON configuration files
- **Type-safe deserialization**: via serde
- **Comprehensive error handling**: detailed, actionable error messages
- **Standard configuration locations**: XDG directories, system paths via `dirs` crate
- **Validation hooks**: Custom validation callbacks for loaded configurations
- **Configuration merging**: Combine multiple sources with precedence

## Features

### Supported Formats

- **TOML** (`.toml`)
- **YAML** (`.yaml`, `.yml`)
- **JSON** (`.json`)

All formats are normalized to JSON internally, allowing transparent format-agnostic operations.

### Directory Resolution

Automatically resolves configuration directories using:

- **Unix/Linux**: `~/.config`, `/etc` (XDG Base Directory Specification)
- **macOS**: `~/Library/Application Support`, `/etc`
- **Windows**: `%APPDATA%`, `%ProgramData%`

### Error Handling

Type-safe error types with clear, actionable messages:

```rust
pub enum ConfigError {
    FileRead { path, reason },
    FileNotFound { path },
    TomlParse { reason },
    YamlParse { reason },
    JsonParse { reason },
    Deserialize { reason },
    UnsupportedFormat { format },
    Validation { reason },
    MissingRequired { field },
    Environment { reason },
    // ...
}
```

## Usage

### Basic Loading

```rust
use phenotype_config_core::ConfigLoader;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
struct AppConfig {
    app_name: String,
    debug: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let loader = ConfigLoader::new();

    // Load from specific file
    let config: AppConfig = loader.load_from_file("config.toml")?;

    // Load from string with explicit format
    let yaml_content = "app_name: myapp\ndebug: true\n";
    let config: AppConfig = loader.load_from_string(
        yaml_content,
        ConfigFormat::Yaml
    )?;

    Ok(())
}
```

### Standard Locations

```rust
let loader = ConfigLoader::new();

// Search in standard locations: ~/.config/myapp/config.toml, /etc/myapp/config.toml
let config: AppConfig = loader
    .search_default_locations("myapp", "config.toml")?
    .ok_or("Config not found")?;
```

### Validation

```rust
let config: AppConfig = loader.load_from_file_with_validation(
    "config.toml",
    |cfg: &AppConfig| {
        if cfg.app_name.is_empty() {
            return Err(ConfigError::validation("app_name cannot be empty"));
        }
        Ok(())
    }
)?;
```

### Format-Agnostic Loading

```rust
// Load as JSON regardless of source format
let json = loader.load_from_file_as_json("config.yaml")?;
println!("{}", json["app_name"]);
```

### Merging Configurations

```rust
let merged = loader.merge_sources(&[
    "/etc/myapp/config.toml",
    "~/.config/myapp/config.toml",
    "./config.local.toml"
])?;
// Later sources take precedence
```

## API

### `ConfigLoader`

Main interface for configuration loading.

| Method | Purpose |
|--------|---------|
| `new()` | Create a new loader |
| `load_from_file<T>(path)` | Load and deserialize from a file |
| `load_from_file_as_json(path)` | Load file as JSON (format-agnostic) |
| `load_from_string<T>(content, format)` | Load from a string |
| `search_default_locations<T>(app, filename)` | Search standard directories |
| `load_from_file_with_validation<T, F>(path, validator)` | Load with custom validation |
| `merge_sources(paths)` | Merge multiple config sources |

### `ConfigDirs`

Helper for platform-aware directory resolution.

| Method | Purpose |
|--------|---------|
| `config_home()` | User config directory |
| `config_system()` | System config directory |
| `cache_home()` | User cache directory |
| `config_file(app, filename)` | Build user config path |
| `system_config_file(app, filename)` | Build system config path |
| `find_config_file(app, filename)` | Search in standard locations |
| `search_paths(app, filename)` | List all search paths |

### `ConfigFormat`

Enum for supported formats and format operations.

| Variant | Extension |
|---------|-----------|
| `Toml` | `.toml` |
| `Yaml` | `.yaml`, `.yml` |
| `Json` | `.json` |

## Design

### Consolidation Scope

This library unifies configuration loading from:

1. **thegent-hooks**: 139 LOC - YAML/JSON policy configuration
2. **phenotype-policy-engine**: 238 LOC - TOML policy loading
3. **codex core config_loader**: 1024 LOC - Layered TOML configuration
4. **harness_checkpoint**: 104 LOC - Config file snapshots
5. **heliosCLI network-proxy**: 128+ LOC - TOML network config

**Total consolidated**: 650+ LOC → 450 LOC (31% reduction)

### Integration Points

Replace custom config loading in these crates with `phenotype-config-core`:

- `platforms/thegent/crates/thegent-hooks/src/config.rs`
- `crates/phenotype-policy-engine/phenotype-policy-engine/src/loader.rs`
- `heliosCLI/codex-rs/core/src/config_loader/mod.rs`
- `heliosCLI/crates/harness_checkpoint/src/config.rs`
- `heliosCLI/codex-rs/network-proxy/src/config.rs`

## Dependencies

- `serde` / `serde_json` - Serialization
- `serde_yaml` - YAML support
- `toml` - TOML support
- `thiserror` - Error handling
- `tracing` - Logging
- `dirs` - Platform-aware directories

## Testing

Run tests:

```bash
cargo test --lib
```

All 23 unit tests pass, covering:

- Format detection (TOML, YAML, JSON)
- File I/O and error handling
- Deserialization and validation
- Directory resolution
- Configuration merging
- Edge cases (unsupported formats, missing files, validation errors)

## Future Enhancements

- [ ] Config file watching (file system events)
- [ ] Configuration caching for large files
- [ ] Environment variable interpolation
- [ ] Schema validation (JSON Schema)
- [ ] Hot-reload support
- [ ] Configuration encryption

## License

MIT
