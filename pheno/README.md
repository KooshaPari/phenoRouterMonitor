# pheno

Unified CLI for the Phenotype ecosystem.

## Overview

`pheno` is the single entry point for all Phenotype domain registries:
- **PhenoProc** - Process management
- **PhenoVCS** - Version control
- **PhenoPlugins** - Plugin system
- **Tracely** - Observability
- **Stashly** - Caching
- **HexaKit** - Templates

## Installation

```bash
cd ~/CodeProjects/Phenotype/repos/pheno
cargo build --release
# Binary at: target/release/pheno
```

## Usage

```bash
# Process management (PhenoProc)
pheno proc list
pheno proc start my-service
pheno proc stop 1234

# Version control (PhenoVCS)
pheno vcs list
pheno vcs create main ./worktrees/main
pheno vcs remove ./worktrees/feature

# Plugin management (PhenoPlugins)
pheno plugin list
pheno plugin load git
pheno plugin unload sqlite

# Observability (Tracely)
pheno trace start my-operation
pheno trace end abc123
pheno trace status

# Cache management (Stashly)
pheno cache get mykey
pheno cache set mykey myvalue
pheno cache clear

# Registry management
pheno registry list
pheno registry info PhenoProc
```

## Architecture

```
pheno CLI
├── commands/
│   ├── proc.rs      → PhenoProc registry
│   ├── vcs.rs       → PhenoVCS registry
│   ├── plugin.rs    → PhenoPlugins registry
│   ├── trace.rs     → Tracely registry
│   ├── cache.rs     → Stashly registry
│   └── registry.rs  → Registry introspection
├── Cargo.toml
│   └── Dependencies on all registries
└── main.rs
    └── Command routing
```

## Registry Dependencies

| Crate | Registry | Purpose |
|-------|----------|---------|
| `pheno-proc-core` | PhenoProc | ProcessPool, ManagedProcess |
| `pheno-proc-dedup` | PhenoProc | Command deduplication |
| `pheno-proc-queue` | PhenoProc | Priority task queue |
| `pheno-vcs-core` | PhenoVCS | Worktree management |
| `pheno-plugin-core` | PhenoPlugins | Plugin system |

## Development

Add new commands:
1. Create `src/commands/<domain>.rs`
2. Add to `src/commands/mod.rs`
3. Add to `main.rs` `Commands` enum
4. Wire in `match` statement

## License

MIT
