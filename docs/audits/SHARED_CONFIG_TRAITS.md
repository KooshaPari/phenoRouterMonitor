# Phenotype Config Core: Shared Trait Definitions

**Version:** 2.0  
**Status:** Proposed Implementation  
**Target Crates:** phenotype-config-core, phenotype-config-loader, phenotype-policy-engine, phenotype-telemetry  

---

## Overview

This document defines the portable trait interfaces for configuration loading, validation, and provision across all Phenotype crates. These traits are language-agnostic and can be implemented for Go, Python, and other language ecosystems.

---

## Trait Hierarchy

```
ConfigLoader (async variant)
  ├─ Generic over Config type
  └─ Returns Result<Config, ConfigError>

ConfigLoaderSync (blocking variant)
  ├─ For non-async contexts
  └─ Returns Result<Config, ConfigError>

ConfigSource (abstraction for sources)
  ├─ Environment variables
  ├─ File-based (TOML, JSON, YAML)
  └─ In-memory/defaults

ConfigValidator (validation after load)
  ├─ Type-specific validation logic
  └─ Returns Result<(), ConfigError>

ConfigProvider (DI pattern)
  ├─ Holds loaded config
  └─ Provides read access

ConfigError (unified error type)
  └─ All config failures unified
```

---

## Core Traits

### 1. ConfigError (Unified Error Type)

**Pseudocode:**

```rust
#[derive(Debug, Clone)]
pub enum ConfigError {
    // Parse errors
    Io(String),                          // File I/O, network
    Parse(String),                        // TOML, JSON, YAML parsing
    Serialization(String),               // Serde errors
    
    // Validation errors
    Validation(String),                  // User-defined validation failed
    InvalidConfiguration(String),        // Config structure invalid
    InvalidValue { field: String, reason: String },
    
    // Lookup errors
    NotFound(String),                    // File not found, key not found
    KeyNotFound { key: String, source: String },
    
    // Internal errors
    Other(String),                       // Catch-all
}

impl ConfigError {
    /// Create an IO error
    pub fn io(msg: impl Into<String>) -> Self { ... }
    
    /// Create a parse error (TOML, JSON, YAML)
    pub fn parse(msg: impl Into<String>) -> Self { ... }
    
    /// Create a validation error
    pub fn validation(msg: impl Into<String>) -> Self { ... }
    
    /// Create a "not found" error
    pub fn not_found(key: impl Into<String>) -> Self { ... }
    
    /// Create an invalid value error
    pub fn invalid_value(field: impl Into<String>, reason: impl Into<String>) -> Self { ... }
}

// Conversions from standard error types
impl From<std::io::Error> for ConfigError { ... }
impl From<toml::de::Error> for ConfigError { ... }
impl From<serde_json::Error> for ConfigError { ... }
```

**Language Variants:**

| Language | Error Type | Implementation |
|----------|------------|-----------------|
| Rust | Enum with thiserror | `#[derive(Error)]` with variants |
| Go | Interface-based | `type ConfigError interface { Error() string }` |
| Python | Exception class | `class ConfigError(Exception)` |

**Usage Examples:**

```rust
// Rust
return Err(ConfigError::validation("timeout must be > 0"));
return Err(ConfigError::not_found("config.toml"));

// Go
return fmt.Errorf("validation error: timeout must be > 0")

// Python
raise ConfigError("Validation error: timeout must be > 0")
```

---

### 2. ConfigLoader (Async Trait)

**Purpose:** Load and deserialize configuration from sources

**Pseudocode:**

```rust
#[async_trait]
pub trait ConfigLoader: Send + Sync + 'static {
    /// The type of configuration this loader produces
    type Config: Debug + Send + Sync + serde::de::DeserializeOwned + 'static;
    
    /// Load configuration asynchronously
    /// 
    /// Implementers should:
    /// 1. Read from all configured sources
    /// 2. Merge according to priority (env > file > default)
    /// 3. Deserialize into Self::Config
    /// 4. Return error if critical source fails
    async fn load(&self) -> Result<Self::Config, ConfigError>;
    
    /// Optional: Reload configuration (for hot-reload support)
    async fn reload(&self) -> Result<Self::Config, ConfigError> {
        self.load().await
    }
}
```

