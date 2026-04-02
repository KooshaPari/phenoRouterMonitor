# Phenotype Shelf - Project Index

**Last Updated**: 2026-04-02

This index catalogs all projects in the `repos/` shelf.

---

## Core Workspaces

| Project | Path | Type | Description |
|---------|------|------|-------------|
| phenotype-infrakit | `repos/` | Rust workspace | Core infrastructure crates |
| phenotype-infrakit-stubs | `.archive/phenotype-infrakit-stubs/` | Archived | Archived duplicate stubs |
| platforms | `platforms/` | Rust workspace | Generic parent workspace |
| thegent | `platforms/thegent/` | Python/Rust workspace | Agent orchestration platform |
| AgilePlus | `AgilePlus/` | Rust workspace | Agile project management |

---

## Platform Workspaces

| Project | Path | Description |
|---------|------|-------------|
| platforms/thegent | `platforms/thegent/` | Agent orchestration and governance |
| thegent-mesh | `thegent-mesh/` | Agent communication mesh |
| thegent-metrics | `thegent-metrics/` | Telemetry for agents |
| thegent-sharecli | `thegent-sharecli/` | CLI utilities |
| thegent-plugin-host | `thegent-plugin-host/` | Plugin system |

---

## CLI Tools

| Project | Path | Language |
|---------|------|----------|
| heliosCLI | `heliosCLI/` | Rust |
| clikit | `clikit/` | Rust |
| pheno-cli | `pheno-cli/` | Rust |

---

## Agent Frameworks

| Project | Path | Language |
|---------|------|----------|
| AgentMCP | `AgentMCP/` | Python |
| Agentora | `Agentora/` | Python |
| agent-api | `agentapi-plusplus/` | TypeScript |

---

## Infrastructure

| Project | Path | Description |
|---------|------|-------------|
| kits | `kits/` | Shared utilities |
| crates | `crates/` | phenotype-* crate library |
| proto | `proto/` | Protocol buffer definitions |
| templates | `templates/` | Project templates |
| harness | `harnesses/` | Test harnesses |

---

## Archived / Inactive

| Project | Path | Crates | Notes |
|---------|------|--------|-------|
| phenotype-stubs | `.archive/phenotype-stubs/` | 18 | Duplicate phenotype-* crate stubs |
| vibeproxy-monitoring | `.archive/vibeproxy-monitoring-unified-archived-2026-03-30/` | - | Embedded git repo (needs cleanup) |

---

## Notes

- All Rust projects should use `cargo check --workspace` before committing
- Python projects should have `pyproject.toml` at root
- Cross-platform dependencies go in `platforms/crates/`
- Product-specific crates stay in their respective directories
