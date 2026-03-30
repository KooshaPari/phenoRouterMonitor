# Adopting phenotype-config-core

Configuration management for Phenotype services.

## Quick Start

```rust
use phenotype_config_core::{ConfigLoader, formats::*};

// Load TOML config
let config = ConfigLoader::new()
    .with(TomlLoader::from_file("config.toml")?)
    .with(EnvLoader::with_prefix("APP")?)
    .load()?;

let value = config.get::<String>("database.url")?;
```

## Multi-Format Support

```rust
use phenotype_config_core::{ConfigLoader, formats::*};

let config = ConfigLoader::new()
    .with(TomlLoader::from_file("config.toml")?)
    .with(JsonLoader::from_file("config.json")?)
    .with(YamlLoader::from_file("config.yaml")?)
    .with(EnvLoader::with_prefix("APP")?)
    .with(ArgsLoader::from_args())
    .load()?;
```

## Environment Overrides

```rust
use phenotype_config_core::{ConfigLoader, formats::*};

let config = ConfigLoader::new()
    .with(FileLoader::<Toml>::from_file("config.toml")?)
    .with(EnvLoader::with_prefix("APP").separator("__"))
    .load()?;
```

## Custom Formats

```rust
use phenotype_config_core::{ConfigLoader, formats::*, ConfigValue};

struct CustomLoader;

impl FormatLoader for CustomLoader {
    fn name(&self) -> &str { "custom" }
    
    fn load(&self, source: &str) -> Result<ConfigValue, ConfigError> {
        // Parse custom format
        Ok(ConfigValue::from_json(...))
    }
}
```
