# Hexagon Template Registry - Consolidation Plan

## Overview

All hexagonal architecture templates are being consolidated into this single registry repository.

## Migration Status

| Source | Destination | Status |
|--------|-------------|--------|
| hexagon-go | templates/go/ | Migrated |
| hexagon-rust | templates/rust/ | Migrated |
| hexagon-zig | templates/zig/ | Migrated |
| hexagon-java | templates/java/ | Planned |
| hexagon-kotlin | templates/kotlin/ | Planned |
| hexagon-elixir | templates/elixir/ | Planned |
| hexagon-swift | templates/swift/ | Planned |

## Repository Structure

```
hexagon/
├── templates/    # Full hexagonal architecture templates
│   ├── go/
│   ├── rust/
│   └── zig/
└── registry.json # Template metadata
```

## Planned Features

- Unified template generator script
- Language-specific scaffolding integration
- Template versioning and registry
