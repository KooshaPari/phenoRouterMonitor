# PhenoConfig - Polyglot Ecosystem Strategy

## Status: ACTIVE - Multi-Language Config Management

PhenoConfig provides **universal configuration management** across all language ecosystems in the Phenotype platform.

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                    Language Frontends                                │
├──────────────┬──────────────┬──────────────┬────────────────────────┤
│ TypeScript   │ Python 3.14+ │ Go           │ Rust (Core)            │
│ pheno-config │ (planned)    │ (planned)    │ pheno-config-core      │
│ -ts          │              │              │                        │
├──────────────┼──────────────┼──────────────┼────────────────────────┤
│ • Zod        │ • Pydantic   │ • Viper      │ • Serde               │
│ validation   │   v2         │   style      │ • Compile-time        │
│ • Hexagonal  │ • Hexagonal  │ • Hexagonal  │   validation          │
│   adapters   │   adapters   │   adapters   │ • Zero-cost           │
└──────────────┴──────────────┴──────────────┴────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────────────┐
│                 Universal Config Core (Rust)                         │
├─────────────────────────────────────────────────────────────────────┤
│  • Layered Configs (File → Env → CLI → Remote)                      │
│  • Validation Engine (serde + custom validators)                    │
│  • Hot Reload (notify + debounce)                                   │
│  • Schema Registry (shared types)                                   │
│  • Secrets Management (encryption at rest)                          │
└─────────────────────────────────────────────────────────────────────┘
```

---

## Ecosystem-Specific Frontends

### ✅ TypeScript (ACTIVE)

| Aspect | Implementation |
|--------|----------------|
| **Package** | `@phenotype/config` |
| **Validation** | Zod 3.x |
| **Runtime** | Node.js 18+, Bun, Deno |
| **Adapters** | File, Environment, CLI, HTTP |
| **Location** | `PhenoConfig/crates/pheno-config-ts/` |

**Usage:**
```typescript
import { config } from '@phenotype/config';

const appConfig = config()
  .fromFile('app.toml')
  .fromEnv('APP_')
  .validate(z.object({ port: z.number() }))
  .build();
```

### 🐍 Python 3.14+ (PLANNED)

| Aspect | Implementation |
|--------|----------------|
| **Package** | `phenotype-config` (PyPI) |
| **Validation** | Pydantic v2 |
| **Runtime** | Python 3.14+ (pattern matching, better typing) |
| **Adapters** | File, Environment, CLI |
| **Location** | `PhenoConfig/crates/pheno-config-py/` (planned) |

**Design Considerations:**
- Python 3.14 brings significant typing improvements
- Pydantic v2 has Rust core (performance)
- Could share validation schemas with Rust via pyo3

**Usage:**
```python
from phenotype_config import Config
from pydantic import BaseModel

class AppConfig(BaseModel):
    port: int

config = Config() \
    .from_file("app.toml") \
    .from_env("APP_") \
    .validate(AppConfig) \
    .build()
```

### 🐹 Go (PLANNED)

| Aspect | Implementation |
|--------|----------------|
| **Package** | `github.com/phenotype/config` |
| **Validation** | Custom + go-playground/validator |
| **Adapters** | Viper-compatible |
| **Location** | `PhenoConfig/crates/pheno-config-go/` (planned) |

**Usage:**
```go
import "github.com/phenotype/config"

cfg := config.New().
    FromFile("app.toml").
    FromEnv("APP_").
    Validate(AppConfig{}).
    Build()
```

### 🦀 Rust (CORE)

| Aspect | Implementation |
|--------|----------------|
| **Crate** | `pheno-config-core` |
| **Validation** | serde + validator |
| **Runtime** | Async (tokio) + Sync |
| **Location** | `PhenoConfig/crates/pheno-config-core/` |

**Usage:**
```rust
use pheno_config_core::ConfigBuilder;

let config: AppConfig = ConfigBuilder::new()
    .from_file("app.toml")
    .from_env()
    .validate()
    .build()
    .await?;
```

---

## Integration Matrix

| Feature | Rust | TypeScript | Python | Go |
|---------|------|------------|--------|-----|
| **File Formats** | TOML/YAML/JSON | TOML/YAML/JSON | TOML/YAML/JSON | TOML/YAML/JSON |
| **Env Vars** | ✅ | ✅ | ✅ | ✅ |
| **CLI Args** | ✅ | ✅ | ✅ | ✅ |
| **Remote (etcd)** | ✅ | ✅ | ⚠️ | ⚠️ |
| **Hot Reload** | ✅ | ✅ | ⚠️ | ⚠️ |
| **Secrets** | ✅ | ✅ | ⚠️ | ⚠️ |
| **Schema Share** | Native | WASM/napi | pyo3 | FFI |

---

## Migration from Legacy

| Legacy | Replacement | Migration Path |
|--------|-------------|----------------|
| Settly (Rust) | `pheno-config-core` | Direct replacement |
| phenotype-config-ts | `pheno-config-ts` | Same API, improved |
| phenotype-middleware-py | `pheno-config-py` (planned) | Similar patterns |

---

## Roadmap

### Phase 1: Core Consolidation ✅
- [x] Merge Settly → pheno-config-core
- [x] Merge phenotype-config-ts → pheno-config-ts
- [x] Create unified PhenoConfig workspace

### Phase 2: Python Frontend 🔄
- [ ] Create pheno-config-py crate
- [ ] Pydantic v2 integration
- [ ] Shared schema validation

### Phase 3: Go Frontend 📋
- [ ] Create pheno-config-go crate
- [ ] Viper-compatible adapters

### Phase 4: Universal Schema 📋
- [ ] JSON Schema export from Rust
- [ ] Generate TS/Python/Go types
- [ ] Single source of truth

