<!-- AI-DD-META:START -->
<!-- This repository is planned, maintained, and managed by AI Agents only. -->
<!-- Slop issues are expected and intentionally present as part of an HITL-less -->
<!-- /minimized AI-DD metaproject of learning, refining, and building brute-force -->
<!-- training for both agents and the human operator. -->
![Downloads](https://img.shields.io/github/downloads/KooshaPari/phenoRouterMonitor/total?style=flat-square&label=downloads&color=blue)
![GitHub release](https://img.shields.io/github/v/release/KooshaPari/phenoRouterMonitor?style=flat-square&label=release)
![License](https://img.shields.io/github/license/KooshaPari/phenoRouterMonitor?style=flat-square)
![AI-Slop](https://img.shields.io/badge/AI--DD-Slop%20Expected-orange?style=flat-square)
![AI-Only-Maintained](https://img.shields.io/badge/Planned%20%26%20Maintained%20by-AI%20Agents%20Only-red?style=flat-square)
![HITL-less](https://img.shields.io/badge/HITL--less%20AI--DD-metaproject-yellow?style=flat-square)

> ⚠️ **AI-Agent-Only Repository**
>
> This repo is **planned, maintained, and managed exclusively by AI Agents**.
> Slop issues, rough edges, and AI artifacts are **expected and intentionally
> present** as part of an **HITL-less / minimized AI-DD** metaproject focused
> on learning, refining, and brute-force training both the agents and the
> human operator. Bug reports and contributions are still welcome, but please
> expect AI-generated code, comments, and documentation throughout.
<!-- AI-DD-META:END -->
> **Work state:** MISLABELED-SHELF · **Progress:** `███░░░░░░░ 30%`
> ⚠ Repo name is a MISNOMER — this is **Phenotype InfraKit** (shared infra crates), NOT a router monitor. ~11 crates duplicate the canonical kit [HexaKit](https://github.com/KooshaPari/HexaKit); canonical routing = [OmniRoute](https://github.com/KooshaPari/OmniRoute) (LLM) + [Tokn](https://github.com/KooshaPari/Tokn) `tokenledger::routing` (Rust). Pending dedup/rehome decision (dom-services-routing #24). · updated 2026-06-02

# Phenotype InfraKit

Shared infrastructure crates extracted from the Phenotype ecosystem.

This workspace contains generic infrastructure components that are shared across Phenotype services.

## Crates

- `phenotype-error-core`: Canonical error types for the Phenotype ecosystem
- `phenotype-git-core`: Phenotype git core crate
- `phenotype-health`: Shared health check abstraction for Phenotype services
- `phenotype-config-core`: Unified configuration loading and management for Phenotype ecosystem
- `phenotype-telemetry`: Telemetry and observability infrastructure
- `phenotype-validation`: Data validation infrastructure
- `phenotype-event-sourcing`: Append-only event store with SHA-256 hash chains
- `phenotype-cache-adapter`: Two-tier LRU + DashMap cache with TTL
- `phenotype-policy-engine`: Rule-based policy evaluation with TOML config
- `phenotype-state-machine`: Generic FSM with transition guards
- `phenotype-contracts`: Shared traits and types

## Usage

Add to your Cargo.toml:

```toml
[dependencies]
phenotype-error-core = { path = "../phenotype-infrakit/crates/phenotype-error-core" }
# ... other phenotype crates
```

## License

MIT
