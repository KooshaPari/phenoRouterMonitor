# Phenotype Skills

> Modular Skill System for Agent Orchestration

A comprehensive skill framework for building extensible agent capabilities with hot-reloading, versioning, and dependency management.

## Features

- **Hot Reloading**: Update skills without restarting agents
- **Version Management**: Semantic versioning for skill compatibility
- **Dependency Resolution**: Automatic dependency graph management
- **Multi-Language**: Rust, Python, TypeScript, C#, Zig skill support
- **Sandboxing**: Secure execution environment via NanoVMS three-tier isolation
- **Registry**: Centralized skill discovery and distribution

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                         Phenotype Skills System                             │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Skill Registry                               │   │
│  │                                                                      │   │
│  │   ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │   │
│  │   │  Skill       │  │  Skill       │  │  Skill       │         │   │
│  │   │  (v1.2.0)    │  │  (v2.0.1)    │  │  (v0.9.0)    │         │   │
│  │   │  web_search  │  │  code_gen    │  │  file_parse  │         │   │
│  │   └──────────────┘  └──────────────┘  └──────────────┘         │   │
│  │                                                                      │   │
│  │   Dependencies, Versions, Metadata                                   │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Skill Loader                                 │   │
│  │                                                                      │   │
│  │   Dynamic Loading → Hot Reload → Dependency Resolution              │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Skill Sandbox                                │   │
│  │                                                                      │   │
│  │   Tier 1: WASM (~1ms) │ Tier 2: gVisor (~90ms) │ Tier 3: Firecracker  │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                         Skill Runtime                                │   │
│  │                                                                      │   │
│  │   Rust │ Python │ TypeScript │ C# │ Zig                               │   │
│  │                                                                      │   │
│  └──────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

## Packages

| Package | Registry | Language | Status |
|---------|----------|----------|--------|
| `phenotype-skills` | crates.io | Rust | 🚧 Building |
| `Phenotype.Skills` | NuGet | C# | ⏳ Planned |
| `phenotype-skills` | PyPI | Python | ⏳ Planned |
| `@phenotype/skills` | npm | TypeScript | ⏳ Planned |

## Quick Start

### Rust

```bash
cargo add phenotype-skills
```

```rust
use phenotype_skills::{SkillRegistry, SkillManifest};

let registry = SkillRegistry::new();
let manifest = SkillManifest::from_file("./my-skill.toml")?;
registry.register(manifest)?;
```

### C#

```bash
dotnet add package Phenotype.Skills
```

```csharp
using Phenotype.Skills;

var registry = new SkillRegistry();
var manifest = SkillManifest.FromFile("./my-skill.toml");
registry.Register(manifest);
```

## License

MIT


## Traceability

/// @trace SKILL-001
/// @trace SKILL-002
/// @trace SKILL-003
/// @trace SKILL-004
/// @trace SKILL-005
/// @trace SKILL-006
/// @trace SKILL-007
/// @trace SKILL-008
