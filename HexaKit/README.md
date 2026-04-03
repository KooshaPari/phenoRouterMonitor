# HexaKit

**Unified hexagonal architecture framework for multiple languages.**

HexaKit provides reference implementations of hexagonal architecture (Ports & Adapters) across Go, Python, Rust, and TypeScript.

## Absorbed Projects

- **HexaGo** → `go/` — Go-Hex: Go hexagonal architecture kit
- **HexaPy** → `python/` — PyHex: Python hexagonal architecture kit
- **Hexacore** → `rust/` — Phenotype Kits: Curated Rust libraries
- **HexaType** → `typescript/` — TsHex: TypeScript hexagonal architecture kit

## Repository Structure

```
HexaKit/
├── go/           # Go hexagonal architecture implementation
│   ├── domain/       # Domain entities and business logic
│   ├── application/  # Use cases and services
│   ├── ports/        # Interface definitions
│   └── infrastructure/ # External concerns
├── python/       # Python hexagonal architecture implementation
│   ├── src/
│   │   ├── domain/
│   │   ├── application/
│   │   ├── adapters/
│   │   └── ports/
│   └── tests/
├── rust/         # Rust hexagonal architecture workspace
│   └── crates/       # Individual hexagonal crates
└── typescript/   # TypeScript hexagonal architecture implementation
    └── src/
        ├── domain/
        ├── application/
        ├── adapters/
        └── ports/
```

## Language-Specific Features

| Feature | Go | Python | Rust | TypeScript |
|---------|-----|--------|------|------------|
| **Hexagonal Arch** | ✅ | ✅ | ✅ | ✅ |
| **Domain Layer** | ✅ | ✅ | ✅ | ✅ |
| **Application Layer** | ✅ | ✅ | ✅ | ✅ |
| **Ports & Adapters** | ✅ | ✅ | ✅ | ✅ |
| **Infrastructure** | ✅ | ✅ | ✅ | ✅ |
| **Clean Architecture** | ✅ | ✅ | ✅ | ✅ |
| **DDD Support** | ✅ | ✅ | ✅ | ✅ |
| **SOLID Principles** | ✅ | ✅ | ✅ | ✅ |

## Quick Start

### Go
```go
import "github.com/KooshaPari/HexaKit/go/domain"

// Use hexagonal patterns from go/
```

### Python
```python
from hexakit.python.domain import Entity

# Use hexagonal patterns from python/
```

### Rust
```rust
use hexakit::rust::domain::Entity;

// Use hexagonal patterns from rust/
```

### TypeScript
```typescript
import { Entity } from '@phenotype/hexakit/typescript';

// Use hexagonal patterns from typescript/
```

## Philosophy

Each language implementation follows the same hexagonal principles while respecting language idioms:

1. **Domain First**: Pure business logic, no external dependencies
2. **Ports Define Contracts**: Interfaces that adapters implement
3. **Adapters Isolate Infrastructure**: External concerns kept at the edges
4. **Dependency Rule**: Dependencies point inward toward domain

## Contributing

Each language directory is a self-contained project with its own:
- Build system (Cargo, Go modules, npm, Poetry)
- Testing framework
- CI/CD configuration
- Documentation

## License

MIT License - See individual language directories for specific license files.

---

*Merged from HexaGo, HexaPy, Hexacore, and HexaType on 2026-04-02.*
