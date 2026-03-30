# Phenotype Federation Pattern: Trait-Based Plugin System Sketch

**Scope:** Detailed design for converting 24/28 crates into pluggable, feature-flagged components  
**Goal:** Enable opt-in compilation, runtime discovery, and swappable implementations  
**Timeline:** 2-4 weeks (Phase 2 continuation)

---

## 1. Architecture Overview

### Current State: Monolithic Dependencies

```
Application Binary
├─ All 28 crates linked (no opt-out)
├─ All features compiled in
├─ Binary size: large (estimated 50-100MB)
└─ Startup time: slow (all modules init)
```

### Target State: Modular Federation

```
Application Binary (Slim Core)
├─ phenotype-port-traits (0.5KB) — Pure trait definitions
├─ phenotype-error-core (2KB) — Foundation error types
├─ phenotype-config-core (1KB) — Foundation config interface
│
└─ Plugin Registry (feature-gated)
   ├─ [feat=config] phenotype-config-loader
   ├─ [feat=errors] phenotype-errors
   ├─ [feat=events] phenotype-event-sourcing
   ├─ [feat=policy] phenotype-policy-engine
   ├─ [feat=health] phenotype-health
   ├─ [feat=crypto] phenotype-crypto
   ├─ [feat=git] phenotype-git-core
   ├─ [feat=cache] phenotype-cache-adapter
   ├─ [feat=http] phenotype-http-client-core
   └─ ... (20+ more optional plugins)

Binary Size Impact:
  • Slim (core only): ~5MB
  • Medium (6 features): ~15MB
  • Full (all 24): ~50-100MB
  • Savings: 60-80% for typical deployments
```

---

## 2. Port-Traits Layer (Central Interface Hub)

### File Structure

```
crates/phenotype-port-traits/
├── src/
│   ├── lib.rs                    — Re-exports all traits
│   ├── plugin.rs                 — Plugin trait + registry
│   ├── config/
│   │   └── provider.rs           — ConfigProvider trait
│   ├── error/
│   │   ├── handler.rs            — ErrorHandler trait
│   │   └── classification.rs     — Error classification interface
│   ├── event/
│   │   ├── store.rs              — EventStore trait
│   │   └── sourcing.rs           — EventSourcing interface
│   ├── policy/
│   │   ├── engine.rs             — PolicyEngine trait
│   │   └── evaluation.rs         — Policy evaluation interface
│   ├── health/
│   │   ├── checker.rs            — HealthChecker trait
│   │   └── status.rs             — Health status interface
│   ├── crypto/
│   │   └── provider.rs           — CryptoProvider trait
│   ├── git/
│   │   └── provider.rs           — GitProvider trait
│   ├── cache/
│   │   └── store.rs              — CacheStore trait
│   ├── http/
│   │   └── client.rs             — HttpClient trait
│   └── common/
│       ├── result.rs             — Common error type
│       └── context.rs            — Execution context
├── Cargo.toml                    — Minimal deps (serde, async-trait only)
└── README.md                     — Plugin interface guide

Dependency Profile:
  • Depends on: serde, serde_json, async-trait, thiserror (external only)
  • Depended on by: All 24 plugins + applications
  • Size: ~5KB source, ~100KB compiled
```

### Core Trait Definitions (Detailed)

