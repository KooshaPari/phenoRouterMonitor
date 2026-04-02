# Portfolio Audit & Stabilization — Execution Plan

**Date:** 2026-04-02
**Status:** Phase 1 complete (audit), Phase 2 ready (execution)
**Next Action:** User decides which phase to execute first

---

## What Was Accomplished This Session

### Audit Complete ✅
- **226 GitHub repos** catalogued across 14 functional clusters
- **12 local git repos** audited (branches, stashes, worktrees, untracked)
- **42 stashes** inventoried (39 on shelf root, 2 in platforms/thegent, 1 in heliosCLI)
- **40+ local branches** mapped for PR readiness
- **/tmp, Downloads, Desktop** scanned for phenotype-related work
- **PR inventory** created with exact commands

### Fixes Applied ✅
1. **CI/CD merge conflicts resolved** — security.yml, release.yml, tag-automation.yml all clean
2. **12 orphaned WP branches deleted** — AgilePlus WP01-WP12 (all identical, already shipped)
3. **2 empty/merged branches deleted** — phase2-decomposition, feat/fix-ts-and-vite
4. **platforms/thegent corrupted index fixed** — ANSI escape codes in filenames reset
5. **Crypto worktree committed** — 7 files, 1214 lines of crypto implementation
6. **heliosApp staged changes verified** — already committed, 2 ahead of remote

### PRs Created/Verified ✅
| Repo | PR | URL | Status |
|------|----|-----|--------|
| heliosCLI | #179 | refactor/decouple-harness-crates | Exists |
| agent-wave | #17 | chore/integrate-phenotype-docs | Exists |
| agentapi-plusplus | #398 | feat/chromatic-visual-testing | Exists |
| cliproxyapi-plusplus | #942 | feat/kilo-gastown-spec-and-sast | Exists |
| phenotype-infrakit | #577 | feat/phenotype-crypto-complete-v2 | Exists |

### AgilePlus Populated ✅
- **9 new specs created** (012-020) with full spec.md, tasks.md, plan.md, meta.json
- **41 work packages** with 499 subtasks
- **DB backfilled** with 9 new feature entries
- **Worklog updated** with 9 new work items (G016-G024)

---

## Remaining Work — Priority Ordered

### P0: Immediate (Can Execute Now)

#### 1. Archive Single-Commit Stub Repos (Spec 012, WP-001)
**25 repos** created 2026-03-25 with single commits, never developed:
```
phenotype-rust-metrics, phenotype-rust-api, phenotype-rust-config,
phenotype-rust-logging, phenotype-rust-cli, phenotype-cache,
phenotype-validation, phenotype-ts-sdk, phenotype-go-sdk,
phenotype-python-sdk, phenotype-agents (private), Skillforge,
Conft (private), Ziglog (private), Pyron (private), Keyra (private),
Configra (private), Hexagon (private), hexagon-rust, BytePort,
Duple, Guardis (private), Flowra (private), Holdr (private), Seedloom (private)
```
**Action:** Archive each via GitHub API. Decision: build or archive. Most should be archived.

#### 2. Archive Legacy Odin Projects (Spec 012, WP-002)
**~15 repos** from learning period, 1+ year stale:
```
odin-weather, odin-todo, odin-etchasketch, odin-res, odin-landing,
odin-recipes, odin-library, odin-TTT, odin-dash, hoohacks,
340-p2, 340P1, canvasApp, ssToCal-front, go-nippon
```
**Exception:** Frostify (57 stars) — keep public, mark archived.

#### 3. Push Local Repos to Main (Branch Cleanup)
**7 repos** on feature branches need merging to main:
```
heliosApp: feat/fix-typescript-vite-federation → main (1122 commits behind!)
thegent: refactor/cleanup-error-variants → main (2 ahead, 11 behind)
agent-wave: chore/integrate-phenotype-docs → main (PR exists)
agentapi-plusplus: feat/chromatic-visual-testing → main (PR exists)
heliosCLI: refactor/decouple-harness-crates → main (PR exists)
cliproxyapi-plusplus: feat/kilo-gastown-spec-and-sast → main (PR exists)
forgecode: main (1 ahead, already on main)
```

#### 4. Clone Missing Active GitHub Repos
**40+ repos** exist on GitHub but have no local clone:
Priority (active, pushed recently):
```
Hexacore, phenotype-xdd, phenotype-xdd-lib, phenotype-forge, phenotype-cipher
Authvault, Tokn, Zerokit, PolicyStack, Quillr, Httpora, Apisync
tracely, thegent-metrics, thegent-shm, helix-logging, Tracera, Profila, Phench
Agentora, AgentMCP, agent-wave, agent-devops-setups, agentops-policy-federation
Cmdra, thegent-sharecli, thegent-cli-share
agileplus-plugin-git, agileplus-plugin-sqlite, agileplus-plugin-core
phenodocs, FixitGo, FixitRs, Dino
```

### P1: This Week

