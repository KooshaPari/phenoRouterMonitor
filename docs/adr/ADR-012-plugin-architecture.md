# ADR-012: Plugin Architecture for AgilePlus

**Status:** Proposed

**Date:** 2026-03-25

## Context

AgilePlus is evolving from a monolithic CLI to a platform with multiple entry points (CLI, MCP server, API, Dashboard). We need to support:
- Productization: Custom deployments for different organizations
- Pluginization: Third-party extensions and integrations
- Libification: Shared code extracted to reusable libraries

## Decision

We will adopt a **Plugin Architecture** pattern using Rust's trait objects and dynamic dispatch.

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                      Application Core                        │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Domain (Entities, Value Objects)         │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              Use Cases / Services                     │   │
│  └─────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
                              │
              ┌───────────────┼───────────────┐
              ▼               ▼               ▼
        ┌──────────┐   ┌──────────┐   ┌──────────┐
        │   CLI    │   │   API    │   │   MCP    │
        │ Adapter  │   │ Adapter  │   │ Adapter  │
        └──────────┘   └──────────┘   └──────────┘
```

### Port Trait Definition

```rust
// crates/agileplus-domain/src/ports/mod.rs

/// Core plugin trait for all adapters
pub trait AdapterPlugin: Send + Sync {
    fn name(&self) -> &str;
    fn version(&self) -> &str;
    fn initialize(&self, config: Config) -> Result<(), PluginError>;
}

/// VCS adapter plugin
pub trait VcsAdapter: AdapterPlugin {
    fn clone_repo(&self, url: &str, dest: &Path) -> Result<()>;
    fn push(&self, branch: &str) -> Result<()>;
    // ...
}

/// Storage adapter plugin
pub trait StorageAdapter: AdapterPlugin {
    fn save(&self, entity: &dyn Entity) -> Result<EntityId>;
    fn load(&self, id: EntityId) -> Result<Box<dyn Entity>>;
    // ...
}

/// Agent adapter plugin
pub trait AgentAdapter: AdapterPlugin {
    fn dispatch(&self, task: Task) -> Result<TaskResult>;
    fn status(&self, task_id: &str) -> Result<TaskStatus>;
    // ...
}
```

### Plugin Registration

```rust
// crates/agileplus-core/src/plugin_registry.rs

pub struct PluginRegistry {
    vcs: HashMap<String, Box<dyn VcsAdapter>>,
    storage: HashMap<String, Box<dyn StorageAdapter>>,
    agents: HashMap<String, Box<dyn AgentAdapter>>,
}

impl PluginRegistry {
    pub fn register_vcs(&mut self, adapter: Box<dyn VcsAdapter>) {
        self.vcs.insert(adapter.name().to_string(), adapter);
    }

    pub fn vcs(&self, name: &str) -> Option<&dyn VcsAdapter> {
        self.vcs.get(name).map(|b| b.as_ref())
    }
}
```

## Plugin Crate Structure

```
crates/
├── agileplus-plugin-core/      # Plugin traits and registry
├── agileplus-plugin-git/        # Git VCS adapter
├── agileplus-plugin-sqlite/     # SQLite storage adapter
├── agileplus-plugin-postgres/   # PostgreSQL storage adapter (future)
└── agileplus-plugin-llm/       # LLM agent adapters
```

## Benefits

| Benefit | Description |
|---------|-------------|
| **Extensibility** | Add new adapters without modifying core |
| **Testability** | Mock adapters for unit testing |
| **Productization** | Custom adapters per deployment |
| **Maintainability** | Smaller, focused crates |
| **Polymorphism** | Runtime plugin selection |

## X-DDs Applied

- **Hexagonal Architecture**: Ports define boundaries
- **Interface Segregation**: Small, focused trait methods
- **Dependency Inversion**: Core depends on abstractions
- **Open/Closed**: Open for extension, closed for modification
- **Plugin Pattern**: Dynamic discovery and loading
- **SpecDD**: This ADR documents the specification

## Consequences

### Positive
- Clear extension points
- Independent release cycles
- Easy to add new VCS/storage/agent backends

### Negative
- Dynamic dispatch overhead (minimal)
- More complex dependency management
- Requires careful API stability

## Alternatives Considered

1. **Static Dispatch**: Compile-time plugin selection via features. Rejected for less flexibility.
2. **External Plugin System**: WASM-based plugins. Deferred for complexity.
3. **Monolithic Adapter**: Single adapter per deployment. Rejected for maintainability.

## References

- [Hexagonal Architecture](../ARCHITECTURE.md)
- [Rust Plugin Pattern](https://rust-lang.github.io/api-guidelines/type-safety.html)
- [ADR-001: Architecture Overview](./ADR-001-architecture-overview.md)