```rust
// phenotype-port-traits/src/plugin.rs

use std::any::Any;
use serde::{Serialize, Deserialize};

/// Base trait for all plugins
pub trait Plugin: Send + Sync {
    /// Plugin identifier (e.g., "config-loader", "event-sourcing")
    fn id(&self) -> &'static str;
    
    /// Semantic version (e.g., "1.0.0")
    fn version(&self) -> &'static str;
    
    /// Human-readable description
    fn description(&self) -> &'static str;
    
    /// Plugin capabilities (for discovery)
    fn capabilities(&self) -> Vec<Capability>;
    
    /// Initialize plugin (called once at startup)
    fn initialize(&self) -> Result<(), PluginError>;
    
    /// Shutdown plugin (called on app termination)
    fn shutdown(&self) -> Result<(), PluginError>;
    
    /// For downcasting to specific plugin type
    fn as_any(&self) -> &(dyn Any + Send + Sync);
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Capability {
    pub name: String,
    pub version: String,
}

/// Plugin error type
#[derive(Debug, thiserror::Error)]
pub enum PluginError {
    #[error("Plugin initialization failed: {0}")]
    InitializationFailed(String),
    
    #[error("Plugin not found: {0}")]
    NotFound(String),
    
    #[error("Plugin incompatible: {0}")]
    Incompatible(String),
    
    #[error("Plugin error: {0}")]
    Other(String),
}

pub type PluginResult<T> = Result<T, PluginError>;

/// Runtime plugin registry
pub struct PluginRegistry {
    plugins: std::collections::HashMap<String, std::sync::Arc<dyn Plugin>>,
}

impl PluginRegistry {
    pub fn new() -> Self {
        Self {
            plugins: std::collections::HashMap::new(),
        }
    }
    
    pub fn register<P: Plugin + 'static>(&mut self, plugin: P) -> PluginResult<()> {
        let id = plugin.id().to_string();
        if self.plugins.contains_key(&id) {
            return Err(PluginError::Other(format!("Plugin {} already registered", id)));
        }
        self.plugins.insert(id, std::sync::Arc::new(plugin));
        Ok(())
    }
    
    pub fn get(&self, id: &str) -> Option<std::sync::Arc<dyn Plugin>> {
        self.plugins.get(id).cloned()
    }
    
    pub fn list(&self) -> Vec<String> {
        self.plugins.keys().cloned().collect()
    }
    
    pub fn unregister(&mut self, id: &str) -> PluginResult<()> {
        self.plugins.remove(id)
            .ok_or_else(|| PluginError::NotFound(id.to_string()))?;
        Ok(())
    }
}

impl Default for PluginRegistry {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Config Plugin Trait
// ============================================================================

pub trait ConfigProvider: Plugin {
    /// Load configuration from source
    async fn load(&self) -> PluginResult<ConfigMap>;
    
    /// Validate configuration
    fn validate(&self, config: &ConfigMap) -> PluginResult<()>;
    
    /// Watch for configuration changes
    async fn watch(&self) -> PluginResult<ConfigWatcher>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigMap {
    pub data: std::collections::HashMap<String, serde_json::Value>,
}

pub struct ConfigWatcher;

// ============================================================================
// Error Handler Plugin Trait
// ============================================================================

pub trait ErrorHandler: Plugin {
    /// Classify an error
    fn classify(&self, error: &dyn std::error::Error) -> ErrorClass;
    
    /// Format error for logging
    fn format(&self, error: &dyn std::error::Error) -> String;
    
    /// Extract error context
    fn context(&self, error: &dyn std::error::Error) -> Option<ErrorContext>;
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum ErrorClass {
    /// User input error (4xx)
    InvalidInput,
    
    /// Authentication/authorization error (401/403)
    Authorization,
    
    /// Resource not found (404)
    NotFound,
    
    /// Resource already exists (409)
    Conflict,
    
    /// Rate limit or resource exhaustion (429)
    RateLimit,
    
    /// Internal error (5xx)
    Internal,
    
    /// Timeout
    Timeout,
    
    /// Network/connectivity error
    Network,
    
    /// Unknown
    Unknown,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ErrorContext {
    pub error_id: String,
    pub timestamp: String,
    pub request_id: Option<String>,
}

// ============================================================================
// Event Sourcing Plugin Trait
// ============================================================================

pub trait EventStore: Plugin {
    /// Append event to stream
    async fn append(&self, stream_id: &str, event: Event) -> PluginResult<EventId>;
    
    /// Read events from stream
    async fn read(&self, stream_id: &str, from: u64) -> PluginResult<Vec<Event>>;
    
    /// Read all events (with pagination)
    async fn read_all(&self, from: u64, limit: u64) -> PluginResult<Vec<Event>>;
    
    /// Subscribe to event stream
    async fn subscribe(&self, stream_id: &str) -> PluginResult<EventSubscriber>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: EventId,
    pub stream_id: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub metadata: std::collections::HashMap<String, String>,
    pub timestamp: String,
    pub version: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct EventId(pub u64);

pub struct EventSubscriber;

// ============================================================================
// Policy Engine Plugin Trait
// ============================================================================

pub trait PolicyEngine: Plugin {
    /// Evaluate a policy against a context
    async fn evaluate(
        &self,
        policy: &Policy,
        context: &PolicyContext,
    ) -> PluginResult<PolicyDecision>;
    
    /// Register a new policy
    async fn register(&self, policy: Policy) -> PluginResult<()>;
    
    /// Unregister a policy
    async fn unregister(&self, policy_id: &str) -> PluginResult<()>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Policy {
    pub id: String,
    pub name: String,
    pub rules: Vec<PolicyRule>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyRule {
    pub condition: String,
    pub action: PolicyAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PolicyAction {
    Allow,
    Deny,
    Log,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PolicyContext {
    pub principal: String,
    pub resource: String,
    pub action: String,
    pub attributes: std::collections::HashMap<String, String>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum PolicyDecision {
    Allow,
    Deny,
}

// ============================================================================
// Health Checker Plugin Trait
// ============================================================================

pub trait HealthChecker: Plugin {
    /// Check health of a component
    async fn check(&self) -> PluginResult<HealthStatus>;
    
    /// Start background health monitoring
    async fn monitor(&self) -> PluginResult<()>;
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HealthStatus {
    pub status: HealthState,
    pub checks: Vec<ComponentHealth>,
    pub timestamp: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum HealthState {
    Healthy,
    Degraded,
    Unhealthy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub name: String,
    pub status: HealthState,
    pub message: Option<String>,
}

// Similar detailed interfaces for Crypto, Git, Cache, Http, etc.
// (Full implementations in respective plugin crates)
```

