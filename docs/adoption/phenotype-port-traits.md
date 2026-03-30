# phenotype-port-traits Adoption Guide

## Overview

`phenotype-port-traits` provides canonical async trait definitions for the hexagonal architecture pattern.

## Quick Start

### Add Dependency

```toml
[dependencies]
phenotype-port-traits = { path = "../crates/phenotype-port-traits" }
```

### Use Inbound Ports

```rust
use phenotype_port_traits::inbound::{UseCase, CommandHandler, QueryHandler};
use phenotype_port_traits::inbound::use_case::UseCase;
use async_trait::async_trait;

#[async_trait]
impl<I, O> UseCase<I, O> for MyService
where
    I: Send + Sync,
    O: Send + Sync,
{
    async fn execute(&self, input: I) -> Result<O, Self::Error> {
        // Your business logic
        Ok(output)
    }
}
```

### Use Outbound Ports

```rust
use phenotype_port_traits::outbound::{Repository, CachePort, EventPublisher};

#[async_trait]
impl<E, I> Repository<E, I> for SqliteRepository
where
    E: Entity + Send + Sync,
    I: EntityId + Send + Sync,
{
    async fn find(&self, id: &I) -> Result<Option<E>, RepositoryError> {
        // Implementation
        Ok(None)
    }
}
```

## Traits Available

### Inbound Ports

| Trait | Purpose | Methods |
|-------|---------|---------|
| `UseCase<I, O>` | Generic use case | `execute(input) -> Result<O, E>` |
| `CommandHandler<C>` | CQRS command handler | `handle(cmd) -> Result<(), E>` |
| `QueryHandler<Q, R>` | CQRS query handler | `handle(query) -> Result<R, E>` |
| `EventHandler<E>` | Domain event handler | `handle(event) -> Result<(), E>` |

### Outbound Ports

| Trait | Purpose | Methods |
|-------|---------|---------|
| `Repository<E, I>` | Persistence | `find`, `save`, `delete` |
| `CachePort` | Caching | `get`, `set`, `delete` |
| `EventPublisher` | Event emission | `publish`, `publish_batch` |
| `SecretPort` | Secrets management | `get`, `set`, `rotate` |

## Migration from Custom Traits

### Before

```rust
#[async_trait]
pub trait MyCustomRepository {
    async fn find(&self, id: &str) -> Result<Option<Entity>, Error>;
    async fn save(&self, entity: &Entity) -> Result<(), Error>;
}
```

### After

```rust
use phenotype_port_traits::outbound::Repository;
use phenotype_port_traits::models::{Entity, EntityId};

#[async_trait]
impl<E, I> Repository<E, I> for MyRepository
where
    E: Entity + Send + Sync,
    I: EntityId + Send + Sync,
{
    // Default implementations available for common patterns
}
```

## Feature Flags

```toml
[dependencies]
phenotype-port-traits = { 
    path = "../crates/phenotype-port-traits",
    features = ["inbound", "outbound", "models"]
}
```

## Related Crates

- `phenotype-error-core` - Error types compatible with these traits
- `phenotype-event-sourcing` - Event patterns using these traits
