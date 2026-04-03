# PhenoConfig

Configuration management registry for the Phenotype ecosystem.

**See:** [POLYGLOT_ECOSYSTEM.md](./POLYGLOT_ECOSYSTEM.md) for cross-language strategy (TypeScript, Python 3.14+, Go, Rust)

## Structure

| Crate | Source | Purpose | Status |
|-------|--------|---------|--------|
| `pheno-config-core` | Settly | Layered configs, validation | ✅ Active |
| `pheno-config-ts` | phenotype-config-ts | TypeScript config | ✅ Active |

## Decision

~1,300 LOC - at threshold. Collect into registry to enable future language bindings.

## License

MIT
