# PhenoConfig - External Library Ecosystem Strategy

## Status: ACTIVE - Leverage Mature External Libraries

**Principle**: Our ecosystem is new. Instead of building custom config libraries, we wrap **proven external libraries** with hexagonal adapters.

---

## External Library Stack by Language

| Language | External Library | Our Wrapper | Purpose |
|----------|------------------|-------------|---------|
| **TypeScript** | [Zod](https://zod.dev/) + [convict](https://github.com/mozilla/node-convict) | `pheno-config-ts` | Schema validation + config management |
| **Python 3.14+** | [Pydantic v2](https://docs.pydantic.dev/) | `pheno-config-py` | Settings management + validation |
| **Go** | [Viper](https://github.com/spf13/viper) + [validator](https://github.com/go-playground/validator) | `pheno-config-go` | Config + validation |
| **Rust** | [config-rs](https://github.com/mehcode/config-rs) + [serde](https://serde.rs/) | `pheno-config-rs` | Layered configs + serialization |
| **Zig** | [zod-zig](https://github.com/andrewrk/zig-uri) pattern | `pheno-config-zig` | Manual parsing + validation |

---

## Hexagonal Adapter Pattern

```
┌────────────────────────────────────────────────────────────────────┐
│                   External Library (Ecosystem)                      │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐                  │
│  │    Zod     │  │  Pydantic  │  │   Viper    │  ...              │
│  │   (TS)     │  │   (Py)     │  │   (Go)     │                  │
│  └─────────────┘  └─────────────┘  └─────────────┘                  │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Ports)
┌────────────────────────────────────────────────────────────────────┐
│                PhenoConfig Hexagonal Adapters                     │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  • ConfigPort (load, reload, validate)                       │  │
│  │  • ValidationPort (schema, error handling)                   │  │
│  │  • SourcePort (file, env, cli, remote)                       │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
                              │
                              ▼ (Domain)
┌────────────────────────────────────────────────────────────────────┐
│                Shared Domain (Language Agnostic)                  │
│  ┌──────────────────────────────────────────────────────────────┐  │
│  │  • ConfigSchema (JSON Schema / OpenAPI spec)                 │  │
│  │  • ValidationRules (cross-language rules)                    │  │
│  │  • ConfigEvents (hot reload notifications)                   │  │
│  └──────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────┘
```

---

## Implementation Strategy

### TypeScript (Zod + Convict)

**External**: `zod` for validation, `convict` for config management
**Our Layer**: Hexagonal ports/adapters around these

```typescript
// External: Zod
import { z } from 'zod';

// Our adapter
import { ConfigPort, ValidationPort } from 'pheno-config-ts';

const config = ConfigPort.create()
  .withValidation(ValidationPort.zod(z.object({ port: z.number() })))
  .load();
```

### Python 3.14+ (Pydantic Settings)

**External**: `pydantic-settings` + `pydantic.v2`
**Our Layer**: Hexagonal config service

```python
# External: Pydantic
from pydantic_settings import BaseSettings
from pydantic import Field

# Our adapter
from pheno_config import ConfigPort, ValidationPort

class AppConfig(BaseSettings):
    port: int = Field(default=8080)
    
config = ConfigPort.create(AppConfig).load()
```

### Go (Viper + Validator)

**External**: `spf13/viper` + `go-playground/validator`
**Our Layer**: Hexagonal config manager

```go
// External: Viper
import "github.com/spf13/viper"
import "github.com/go-playground/validator/v10"

// Our adapter
import "github.com/phenotype/config-go/ports"

config := ports.ConfigPort.Create(
    ports.WithViper(viper.New()),
    ports.WithValidation(validator.New()),
).Load()
```

### Rust (config-rs + Serde)

**External**: `config-rs` + `serde`
**Our Layer**: Already in `pheno-config-core`

```rust
// External: config-rs
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;

// Our adapter
use pheno_config_rs::ports::{ConfigPort, ValidationPort};

#[derive(Debug, Deserialize)]
struct AppConfig {
    port: u16,
}

let config: AppConfig = ConfigPort::new()
    .with_source(SourcePort::file("app.toml"))
    .with_validation(ValidationPort::serde())
    .load()?;
```

---

## Why External Libraries?

| Criterion | External (Zod/Viper/Pydantic) | Custom Build |
|-----------|------------------------------|--------------|
| **Maturity** | Battle-tested, millions of users | New, untested |
| **Ecosystem** | Integrates with existing tools | Isolated |
| **Maintenance** | Community-maintained | Our burden |
| **Docs/Examples** | Extensive | Ours to write |
| **Contributors** | Open source community | Just us |

**Our Value Add**: 
- Hexagonal architecture adapters
- Cross-language schema consistency
- Unified config events/monitoring
- Migration tooling

---

## Schema Sharing Across Languages

```json
// shared-schema.json (OpenAPI 3.1 spec)
{
  "AppConfig": {
    "type": "object",
    "properties": {
      "port": { "type": "integer", "minimum": 1, "maximum": 65535 },
      "database": { "$ref": "#/components/schemas/DatabaseConfig" }
    },
    "required": ["port"]
  }
}
```

**Generate Types**:
- TypeScript: `openapi-typescript`
- Python: `datamodel-code-generator`
- Go: `oapi-codegen`
- Rust: `typify`

---

## Consolidated Repositories

| Before | After | External Lib | Our Code |
|--------|-------|--------------|----------|
| `Settly` | `pheno-config-rs` | config-rs | ~10% adapters |
| `phenotype-config-ts` | `pheno-config-ts` | Zod | ~15% adapters |
| `phenotype-config` | DELETED | - | - |
| `Configra` | DELETED | - | - |

---

## Migration Guide

### From Custom to External

```diff
- // Old: Custom validation
- import { validate } from 'phenotype-config';
- validate(config, rules);

+ // New: Zod (external)
+ import { z } from 'zod';
+ import { withZod } from 'pheno-config-ts';
+ withZod(z.object({ port: z.number() }));
```

---

## Roadmap

### Phase 1: TS Adapter ✅
- [x] Zod integration
- [x] Hexagonal ports

### Phase 2: Python Adapter 🔄
- [ ] Pydantic v2 integration
- [ ] ConfigPort implementation

### Phase 3: Go Adapter 📋
- [ ] Viper integration
- [ ] ValidationPort with go-playground/validator

### Phase 4: Schema Sync 📋
- [ ] OpenAPI spec generation
- [ ] Type generation for all languages

---

## Result

**Leverage ecosystem maturity instead of reinventing.**

- **4 custom config repos** → **1 unified strategy**
- **100% custom code** → **~90% external libs, ~10% adapters**
- **Maintenance burden**: Community + thin wrappers