---

## 3. Plugin Implementation Pattern

### Example: Config Loader Plugin

```rust
// phenotype-config-loader/src/plugin.rs

use phenotype_port_traits::{
    ConfigProvider, ConfigMap, Plugin, Capability, PluginError, PluginResult,
};

pub struct ConfigLoaderPlugin {
    figment: figment::Figment,
}

impl Plugin for ConfigLoaderPlugin {
    fn id(&self) -> &'static str {
        "config-loader"
    }
    
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &'static str {
        "Figment-based unified configuration loader"
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "load".to_string(),
                version: "1.0.0".to_string(),
            },
            Capability {
                name: "watch".to_string(),
                version: "1.0.0".to_string(),
            },
        ]
    }
    
    fn initialize(&self) -> PluginResult<()> {
        tracing::info!("Initializing config-loader plugin");
        Ok(())
    }
    
    fn shutdown(&self) -> PluginResult<()> {
        tracing::info!("Shutting down config-loader plugin");
        Ok(())
    }
    
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

#[async_trait::async_trait]
impl ConfigProvider for ConfigLoaderPlugin {
    async fn load(&self) -> PluginResult<ConfigMap> {
        let config = self.figment.extract()
            .map_err(|e| PluginError::Other(e.to_string()))?;
        Ok(ConfigMap { data: config })
    }
    
    fn validate(&self, _config: &ConfigMap) -> PluginResult<()> {
        // Custom validation logic
        Ok(())
    }
    
    async fn watch(&self) -> PluginResult<ConfigWatcher> {
        todo!("Implement file watcher")
    }
}
```

### Example: Error Handler Plugin

```rust
// phenotype-error-core/src/plugin.rs

use phenotype_port_traits::{
    ErrorHandler, ErrorClass, ErrorContext, Plugin, Capability, PluginError, PluginResult,
};

pub struct ErrorHandlerPlugin;

impl Plugin for ErrorHandlerPlugin {
    fn id(&self) -> &'static str {
        "error-handler"
    }
    
    fn version(&self) -> &'static str {
        env!("CARGO_PKG_VERSION")
    }
    
    fn description(&self) -> &'static str {
        "Canonical error classification and formatting"
    }
    
    fn capabilities(&self) -> Vec<Capability> {
        vec![
            Capability {
                name: "classify".to_string(),
                version: "1.0.0".to_string(),
            },
            Capability {
                name: "format".to_string(),
                version: "1.0.0".to_string(),
            },
        ]
    }
    
    fn initialize(&self) -> PluginResult<()> {
        Ok(())
    }
    
    fn shutdown(&self) -> PluginResult<()> {
        Ok(())
    }
    
    fn as_any(&self) -> &(dyn std::any::Any + Send + Sync) {
        self
    }
}

impl ErrorHandler for ErrorHandlerPlugin {
    fn classify(&self, error: &dyn std::error::Error) -> ErrorClass {
        // Classification logic using error downcasting
        let s = error.to_string().to_lowercase();
        if s.contains("not found") {
            ErrorClass::NotFound
        } else if s.contains("timeout") {
            ErrorClass::Timeout
        } else if s.contains("unauthorized") {
            ErrorClass::Authorization
        } else {
            ErrorClass::Internal
        }
    }
    
    fn format(&self, error: &dyn std::error::Error) -> String {
        format!("{:?}", error)
    }
    
    fn context(&self, _error: &dyn std::error::Error) -> Option<ErrorContext> {
        Some(ErrorContext {
            error_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            request_id: None,
        })
    }
}
```