**Implementation Notes:**

- Async required for I/O-heavy operations (file reads, network)
- Type parameter allows type-safe config structs
- No assumptions about source types (file, env, secrets, etc.)
- Implementers should document source priority order

**Example Implementations:**

```rust
// Figment-based loader (phenotype-config-loader)
#[async_trait]
impl<T: DeserializeOwned + Send + 'static> ConfigLoader for FigmentConfigLoader {
    type Config = T;
    
    async fn load(&self) -> Result<T, ConfigError> {
        let figment = self.build_figment();
        let config = tokio::task::spawn_blocking(move || {
            figment.extract::<T>()
        })
        .await
        .map_err(|e| ConfigError::Other(e.to_string()))??;
        Ok(config)
    }
}

// Policy loader (phenotype-policy-engine)
#[async_trait]
impl ConfigLoader for PolicyConfigLoader {
    type Config = Vec<Policy>;
    
    async fn load(&self) -> Result<Vec<Policy>, ConfigError> {
        let policies_config = tokio::fs::read_to_string(&self.path)
            .await
            .map_err(|e| ConfigError::io(e.to_string()))?;
        
        let file: PoliciesConfigFile = toml::from_str(&policies_config)
            .map_err(|e| ConfigError::parse(e.to_string()))?;
        
        file.to_policies()
    }
}

// Telemetry loader (phenotype-telemetry)
#[async_trait]
impl ConfigLoader for TelemetryConfigLoader {
    type Config = TelemetryConfig;
    
    async fn load(&self) -> Result<TelemetryConfig, ConfigError> {
        let service_name = std::env::var("SERVICE_NAME")
            .map_err(|_| ConfigError::not_found("SERVICE_NAME env var"))?;
        let environment = std::env::var("ENVIRONMENT")
            .unwrap_or_else(|_| "production".to_string());
        
        Ok(TelemetryConfig { service_name, environment })
    }
}
```

---

### 3. ConfigLoaderSync (Blocking Trait)

**Purpose:** Load configuration in blocking/sync contexts

**Pseudocode:**

```rust
pub trait ConfigLoaderSync: Send + Sync + 'static {
    /// The type of configuration this loader produces
    type Config: Debug + Send + Sync + serde::de::DeserializeOwned + 'static;
    
    /// Load configuration synchronously
    /// 
    /// For use in:
    /// - main() initialization
    /// - Sync library contexts
    /// - Testing
    fn load(&self) -> Result<Self::Config, ConfigError>;
    
    /// Load from a specific file path
    fn load_from_file(path: &Path) -> Result<Self::Config, ConfigError>
    where
        Self::Config: serde::de::DeserializeOwned,
    {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ConfigError::io(e.to_string()))?;
        // Delegate to implementation-specific parsing
        Self::parse(&content)
    }
    
    /// Load from a string (TOML, JSON, etc.)
    fn parse(input: &str) -> Result<Self::Config, ConfigError>;
}
```

**Companion Helper:**

```rust
/// Async wrapper for sync loaders
pub struct AsyncWrapper<L: ConfigLoaderSync> {
    inner: L,
}

#[async_trait]
impl<L: ConfigLoaderSync> ConfigLoader for AsyncWrapper<L> {
    type Config = L::Config;
    
    async fn load(&self) -> Result<L::Config, ConfigError> {
        tokio::task::spawn_blocking({
            let inner = &self.inner;
            move || inner.load()
        })
        .await
        .map_err(|e| ConfigError::Other(e.to_string()))?
    }
}
```

---

### 4. ConfigSource (Abstraction for Config Sources)

**Purpose:** Pluggable source abstraction (env vars, files, secrets, etc.)

**Pseudocode:**