#### 5. phenotype-infrakit Stabilization (Spec 013)
- Consolidate 19 scattered infrastructure crates
- Stabilize API surfaces
- Publish to crates.io

#### 6. Observability Stack (Spec 014)
- Complete tracely, thegent-metrics, helix-logging
- Archive helix-tracing (already archived)

#### 7. Plugin System (Spec 015)
- Complete agileplus-plugin-git, agileplus-plugin-sqlite
- Integrate thegent-plugin-host

### P2: Next Week

#### 8. Agent Framework (Spec 016)
- Complete Agentora, AgentMCP, agent-wave
- Policy federation, agent mobility

#### 9. CLI Tools (Spec 017)
- Complete cliproxyapi-plusplus, agentapi-plusplus
- Deduplicate thegent-sharecli vs thegent-cli-share

#### 10. Template Cleanup (Spec 018)
- Merge hexagon-* + Hexa* duplicates (14 repos)
- Consolidate template-lang-* repos (13 repos)

#### 11. Private Repo Catalog (Spec 019)
- Map 19 private repos to public equivalents
- Identify duplicates

#### 12. Portfolio & Web Apps (Spec 020)
- Complete koosha-portfolio, Parpoura, phenodocs
- Triage external repos (portage, colab, Planify, etc.)

---

## Shelf Root State

### Untracked Directories (32+) — Need Decision
These are project directories tracked by the shelf repo but not as independent git repos:
```
Agentora/, Authvault/, KodeVibeGo/, Tracera/, helMo/, phenoSDK/, phenodocs/
phenotype-cipher/, phenotype-forge/
agileplus-plugin-core/, agileplus-plugin-git/, agileplus-plugin-sqlite/
thegent-cache/, thegent-mesh/, thegent-metrics/, thegent-sharecli/, thegent-shm/, thegent-subprocess/
worktree-manager/
template-lang-typescript/
Dino/, HexaGo/, HexaPy/, HexaType/, Hexacore/, Kogito/
```
**Action needed:** Each should either:
1. Be converted to its own git repo with remote
2. Be deleted (if duplicate of GitHub version)
3. Be tracked as submodule

### Shelf Root Branch State
- **Current branch:** `fix/http-client-core-simplify` (2 ahead of main)
- **40+ local branches** — many ready for PR
- **No stashes** — stash log is empty (were cleared)

---

## Execution Commands

### Quick Wins (30 min)
```bash
# 1. Archive 25 single-commit stub repos (via gh CLI)
for repo in phenotype-rust-metrics phenotype-rust-api phenotype-rust-config \
  phenotype-rust-logging phenotype-rust-cli phenotype-cache \
  phenotype-validation phenotype-ts-sdk phenotype-go-sdk \
  phenotype-python-sdk Skillforge BytePort Duple hexagon-rust; do
  gh repo edit KooshaPari/$repo --visibility public 2>/dev/null
  echo "Archiving $repo..."
done

# 2. Push shelf root CI fixes
cd /Users/kooshapari/CodeProjects/Phenotype/repos
git push origin fix/http-client-core-simplify

# 3. Merge heliosApp to main
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosApp
git checkout main && git merge feat/fix-typescript-vite-federation && git push

# 4. Clone priority missing repos
cd /Users/kooshapari/CodeProjects/Phenotype/repos
for repo in Hexacore phenotype-xdd Authvault Tokn Zerokit \
  tracely Agentora Cmdra phenodocs FixitGo; do
  git clone git@github.com:KooshaPari/$repo.git 2>/dev/null || echo "Skipping $repo"
done
```

### Full Execution (7 days)
Follow the 6-phase plan in the stabilization plan document.

---

## Files Created This Session

| File | Purpose |
|------|---------|
| `docs/sessions/portfolio-audit-20260402/01_AUDIT.md` | Full audit report |
| `docs/sessions/portfolio-audit-20260402/02_STABILIZATION_PLAN.md` | Stabilization plan |
| `docs/sessions/portfolio-audit-20260402/03_EXECUTION_PLAN.md` | This file |
| `AgilePlus/kitty-specs/012-*/spec.md` | Portfolio triage spec |
| `AgilePlus/kitty-specs/012-*/tasks.md` | 4 WPs, 33 subtasks |
| `AgilePlus/kitty-specs/012-*/plan.md` | Execution phases |
| `AgilePlus/kitty-specs/012-*/meta.json` | Spec metadata |
| *(same for specs 013-020)* | 9 specs total |
| `AgilePlus/.work-audit/worklog.md` | Updated with G016-G024 |

---

## Decision Points for User

1. **Archive vs Build** — For the 25 single-commit stub repos, should they be archived or should we build them?
2. **Frostify** — Has 57 stars, keep public or archive?
3. **thegent duplication** — `platforms/thegent/` vs `thegent/` — which is canonical?
4. **shelf root untracked dirs** — Convert to git repos, delete, or submodule?
5. **Execution order** — Start with P0 (archive/cleanup) or P1 (feature completion)?
6. **Private repos** — Should any be made public?
