# Portfolio Stabilization Plan — KooshaPari GitHub

**Date:** 2026-04-02
**Scope:** Full audit of 226 GitHub repos + local shelf state
**Status:** Ready for execution

---

## Executive Summary

Your GitHub portfolio has grown to **226 repositories** across 14 functional clusters. Only **~7% (15 repos)** are tracked by AgilePlus specs. The remaining **~211 repos** are untracked, creating massive fragmentation.

**Key findings:**
- 60+ repos actively pushed in last 7 days
- 25 single-commit stub repos (created same day, never developed)
- 16 archived repos
- 19 private repos
- 7 of 9 local repos are on feature branches (not main)
- 70+ GitHub repos have no local clone
- AgilePlus DB has 2 orphaned entries

---

## Cluster Inventory

### CLUSTER 1: Core Platform (Active, 6 repos) ✅ Partially Covered
| Repo | Language | Last Pushed | Spec Coverage |
|------|----------|-------------|---------------|
| thegent | Python | 2026-04-02 | ✅ 007-thegent-completion |
| AgilePlus | Rust | 2026-04-02 | ✅ 001-spec-driven-dev |
| phenotype-infrakit | Rust | 2026-04-02 | ⚠️ Via 002 (indirect) |
| phenotype-xdd | — | 2026-03-29 | ❌ None |
| phenotype-design | CSS | 2026-03-31 | ❌ None |
| phenotypeActions | Shell | 2026-03-31 | ❌ None |

### CLUSTER 2: Agent Framework (Active, 8 repos) ⚠️ Partially Covered
| Repo | Language | Last Pushed | Spec Coverage |
|------|----------|-------------|---------------|
| heliosCLI | Rust | 2026-04-02 | ✅ 006-helioscli-completion |
| heliosApp | TypeScript | 2026-04-01 | ✅ 005-heliosapp-completion |
| helMo | Shell | 2026-04-01 | ❌ None |
| Agentora | Rust | 2026-04-01 | ❌ None |
| AgentMCP | Python | 2026-04-01 | ❌ None |
| agent-wave | Shell | 2026-04-02 | ❌ None |
| agent-devops-setups | Python | 2026-03-29 | ❌ None |
| agentops-policy-federation | Python | 2026-04-02 | ❌ None |

### CLUSTER 3: CLI Tools (Active, 7 repos) ❌ Not Covered
| Repo | Language | Last Pushed | Notes |
|------|----------|-------------|-------|
| cliproxyapi-plusplus | Go | 2026-04-01 | LLM proxy, 8+ providers |
| agentapi-plusplus | Go | 2026-04-01 | HTTP API for CLI agents |
| Cmdra | Rust | 2026-03-29 | Universal CLI framework |
| forgecode | Shell | 2026-04-01 | Git workflow framework |
| thegent-sharecli | — | 2026-03-31 | CLI share system |
| thegent-cli-share | Python | 2026-03-28 | CLI dedup/merge |
| thegent-subprocess | Rust | 2026-03-28 | Subprocess mgmt |

### CLUSTER 4: Infrastructure (Active, 19 repos) ❌ Not Covered
| Repo | Language | Last Pushed | Notes |
|------|----------|-------------|-------|
| phenotype-go-kit | Go | 2026-03-29 | Go infra toolkit |
| phenotype-config | Rust | 2026-03-30 | Config management (private) |
| phenotype-shared | Rust | 2026-03-30 | Shared types/traits |
| phenotype-gauge | Rust | 2026-03-29 | Gauge/metrics |
| phenotype-nexus | Rust | 2026-03-29 | Nexus library |
| phenotype-forge | Rust | 2026-03-29 | Forge library |
| phenotype-cipher | Rust | 2026-03-29 | Cipher library |
| phenotype-xdd-lib | Rust | 2026-03-29 | xDD utilities |
| Authvault | Rust | 2026-04-01 | Auth framework |
| Tokn | Rust | 2026-03-29 | Token management |
| Zerokit | — | 2026-03-27 | ZK crypto utilities |
| PolicyStack | Python | 2026-03-29 | Policy scope stack |
| Quillr | TypeScript | 2026-03-29 | HTTP client |
| Httpora | — | 2026-03-27 | HTTP framework |
| Apisync | Rust | 2026-03-27 | API toolkit |
| phenotype-cli-core | Go | 2026-03-28 | Go CLI core |
| phenotype-middleware-py | Python | 2026-03-27 | Python middleware |
| phenotype-logging-zig | Zig | 2026-03-28 | Zig logging |
| phenotype-auth-ts | TypeScript | 2026-03-27 | TS auth patterns |