```rust
pub trait ConfigSource: Send + Sync + 'static {
    /// Get a configuration value by key
    fn get(&self, key: &str) -> Option<Value>;
    
    /// Check if a key exists
    fn contains_key(&self, key: &str) -> bool {
        self.get(key).is_some()
    }
    
    /// Get all keys (for debugging)
    fn keys(&self) -> Vec<String>;
    
    /// Get the source name (for logging/debugging)
    fn source_name(&self) -> &str;
}

// Standard implementations
pub struct EnvConfigSource {
    prefix: Option<String>,
}

impl ConfigSource for EnvConfigSource {
    fn get(&self, key: &str) -> Option<Value> {
        let env_key = self.prefix.as_ref()
            .map(|p| format!("{}_{}", p, key))
            .unwrap_or_else(|| key.to_string());
        std::env::var(&env_key).ok().map(Value::String)
    }
    
    fn source_name(&self) -> &str {
        "environment variables"
    }
}

pub struct FileConfigSource {
    path: PathBuf,
    content: String,
}

impl ConfigSource for FileConfigSource {
    fn get(&self, key: &str) -> Option<Value> {
        // Parse TOML/JSON and extract nested key
        // e.g., "database.url" -> data["database"]["url"]
    }
    
    fn source_name(&self) -> &str {
        self.path.to_string_lossy().as_ref()
    }
}

pub struct SecretConfigSource {
    // Could integrate with HashiCorp Vault, AWS Secrets, etc.
}

pub struct CachedConfigSource<S: ConfigSource> {
    inner: S,
    cache: Arc<DashMap<String, Value>>,
}
```

**Composable Builder:**

```rust
pub struct ConfigSourceBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
}

impl ConfigSourceBuilder {
    pub fn with_env_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.sources.push(Box::new(EnvConfigSource {
            prefix: Some(prefix.into()),
        }));
        self
    }
    
    pub fn with_file(mut self, path: impl AsRef<Path>) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| ConfigError::io(e.to_string()))?;
        self.sources.push(Box::new(FileConfigSource {
            path: path.as_ref().to_path_buf(),
            content,
        }));
        Ok(self)
    }
    
    pub fn with_defaults(mut self, defaults: serde_json::Value) -> Self {
        self.sources.push(Box::new(DefaultsConfigSource { defaults }));
        self
    }
    
    pub fn build(self) -> impl ConfigSource {
        CascadingConfigSource { sources: self.sources }
    }
}
```

---

### 5. ConfigValidator (Post-Load Validation)

**Purpose:** Validate configuration after deserialization

**Pseudocode:**

```rust
pub trait ConfigValidator: Send + Sync + 'static {
    /// The configuration type this validator validates
    type Config: Debug + Send + Sync + 'static;
    
    /// Validate the configuration
    /// 
    /// Called after deserialization to check:
    /// - Required fields are present
    /// - Value ranges (timeout > 0, port in 1-65535, etc.)
    /// - Dependencies (if A is set, B must be set)
    /// - Custom domain rules
    fn validate(&self, config: &Self::Config) -> Result<(), ConfigError>;
}

// Example validators
pub struct DatabaseConfigValidator;

impl ConfigValidator for DatabaseConfigValidator {
    type Config = DatabaseConfig;
    
    fn validate(&self, config: &DatabaseConfig) -> Result<(), ConfigError> {
        if config.url.is_empty() {
            return Err(ConfigError::invalid_value("url", "cannot be empty"));
        }
        if config.pool_size == 0 || config.pool_size > 1000 {
            return Err(ConfigError::invalid_value("pool_size", "must be 1-1000"));
        }
        if config.timeout_secs == 0 {
            return Err(ConfigError::invalid_value("timeout_secs", "must be > 0"));
        }
        Ok(())
    }
}

pub struct CacheConfigValidator;

impl ConfigValidator for CacheConfigValidator {
    type Config = CacheConfig;
    
    fn validate(&self, config: &CacheConfig) -> Result<(), ConfigError> {
        if config.ttl_secs == 0 {
            return Err(ConfigError::invalid_value("ttl_secs", "must be > 0"));
        }
        if config.max_entries == 0 {
            return Err(ConfigError::invalid_value("max_entries", "must be > 0"));
        }
        Ok(())
    }
}

pub struct ChainValidator<V1, V2>(V1, V2);

impl<C, V1, V2> ConfigValidator for ChainValidator<V1, V2>
where
    C: Debug + Send + Sync,
    V1: ConfigValidator<Config = C>,
    V2: ConfigValidator<Config = C>,
{
    type Config = C;
    
    fn validate(&self, config: &C) -> Result<(), ConfigError> {
        self.0.validate(config)?;
        self.1.validate(config)?;
        Ok(())
    }
}
```