---

## 4. Feature Flag Matrix

### Cargo.toml Example

```toml
[package]
name = "my-phenotype-app"
version = "0.1.0"
edition = "2021"

[dependencies]
phenotype-port-traits = "1.0"

# Optional plugin dependencies
phenotype-config-core = { version = "1.0", optional = true }
phenotype-config-loader = { version = "1.0", optional = true }
phenotype-error-core = { version = "1.0", optional = true }
phenotype-errors = { version = "1.0", optional = true }
phenotype-event-sourcing = { version = "1.0", optional = true }
phenotype-policy-engine = { version = "1.0", optional = true }
phenotype-health = { version = "1.0", optional = true }
phenotype-crypto = { version = "1.0", optional = true }
phenotype-git-core = { version = "1.0", optional = true }
phenotype-cache-adapter = { version = "1.0", optional = true }
phenotype-http-client-core = { version = "1.0", optional = true }
phenotype-logging = { version = "1.0", optional = true }
phenotype-telemetry = { version = "1.0", optional = true }
phenotype-process = { version = "1.0", optional = true }
# ... more optional plugins

[features]
# Base feature set (always included)
default = ["core"]
core = []  # Just the port-traits; no plugins

# Feature groups
config = ["phenotype-config-core", "phenotype-config-loader"]
errors = ["phenotype-error-core", "phenotype-errors"]
events = ["phenotype-event-sourcing"]
policy = ["phenotype-policy-engine"]
health = ["phenotype-health"]
crypto = ["phenotype-crypto"]
git = ["phenotype-git-core"]
cache = ["phenotype-cache-adapter"]
http = ["phenotype-http-client-core"]
logging = ["phenotype-logging"]
telemetry = ["phenotype-telemetry"]
process = ["phenotype-process"]

# Bundled feature sets
full = [
    "config", "errors", "events", "policy", "health",
    "crypto", "git", "cache", "http", "logging",
    "telemetry", "process",
]

minimal = ["core"]
standard = ["config", "errors", "health", "logging"]
distributed = ["config", "errors", "events", "policy", "health", "crypto", "git"]

[profile.release]
opt-level = "z"      # Optimize for size (LTO + tree-shaking)
lto = true           # Link-time optimization
codegen-units = 1   # Full optimization
strip = true         # Strip symbols
```

### Usage Example

```bash
# Build minimal (core only)
cargo build --release --no-default-features

# Build standard (config, errors, health, logging)
cargo build --release --features standard

# Build full (all plugins)
cargo build --release --features full

# Build custom subset
cargo build --release --features "config,errors,events,policy"
```

---

## 5. Runtime Plugin Discovery

### Application Bootstrap

```rust
// src/main.rs

use phenotype_port_traits::{PluginRegistry, ConfigProvider, ErrorHandler};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize registry
    let mut registry = PluginRegistry::new();
    
    // Register plugins based on features
    #[cfg(feature = "config")]
    {
        let config_plugin = phenotype_config_loader::ConfigLoaderPlugin::new();
        registry.register(config_plugin)?;
    }
    
    #[cfg(feature = "errors")]
    {
        let error_plugin = phenotype_error_core::ErrorHandlerPlugin;
        registry.register(error_plugin)?;
    }
    
    #[cfg(feature = "events")]
    {
        let event_plugin = phenotype_event_sourcing::EventPlugin::new();
        registry.register(event_plugin)?;
    }
    
    #[cfg(feature = "policy")]
    {
        let policy_plugin = phenotype_policy_engine::PolicyEnginePlugin::new();
        registry.register(policy_plugin)?;
    }
    
    #[cfg(feature = "health")]
    {
        let health_plugin = phenotype_health::HealthCheckerPlugin::new();
        registry.register(health_plugin)?;
    }
    
    // Initialize all plugins
    for plugin_id in registry.list() {
        if let Some(plugin) = registry.get(&plugin_id) {
            plugin.initialize()?;
        }
    }
    
    // Use plugins
    if let Some(config_plugin) = registry.get("config-loader") {
        // Downcast to ConfigProvider
        if let Some(config_provider) = config_plugin
            .as_any()
            .downcast_ref::<dyn ConfigProvider>()
        {
            let config = config_provider.load().await?;
            println!("Loaded config: {:?}", config);
        }
    }
    
    // Application logic here
    
    // Cleanup
    for plugin_id in registry.list() {
        if let Some(plugin) = registry.get(&plugin_id) {
            plugin.shutdown()?;
        }
    }
    
    Ok(())
}
```

