# PhenoPlugins

Plugin system registry for the Phenotype ecosystem.

## Overview

PhenoPlugins provides the foundation for extensible plugin architectures across Phenotype applications.

## Workspace Crates

| Crate | Purpose |
|-------|---------|
| `pheno-plugin-core` | Plugin traits, registry, error handling |
| `pheno-plugin-git` | Git VCS adapter plugin |
| `pheno-plugin-sqlite` | SQLite storage adapter plugin |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Plugin Host (AgilePlus, etc.)              │
│                                                              │
│   ┌─────────────────────────────────────────────────────┐   │
│   │              pheno-plugin-core                      │   │
│   │   • Plugin trait                                    │   │
│   │   • Registry                                        │   │
│   │   • Lifecycle management                            │   │
│   └─────────────────────────────────────────────────────┘   │
│                        │                                     │
│           ┌────────────┼────────────┐                       │
│           ▼            ▼            ▼                       │
│   ┌─────────────┐ ┌────────┐ ┌────────────┐                │
│   │pheno-plugin │ │pheno-  │ │pheno-      │                │
│   │   -git      │ │plugin- │ │plugin-     │                │
│   │             │ │sqlite  │ │{future}    │                │
│   └─────────────┘ └────────┘ └────────────┘                │
└─────────────────────────────────────────────────────────────┘
```

## Migrated From

- `agileplus-plugin-core/` → `pheno-plugin-core/`
- `agileplus-plugin-git/` → `pheno-plugin-git/`
- `agileplus-plugin-sqlite/` → `pheno-plugin-sqlite/`

## License

MIT