---

### 6. ConfigProvider (Dependency Injection Pattern)

**Purpose:** Hold and provide access to loaded config

**Pseudocode:**

```rust
pub trait ConfigProvider: Send + Sync + 'static {
    /// The configuration type provided
    type Config: Debug + Send + Sync + 'static;
    
    /// Get a reference to the configuration
    fn config(&self) -> &Self::Config;
}

// Standard implementation
pub struct DefaultConfigProvider<C: Debug + Send + Sync + 'static> {
    config: Arc<C>,
}

impl<C: Debug + Send + Sync + 'static> DefaultConfigProvider<C> {
    pub fn new(config: C) -> Self {
        Self {
            config: Arc::new(config),
        }
    }
}

impl<C: Debug + Send + Sync + 'static> ConfigProvider for DefaultConfigProvider<C> {
    type Config = C;
    
    fn config(&self) -> &C {
        &self.config
    }
}

// Mutable variant for hot-reload (rare)
pub struct MutableConfigProvider<C: Debug + Send + Sync + 'static> {
    config: Arc<RwLock<C>>,
}

impl<C: Debug + Send + Sync + 'static> MutableConfigProvider<C> {
    pub fn update(&self, new_config: C) {
        *self.config.write().unwrap() = new_config;
    }
}

impl<C: Debug + Send + Sync + 'static> ConfigProvider for MutableConfigProvider<C> {
    type Config = C;
    
    fn config(&self) -> &C {
        // Note: Returns reference into RwLock, tricky in practice
        // Better approach: clone config or use Arc<Config>
    }
}
```

---

## Cross-Language Trait Design

### Rust

```rust
#[async_trait]
pub trait ConfigLoader {
    type Config: serde::de::DeserializeOwned;
    async fn load(&self) -> Result<Self::Config, ConfigError>;
}
```

### Go

```go
type ConfigLoader interface {
    Load(ctx context.Context) (interface{}, error)
}

type TypedConfigLoader[T any] interface {
    Load(ctx context.Context) (*T, error)
}
```

### Python

```python
from abc import ABC, abstractmethod
from typing import Generic, TypeVar

T = TypeVar('T')

class ConfigLoader(ABC, Generic[T]):
    @abstractmethod
    async def load(self) -> T:
        pass
```

---

## Integration Patterns

### Pattern 1: Load + Validate

```rust
let loader = FigmentConfigLoader::new()
    .with_env_prefix("APP");

let config: AppConfig = loader.load().await?;

let validator = AppConfigValidator;
validator.validate(&config)?;

let provider = DefaultConfigProvider::new(config);
```

### Pattern 2: Cascading Sources

```rust
let config = ConfigSourceBuilder::new()
    .with_file("config.toml")?
    .with_file(format!("config.{}.toml", std::env::var("ENV")?)).ok()
    .with_env_prefix("APP")
    .with_defaults(defaults_json)
    .build();

let app_config: AppConfig = config_deserialize(&config)?;
```

### Pattern 3: Hot Reload

```rust
let provider = MutableConfigProvider::new(initial_config);

// In background task:
tokio::spawn({
    let provider = provider.clone();
    async move {
        loop {
            if let Ok(new_config) = loader.load().await {
                provider.update(new_config);
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
        }
    }
});
```

