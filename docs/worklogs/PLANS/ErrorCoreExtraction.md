# Error Core Extraction Plan

**Created:** 2026-03-29
**Status:** Planned
**Priority:** P0 (Critical)
**Estimated LOC Savings:** ~400 lines

## Problem Statement

8 independent error enum definitions exist across the AgilePlus crates, with duplicated variants like `NotFound`, `Serialization`, `Conflict`, and `StorageError`.

## Current Error Types

| Crate | Error Type | Lines | Key Variants |
|-------|------------|-------|--------------|
| agileplus-api | `ApiError` | 67 | NotFound, Internal |
| agileplus-p2p | `SyncError`, `PeerDiscoveryError`, `ConnectionError` | 78 | Nats, Serialization, Discovery |
| agileplus-sync | `SyncError` | 24 | Store, Nats, Serialization |
| agileplus-domain | `DomainError` | 50 | NotFound, Conflict, InvalidTransition |
| agileplus-events | `EventError` | 53 | NotFound, StorageError, SequenceGap |
| agileplus-graph | `GraphError` | 326 | ConnectionError, QueryError |
| agileplus-cache | `CacheError` | 129 | Serialization, Redis, NotFound |
| phenotype-port-interfaces | `PortError` | 51 | NotFound, Validation, Storage |

**Total:** ~778 lines of error type definitions

## Proposed Architecture

```
agileplus-error-core/
├── src/
│   ├── lib.rs
│   ├── domain.rs       # DomainError variants
│   ├── api.rs          # ApiError with IntoResponse
│   ├── storage.rs      # StorageError, NotFound
│   ├── sync.rs         # SyncError, NatsError
│   ├── serialization.rs # SerError with #[from]
│   └── traits.rs       # ErrorMarker traits
└── Cargo.toml
```

## Duplicated Variants to Extract

### High Priority (appear in 3+ error types)

| Variant | Appears In | Proposed Location |
|---------|------------|------------------|
| `NotFound(String)` | ApiError, DomainError, EventError, CacheError, PortError | `storage.rs` |
| `Serialization(String)` | SyncError, CacheError, EventError | `serialization.rs` |
| `Storage(String)` | EventError, PortError | `storage.rs` |

### Medium Priority (appear in 2 error types)

| Variant | Appears In | Proposed Location |
|---------|------------|------------------|
| `Conflict(String)` | DomainError, PortError | `domain.rs` |
| `Internal(String)` | ApiError, GraphError | `api.rs` |

## Implementation Plan

### Phase 1: Create agileplus-error-core (3 hours)

```rust
// src/lib.rs
pub mod domain;
pub mod api;
pub mod storage;
pub mod serialization;
pub mod sync;

pub use domain::DomainError;
pub use api::ApiError;
pub use storage::{StorageError, NotFoundError};
pub use serialization::SerializationError;
pub use sync::SyncError;
```

### Phase 2: Define Shared Error Types (4 hours)

```rust
// src/storage.rs
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Storage error: {0}")]
    Storage(String),

    #[error("Connection failed: {0}")]
    Connection(String),
}

// Marker trait for NotFound
pub trait NotFoundError {
    fn not_found(id: impl Into<String>) -> Self;
}
```

### Phase 3: Migrate Error Types (8 hours)

Per crate migration:

1. Add `agileplus-error-core` dependency
2. Replace local error variants with imports
3. Add `#[from]` derives for shared error types
4. Update `IntoResponse` implementations
5. Remove local error definitions

### Phase 4: Verification (2 hours)

```bash
# Ensure all error conversions work
cargo check --workspace
cargo test
```

## Migration Example

**Before:**
```rust
// crates/agileplus-domain/src/error.rs
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Conflict: {0}")]
    Conflict(String),
}
```

**After:**
```rust
// crates/agileplus-domain/src/error.rs
use agileplus_error_core::{DomainError, StorageError, NotFound};

impl From<StorageError> for DomainError {
    fn from(e: StorageError) -> Self {
        match e {
            StorageError::NotFound(id) => DomainError::NotFound(id),
            _ => DomainError::Internal(e.to_string()),
        }
    }
}
```

## Effort Estimate

| Phase | Effort | Risk |
|-------|--------|------|
| Create error-core crate | 3 hours | Low |
| Define shared types | 4 hours | Low |
| Migrate 8 crates | 8 hours | Medium |
| Verification | 2 hours | Low |
| **Total** | **17 hours** | — |

## LOC Impact

| Metric | Before | After | Savings |
|--------|--------|-------|---------|
| Error type LOC | 778 | ~300 | **478 lines (62%)** |
| Test LOC | ~200 | ~150 | 50 lines |
| **Total** | **~978** | **~450** | **~528 lines** |

## Action Items

- [ ] 🔴 **CRITICAL** Create `crates/agileplus-error-core` crate
- [ ] 🟡 **HIGH** Define `StorageError`, `NotFound`, `Serialization` in storage.rs
- [ ] 🟡 **HIGH** Define `DomainError` variants in domain.rs
- [ ] 🟡 **HIGH** Define `ApiError` with `IntoResponse` in api.rs
- [ ] 🟠 **MEDIUM** Migrate `agileplus-domain` first (template)
- [ ] 🟠 **MEDIUM** Migrate remaining 7 crates
- [ ] 🟢 **LOW** Run full test suite
- [ ] 🟢 **LOW** Update worklogs/DUPLICATION.md

## Related

- worklogs/DUPLICATION.md (Error Type Duplication)
- worklogs/ARCHITECTURE.md (hexagonal-rs has RepositoryError pattern)
