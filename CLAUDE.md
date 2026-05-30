CLAUDE.md — phenoRouterMonitor (phenotype-infrakit shelf)

## Project Overview
- **Name**: phenoRouterMonitor (repo name) / phenotype-infrakit (content name)
- **Owner**: KooshaPari
- **Stack**: Rust workspace (phenotype-infrakit crates + nested project dirs)

## Routing Canonical (OmniRoute direction)

`crates/bifrost-routing` and `crates/bifrost-routing-backup` are **deprecated orphan code** (no `Cargo.toml`, not in workspace). Canonical routing:

| Concern | Canonical Location |
|---|---|
| LLM proxy / runtime routing | [OmniRoute](https://github.com/KooshaPari/OmniRoute) (TypeScript) |
| Rust routing adapter + Pareto scoring | [Tokn](https://github.com/KooshaPari/Tokn) — `crates/tokenledger/src/routing/` |
| LLM cost tracking / ledger | [Tokn](https://github.com/KooshaPari/Tokn) — `crates/tokenledger/` + `crates/pareto-rs/` |
| Pareto dashboard (Streamlit) | [helios-router](https://github.com/KooshaPari/helios-router) |

Do **not** add new routing or cost logic to this repo. Use the canonical repos above.

## AgilePlus Mandate
All work MUST be tracked in AgilePlus:
- Reference: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- CLI: `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`

## Branch Discipline
- Feature branches: `worktrees/<topic>/`
- Canonical: `main`
- Never commit directly to `main`

## References
- Parent workspace: /Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md