### CLUSTER 5: Observability (Active, 8 repos) ❌ Not Covered
| Repo | Language | Last Pushed | Notes |
|------|----------|-------------|-------|
| tracely | Rust | 2026-03-31 | Unified observability |
| thegent-metrics | Rust | 2026-04-01 | Agent metrics |
| thegent-shm | Rust | 2026-04-01 | Shared memory |
| helix-logging | Rust | 2026-03-27 | Structured logging |
| helix-tracing | Rust | 2026-03-27 | ⚠️ Archived |
| Tracera | JavaScript | 2026-04-01 | Tracking platform |
| Profila | Python | 2026-03-29 | Profiling toolkit |
| Phench | Python | 2026-03-29 | Benchmarking |

### CLUSTER 6: Hexagonal Templates (Stubs, 14 repos) ❌ Not Covered
hexagon-rs, hexagon-ts, hexagon-python, hexagon-go, hexagon-cs, hexagon-zig, hexagon-rust, HexaGo, HexaPy, HexaType, Hexacore, hexagon-kotlin, hexagon-swift, hexagon-elixir, hexagon-java

### CLUSTER 7: Language Templates (Scaffold, 13 repos) ❌ Not Covered
template-lang-rust, template-lang-python, template-lang-typescript, template-lang-go (private), template-lang-zig, template-lang-swift, template-lang-kotlin, template-lang-mojo, template-lang-elixir-hex, template-domain-webapp, template-domain-service-api, template-program-ops, template-commons (private)

### CLUSTER 8: Plugin Systems (Active, 4 repos) ❌ Not Covered
agileplus-plugin-git, agileplus-plugin-sqlite, agileplus-plugin-core, thegent-plugin-host

### CLUSTER 9: Apps/Web (Active, 8 repos) ❌ Not Covered
heliosApp (covered by 005), cloud (Kilo-Org), koosha-portfolio, Parpoura (private), phenodocs, FixitGo, FixitRs, Dino, Tracera

### CLUSTER 10: External/Forks (Mixed, 9 repos) ❌ Not Covered
portage, colab, vibeproxy, aizen (archived), ccusage (archived), Planify, MCPForge, Synthia, Tossy

### CLUSTER 11: Archived (16 repos) 🟢 Low Priority
CLIProxyAPI (private), claude-code-flow, agentapi-deprec, BytePort-TestPortfolio, phenotype-colab-extensions (private), KaskMan, KWatch, thegent-cache, helix-tracing, vibe-kanban, slick-portfolio-svelte-5, slickport, chatta, marketplace-utils (private), ccusage

### CLUSTER 12: Private (19 repos) ❌ Not Covered
template-lang-go, template-commons, Schemaforge, Flagward, phenotype-docs-engine, phenotype-evaluation, phenotype-skills, Prismal, Cursora, phenotype-patch, phenotype-sentinel, phenotype-agent-core, phenotype-vessel, phenotype-config, Parpoura, Civis, phenotype-agents, Holdr, Flowra

### CLUSTER 13: Legacy/Odin (Stale, ~15 repos) 🟢 Low Priority
odin-weather, odin-todo, odin-etchasketch, odin-res, odin-landing, odin-recipes, odin-library, odin-TTT, odin-dash, Frostify (57 stars), NetWeave, agslag-dash, agslagtmp-2, agslag-tmp, v0-agslag-project, model-conductor-hub, hoohacks, 340-p2, 340P1, canvasApp, ssToCal-front, go-nippon, KVirtualStage