---

## 6. Dependency Injection via Traits

### Service Layer Pattern

```rust
// src/services/config_service.rs

pub struct ConfigService {
    provider: std::sync::Arc<dyn ConfigProvider>,
}

impl ConfigService {
    pub fn new(provider: std::sync::Arc<dyn ConfigProvider>) -> Self {
        Self { provider }
    }
    
    pub async fn get_config(&self) -> Result<ConfigMap> {
        self.provider.load().await
    }
}

// src/services/error_service.rs

pub struct ErrorService {
    handler: std::sync::Arc<dyn ErrorHandler>,
}

impl ErrorService {
    pub fn new(handler: std::sync::Arc<dyn ErrorHandler>) -> Self {
        Self { handler }
    }
    
    pub fn classify(&self, error: &dyn std::error::Error) -> ErrorClass {
        self.handler.classify(error)
    }
}

// Usage in application
let config_service = ConfigService::new(
    registry.get("config-loader").unwrap()
);
let config = config_service.get_config().await?;
```

---

## 7. Testing Strategy

### Plugin Testing Pattern

```rust
// phenotype-config-loader/tests/integration.rs

use phenotype_port_traits::{Plugin, ConfigProvider};
use phenotype_config_loader::ConfigLoaderPlugin;

#[test]
fn test_plugin_metadata() {
    let plugin = ConfigLoaderPlugin::new();
    assert_eq!(plugin.id(), "config-loader");
    assert_eq!(plugin.version(), "1.0.0");
    assert!(!plugin.capabilities().is_empty());
}

#[tokio::test]
async fn test_plugin_load() {
    let plugin = ConfigLoaderPlugin::new();
    plugin.initialize().unwrap();
    
    let config = plugin.load().await.unwrap();
    assert!(!config.data.is_empty());
    
    plugin.shutdown().unwrap();
}

#[test]
fn test_plugin_registry() {
    let mut registry = PluginRegistry::new();
    let plugin = ConfigLoaderPlugin::new();
    
    registry.register(plugin).unwrap();
    assert!(registry.get("config-loader").is_some());
}
```

---

## 8. Migration Path from Current State

### Phase 2 Execution Steps

```
WEEK 1: Foundation Setup
├─ Day 1-2: Expand phenotype-port-traits with all trait definitions
├─ Day 3: Implement PluginRegistry and Plugin base trait
├─ Day 4: Add feature flags to all 24 plugin crates
└─ Day 5: Verify compilation with feature combinations

WEEK 2-3: Plugin Implementation
├─ Implement Plugin trait in phenotype-config-loader
├─ Implement Plugin trait in phenotype-error-core
├─ Implement Plugin trait in phenotype-event-sourcing
├─ Implement Plugin trait in phenotype-policy-engine
├─ Implement Plugin trait in phenotype-health
└─ ... (remaining 19 crates)

WEEK 4: Bootstrap & Testing
├─ Update main.rs to use PluginRegistry
├─ Add feature gate compile tests
├─ Measure binary size with different features
├─ Performance benchmarks (plugin init overhead)
└─ Documentation & examples

WEEK 5: Release & Adoption
├─ Release v1.0.0 with federation support
├─ Update all dependent projects
├─ Deprecate old static linking approach
└─ Monitor adoption
```

---

## 9. Benefits Summary

| Aspect | Before | After |
|--------|--------|-------|
| Binary Size (full) | 50-100MB | 50-100MB (no change if all features) |
| Binary Size (minimal) | 50-100MB | 5MB (90% reduction!) |
| Startup Time (full) | 500ms | 500ms (no change) |
| Startup Time (minimal) | 500ms | 50ms (90% reduction!) |
| Modularity | 85% | 100% (all crates have traits) |
| Swappability | 20% (some) | 100% (all plugins) |
| Testing Ease | Medium | High (mock plugins) |
| Dependency Bloat | High | Zero (only compile what needed) |

---

## 10. Success Criteria

- [ ] All 24 plugin crates implement Plugin trait
- [ ] phenotype-port-traits exports all required trait interfaces
- [ ] Feature flags work for all 7 feature groups
- [ ] Binary size reduces by 80-90% with minimal features
- [ ] Plugin registry discovery works at runtime
- [ ] All tests pass with feature combinations
- [ ] Documentation complete with examples
- [ ] Performance overhead < 5% (plugin init)

---

**Architecture Ready:** Phase 2 Federation can begin immediately after dependency extraction completes.