### Pattern 4: Dependency Injection

```rust
#[derive(Clone)]
struct AppContext {
    config_provider: Arc<dyn ConfigProvider<Config = AppConfig>>,
}

impl AppContext {
    async fn new(loader: &dyn ConfigLoader<Config = AppConfig>) -> Result<Self> {
        let config = loader.load().await?;
        Ok(Self {
            config_provider: Arc::new(DefaultConfigProvider::new(config)),
        })
    }
}
```

---

## Error Handling Matrix

| Scenario | Error Type | Recovery |
|----------|------------|----------|
| File not found | `ConfigError::NotFound` | Use defaults, exit |
| Parse error | `ConfigError::Parse` | Log, retry with backup file |
| Validation fails | `ConfigError::Validation` | Fail fast, log details |
| Env var missing | `ConfigError::KeyNotFound` | Use default if optional |
| IO error on read | `ConfigError::Io` | Retry with backoff |
| Serialization error | `ConfigError::Serialization` | Log, check data types |

---

## Testing Utilities

### Mock Config Source

```rust
pub struct MockConfigSource {
    data: HashMap<String, Value>,
}

impl ConfigSource for MockConfigSource {
    fn get(&self, key: &str) -> Option<Value> {
        self.data.get(key).cloned()
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_validation_fails_on_invalid_timeout() {
        let config = DatabaseConfig {
            url: "postgresql://localhost".into(),
            pool_size: 10,
            timeout_secs: 0, // Invalid
        };
        
        let validator = DatabaseConfigValidator;
        let result = validator.validate(&config);
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ConfigError::InvalidValue { .. }));
    }
}
```

---

## Documentation Examples

### Example 1: Loading from File with Env Override

```rust
let config: AppConfig = FigmentConfigLoader::new()
    .with_config_name("myapp")
    .with_env_prefix("MYAPP")
    .load()
    .await?;
```

### Example 2: Validating Loaded Config

```rust
let validator = ChainValidator(
    DatabaseConfigValidator,
    CacheConfigValidator,
);

if let Err(e) = validator.validate(&config) {
    eprintln!("Configuration invalid: {}", e);
    std::process::exit(1);
}
```

### Example 3: Dependency Injection in Axum

```rust
#[derive(Clone)]
struct AppState {
    config_provider: Arc<dyn ConfigProvider<Config = AppConfig>>,
}

#[tokio::main]
async fn main() -> Result<()> {
    let loader = FigmentConfigLoader::new().with_env_prefix("APP");
    let config = loader.load().await?;
    
    let state = AppState {
        config_provider: Arc::new(DefaultConfigProvider::new(config)),
    };
    
    let app = Router::new()
        .route("/", get(handler))
        .with_state(state);
    
    // ...
    Ok(())
}

async fn handler(
    State(state): State<AppState>,
) -> impl IntoResponse {
    let config = state.config_provider.config();
    // Use config
}
```

---

## Summary Table

| Trait | Purpose | When to Implement | Example |
|-------|---------|------------------|---------|
| **ConfigLoader** | Load from sources | Every config type | FigmentConfigLoader |
| **ConfigLoaderSync** | Sync loading | CLI/main startup | FileConfigLoaderSync |
| **ConfigSource** | Abstract source | New source type | VaultConfigSource |
| **ConfigValidator** | Validate values | Each config domain | DatabaseConfigValidator |
| **ConfigProvider** | Hold & provide config | DI/service layer | DefaultConfigProvider |
| **ConfigError** | Unified errors | All config failures | Use variants |

---

## References

- ADR-001: Config Architecture Decision Record
- phenotype-config-core repository
- Rust async_trait macro: https://docs.rs/async-trait
- Serde trait bounds: https://serde.rs/
- Figment documentation: https://docs.rs/figment

---

**Document Version:** 1.0  
**Last Updated:** 2026-03-30  
**Maintainer:** Phenotype Architecture Team