### CLUSTER 14: Single-Commit Stubs (25 repos) 🔴 Critical Triage
phenotype-rust-metrics, phenotype-rust-api, phenotype-rust-config, phenotype-rust-logging, phenotype-rust-cli, phenotype-cache, phenotype-validation, phenotype-ts-sdk, phenotype-go-sdk, phenotype-python-sdk, phenotype-agents (private), Skillforge, Conft (private), Ziglog (private), Pyron (private), Keyra (private), Configra (private), Hexagon (private), hexagon-rust, BytePort, Duple, Guardis (private), Flowra (private), Holdr (private), Seedloom (private)

---

## Stabilization Phases

### Phase 1: Reduce Noise (Week 1)
**Goal:** Eliminate 40+ repos of noise to focus on what matters

1. **Archive single-commit stubs** (Cluster 14) — 25 repos
   - Decision: build or archive each
   - Most should be archived (they're placeholders)
   
2. **Archive legacy projects** (Cluster 13) — ~15 repos
   - Odin projects are learning exercises, archive
   - Frostify has 57 stars — keep public but mark archived
   - agslag/model-conductor — archive

3. **Clean up DB orphans** — 2 entries
   - snyk-phase-1-deploy (no spec dir)
   - 008-temporal-deployment-workflow (no spec dir)

**Expected outcome:** ~170 active repos (down from 226)

### Phase 2: Core Infrastructure (Week 2-3)
**Goal:** Stabilize the foundation everything else depends on

1. **phenotype-infrakit consolidation** — merge scattered crates
2. **Observability stack** — tracely, thegent-metrics, helix-logging
3. **Plugin system** — agileplus-plugin-*, thegent-plugin-host
4. **Auth stack** — Authvault, phenotype-auth-ts, Zerokit

### Phase 3: Agent Framework (Week 3-4)
**Goal:** Complete agent orchestration capabilities

1. **Agentora** — agent framework completion
2. **AgentMCP** — MCP protocol
3. **agent-wave** — event-driven communication
4. **helMo** — agent mobility
5. **agentops-policy-federation** — policy distribution

### Phase 4: CLI Tools (Week 4-5)
**Goal:** Complete CLI tooling ecosystem

1. **cliproxyapi-plusplus** — LLM proxy
2. **agentapi-plusplus** — agent API
3. **Cmdra** — CLI framework
4. **forgecode** — git workflows
5. **Deduplicate** — thegent-sharecli vs thegent-cli-share

### Phase 5: Template Cleanup (Week 5-6)
**Goal:** Consolidate 27 template repos into manageable set

1. **Merge hexagon-* + Hexa*** — eliminate duplicates
2. **Consolidate template-lang-*** — single template generator
3. **Document remaining templates**

### Phase 6: Apps & External (Week 6-7)
**Goal:** Complete or archive remaining apps

1. **Portfolio sites** — koosha-portfolio, Parpoura
2. **Fixit tools** — FixitGo, FixitRs
3. **External repos** — decide keep/archive/integrate
4. **Private repo sync** — catalog and map to public equivalents

---

## AgilePlus Spec Creation Plan

### New Specs to Create (in priority order)

| Spec ID | Title | Clusters Covered | WP Count |
|---------|-------|-----------------|----------|
| 012 | `github-portfolio-triage` | 13, 14 (legacy + stubs) | 4 |
| 013 | `phenotype-infrakit-stabilization` | 4 (infrastructure) | 5 |
| 014 | `observability-stack-completion` | 5 (observability) | 6 |
| 015 | `plugin-system-completion` | 8 (plugins) | 4 |
| 016 | `agent-framework-expansion` | 2 (agents) | 6 |
| 017 | `cli-tools-consolidation` | 3 (CLI) | 6 |
| 018 | `template-repo-cleanup` | 6, 7 (templates) | 5 |
| 019 | `private-repo-catalog` | 12 (private) | 5 |
| 020 | `portfolio-and-web-apps` | 9, 10 (apps) | 5 |

### Existing Specs to Update

| Spec ID | Current State | Action |
|---------|--------------|--------|
| 007-thegent-completion | in_progress | Continue, add subprocess/sharecli WPs |
| 006-helioscli-completion | specified | Add harness stabilization WPs |
| 005-heliosapp-completion | specified | Add federation WPs |
| 002-org-wide-release-governance | validated | Expand to cover all 226 repos |

---

## Local ↔ Remote Sync Plan

### Repos to Push to GitHub (local only, no remote)
- phenotype-infrakit/ (tracked by shelf, needs own remote)
- agileplus-agents/ (no git, needs init + push)
- agileplus-mcp/ (no git, needs init + push)
- phenotype-router-monitor/ (no git, needs init + push)

### Repos to Return to Main Branch
- heliosApp: feat/fix-typescript-vite-federation → main
- agent-wave: chore/integrate-phenotype-docs → main
- agentapi-plusplus: feat/chromatic-visual-testing → main
- heliosCLI: refactor/decouple-harness-crates → main
- thegent: refactor/cleanup-error-variants → main
- cliproxyapi-plusplus: feat/kilo-gastown-spec-and-sast → main

### Repos to Clone Locally (GitHub only, no local)
Priority clones (active repos):
- Hexacore, phenotype-xdd, phenotype-xdd-lib, phenotype-forge, phenotype-cipher
- Authvault, Tokn, Zerokit, PolicyStack, Quillr, Httpora, Apisync
- tracely, thegent-metrics, thegent-shm, helix-logging, Tracera, Profila, Phench
- Agentora, AgentMCP, agent-wave, agent-devops-setups, agentops-policy-federation
- Cmdra, forgecode, thegent-sharecli, thegent-cli-share
- agileplus-plugin-git, agileplus-plugin-sqlite, agileplus-plugin-core
- phenodocs, FixitGo, FixitRs, Dino

---

## Metrics & Success Criteria

| Metric | Current | Target (30 days) |
|--------|---------|-----------------|
| Total repos | 226 | ≤170 |
| Spec coverage | 7% | ≥80% |
| Repos on main branch | 2/9 | 9/9 |
| Single-commit stubs | 25 | 0 |
| Archived repos | 16 | ≥40 |
| AgilePlus DB entries | 9 (2 orphaned) | 20+ (0 orphaned) |
| Local clones of active repos | 9 | ≥40 |

---

## Execution Commands

```bash
# Phase 1: Create specs for triage
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
agileplus specify --title "GitHub Portfolio Triage" --description "Archive stale repos, triage single-commit stubs, clean up legacy projects"
agileplus specify --title "Phenotype Infrakit Stabilization" --description "Consolidate infrastructure crates, stabilize API surfaces"
agileplus specify --title "Observability Stack Completion" --description "Complete tracely, thegent-metrics, helix-logging, Profila, Phench"

# Phase 2: Create remaining specs
agileplus specify --title "Plugin System Completion" --description "Complete agileplus-plugin-* and thegent-plugin-host"
agileplus specify --title "Agent Framework Expansion" --description "Complete Agentora, AgentMCP, agent-wave, agent-devops-setups"
agileplus specify --title "CLI Tools Consolidation" --description "Complete cliproxyapi-plusplus, agentapi-plusplus, Cmdra, deduplicate CLI tools"
agileplus specify --title "Template Repo Cleanup" --description "Consolidate hexagon-* and template-lang-* repos"
agileplus specify --title "Private Repo Catalog" --description "Catalog and map 19 private repos to public equivalents"
agileplus specify --title "Portfolio and Web Apps" --description "Complete koosha-portfolio, Parpoura, phenodocs, Fixit tools"
```
