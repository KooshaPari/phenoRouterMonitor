# Portfolio Audit — Executive Summary

**Date:** 2026-04-02
**Agent:** Multi-agent audit (6 workers)
**Scope:** 226 GitHub repos + local shelf state

---

## The Numbers

| Metric | Count |
|--------|-------|
| **Total GitHub repos** | 226 |
| **Local git repos** | 9 |
| **Local non-git dirs** | 10 |
| **Archived on GitHub** | 16 |
| **Private on GitHub** | 19 |
| **Single-commit stubs** | 25 |
| **Active (pushed 7d)** | 60+ |
| **Spec coverage** | 7% → target 80% |

---

## What We Found

### 1. Massive Scaffold Burst (2026-03-25)
~25 repos created on the same day with single commits. These are skeleton repos (phenotype-rust-*, phenotype-*-sdk, hexagon-*, etc.) awaiting implementation. **Decision needed: build or archive each.**

### 2. 14 Functional Clusters
Your repos naturally group into 14 clusters:
- **Core Platform** (6 repos): thegent, AgilePlus, phenotype-infrakit
- **Agent Framework** (8 repos): heliosCLI, heliosApp, Agentora, AgentMCP
- **CLI Tools** (7 repos): cliproxyapi-plusplus, agentapi-plusplus, Cmdra
- **Infrastructure** (19 repos): phenotype-go-kit, Authvault, Tokn, Zerokit
- **Observability** (8 repos): tracely, thegent-metrics, helix-logging
- **Hexagonal Templates** (14 repos): hexagon-*, Hexa*
- **Language Templates** (13 repos): template-lang-*
- **Plugin Systems** (4 repos): agileplus-plugin-*
- **Apps/Web** (8 repos): heliosApp, phenodocs, FixitGo
- **External/Forks** (9 repos): portage, colab, Planify
- **Archived** (16 repos): CLIProxyAPI, KaskMan, etc.
- **Private** (19 repos): Schemaforge, Flagward, phenotype-*
- **Legacy/Odin** (~15 repos): Learning projects, 1+ year stale
- **Single-Commit Stubs** (25 repos): Created 2026-03-25

### 3. Local ↔ Remote Disconnect
- **7 of 9 local repos** are on feature branches, not main
- **70+ GitHub repos** have no local clone
- **phenotype-infrakit/** tracked by shelf repo, not independent git
- **cloud/** belongs to Kilo-Org, not KooshaPari

### 4. AgilePlus Gaps
- Only **7 of 226 repos** tracked by specs (3% coverage)
- **2 orphaned DB entries** (no spec directories)
- **9 new specs needed** (012-020) to cover remaining clusters

### 5. CI/CD Issues
- **3 workflows** have merge conflicts (security.yml, release.yml, tag-automation.yml)
- **CircleCI** severely outdated (Rust 1.75, Go 1.22)
- **Empty plugin directories** (thegent-plugin-host, vibeproxy-monitoring-unified)

---

## Stabilization Plan

### Phase 1: Reduce Noise (Week 1) — 40+ repos
1. Archive 25 single-commit stubs
2. Archive ~15 legacy Odin projects
3. Clean up 2 orphaned DB entries

### Phase 2: Core Infrastructure (Week 2-3)
1. phenotype-infrakit consolidation (19 crates)
2. Observability stack completion (8 repos)
3. Plugin system completion (4 repos)
4. Auth stack stabilization (Authvault, phenotype-auth-ts, Zerokit)

### Phase 3: Agent Framework (Week 3-4)
1. Agentora, AgentMCP, agent-wave completion
2. helMo, agentops-policy-federation
3. agent-devops-setups

### Phase 4: CLI Tools (Week 4-5)
1. cliproxyapi-plusplus, agentapi-plusplus
2. Cmdra, forgecode
3. Deduplicate thegent-sharecli vs thegent-cli-share

### Phase 5: Template Cleanup (Week 5-6)
1. Merge hexagon-* + Hexa* duplicates
2. Consolidate template-lang-* repos
3. Document remaining templates

### Phase 6: Apps & External (Week 6-7)
1. Portfolio sites, phenodocs, Fixit tools
2. External repos triage
3. Private repo catalog and sync

---

## New AgilePlus Specs Created

| Spec | Title | Clusters | WPs |
|------|-------|----------|-----|
| 012 | GitHub Portfolio Triage | 13, 14 | 4 |
| 013 | Phenotype Infrakit Stabilization | 4 | 5 |
| 014 | Observability Stack Completion | 5 | 6 |
| 015 | Plugin System Completion | 8 | 4 |
| 016 | Agent Framework Expansion | 2 | 6 |
| 017 | CLI Tools Consolidation | 3 | 6 |
| 018 | Template Repo Cleanup | 6, 7 | 5 |
| 019 | Private Repo Catalog | 12 | 5 |
| 020 | Portfolio and Web Apps | 9, 10 | 5 |

---

## Immediate Actions

1. **Review spec files** in `AgilePlus/kitty-specs/012-*` through `020-*`
2. **Run `agileplus plan`** for each new spec to generate work packages
3. **Start with Spec 012** (portfolio triage) — fastest way to reduce noise
4. **Fix CI/CD merge conflicts** in .github/workflows/
5. **Return local repos to main** branch before further work

---

## Files Created

- `docs/sessions/portfolio-audit-20260402/01_AUDIT.md` — Full audit report
- `docs/sessions/portfolio-audit-20260402/02_STABILIZATION_PLAN.md` — This file
- `AgilePlus/kitty-specs/012-github-portfolio-triage/spec.md`
- `AgilePlus/kitty-specs/013-phenotype-infrakit-stabilization/spec.md`
- `AgilePlus/kitty-specs/014-observability-stack-completion/spec.md`
- `AgilePlus/kitty-specs/015-plugin-system-completion/spec.md`
- `AgilePlus/kitty-specs/016-agent-framework-expansion/spec.md`
- `AgilePlus/kitty-specs/017-cli-tools-consolidation/spec.md`
- `AgilePlus/kitty-specs/018-template-repo-cleanup/spec.md`
- `AgilePlus/kitty-specs/019-private-repo-catalog/spec.md`
- `AgilePlus/kitty-specs/020-portfolio-and-web-apps/spec.md`
