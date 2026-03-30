# Migration Guide: phenotype-ports-canonical

## Overview

`phenotype-ports-canonical` consolidates trait definitions previously scattered
across `phenotype-contracts`, `harness_cache`, `harness_interfaces`,
`harness_teammates`, and `phenotype-event-sourcing`.

## Import Replacements

### Domain Models

```diff
- use phenotype_contracts::models::{Entity, ValueObject, AggregateRoot};
- use phenotype_contracts::models::aggregate::DomainEvent;
+ use phenotype_ports_canonical::{Entity, ValueObject, AggregateRoot, DomainEvent};
```

### Inbound Ports

```diff
- use phenotype_contracts::ports::inbound::{UseCase, CommandHandler, QueryHandler, EventHandler, Error};
+ use phenotype_ports_canonical::{UseCase, CommandHandler, QueryHandler, EventHandler};
+ use phenotype_ports_canonical::PortError;
```

### Outbound Ports

```diff
- use phenotype_contracts::ports::outbound::{Repository, CachePort, SecretPort, EventPublisher, EventSubscriber, Error};
+ use phenotype_ports_canonical::{Repository, CachePort, SecretPort, EventPublisher, EventSubscriber};
+ use phenotype_ports_canonical::PortError;
```

### Cache Extensions

```diff
- use phenotype_contracts::ports::outbound::cache::{CacheJsonPort, CacheCounterPort, CacheLockPort};
+ use phenotype_ports_canonical::outbound::{CacheJsonPort, CacheCounterPort, CacheLockPort};
```

### Secret Extensions

```diff
- use phenotype_contracts::ports::outbound::secret::{VersionedSecretPort, SecretRotator};
+ use phenotype_ports_canonical::outbound::{VersionedSecretPort, SecretRotator};
```

### Event Sourcing

```diff
- use phenotype_event_sourcing::EventStore;
+ use phenotype_ports_canonical::EventStore;
+ use phenotype_ports_canonical::eventsourcing::Snapshot;
```

### Health (new, from harness_teammates)

```diff
- use harness_teammates::ports::HealthCheckPort;
+ use phenotype_ports_canonical::HealthChecker;
+ use phenotype_ports_canonical::health::HealthStatus;
```

## Error Type Migration

The two separate error enums (`inbound::Error` and `outbound::Error`) are
unified into `PortError`:

```diff
- use phenotype_contracts::ports::inbound::Error;
- use phenotype_contracts::ports::outbound::Error;
+ use phenotype_ports_canonical::PortError;
```

`PortError` is a superset of both, with variants: `NotFound`, `AlreadyExists`,
`Validation`, `Conflict`, `PermissionDenied`, `Connection`, `Timeout`,
`Internal`.

## Cargo.toml

```toml
[dependencies]
phenotype-ports-canonical = { path = "../phenotype-ports-canonical" }
# Remove:
# phenotype-contracts = ...
# phenotype-event-sourcing = ...
```

## DomainEvent Note

`phenotype-contracts` had two `DomainEvent` definitions:

1. `models::aggregate::DomainEvent` -- marker trait (no serde bounds)
2. `ports::outbound::event::DomainEvent` -- requires `Serialize + Deserialize`

The canonical version uses the marker form (option 1). Serialization is the
adapter's concern, not the event's. If you need serde bounds, add them to your
concrete event types or use an adapter-specific wrapper.
