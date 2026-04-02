# Projects Index — Phenotype Repos Shelf

**Last Updated**: 2026-04-02
**Total Local Repos**: 9
**Total GitHub Repos**: 247 (KooshaPari org)
**Disk Usage**: 89 GB (target: 20 GB after cleanup)

---

## Cloned Repos (Local)

| Repo | Language | Branch | Status | Disk | Last Commit |
|------|----------|--------|--------|------|-------------|
| [phenotype-infrakit](./) | Rust | `fix/http-client-core-simplify` | DIRTY (8 files) | 1.8 GB | 2026-04-01 |
| [AgilePlus](./AgilePlus) | Rust/Python | `main` | DIRTY (28 files) | 20 GB | 2026-04-01 |
| [thegent](./thegent) | Rust/Python/Go | `refactor/cleanup-error-variants` | DIRTY (4 files) | 8.1 GB | 2026-04-01 |
| [heliosCLI](./heliosCLI) | Rust | `refactor/decouple-harness-crates` | DIRTY (8 files) | 39 GB | 2026-04-01 |
| [heliosApp](./heliosApp) | TypeScript | `feat/fix-typescript-vite-federation` | DIRTY (3 files) | 120 MB | 2026-04-01 |
| [agentapi-plusplus](./agentapi-plusplus) | Go | `feat/chromatic-visual-testing` | DIRTY (1 file) | 764 MB | 2026-04-01 |
| [cliproxyapi-plusplus](./cliproxyapi-plusplus) | Go | `feat/kilo-gastown-spec-and-sast` | DIRTY (1 file) | 441 MB | 2026-04-01 |
| [cloud](./cloud) | TypeScript | `main` | DIRTY (2 files) | 2.7 GB | 2026-04-01 |
| [agent-wave](./agent-wave) | Shell | `chore/integrate-phenotype-docs` | CLEAN | 3.4 MB | 2026-03-31 |
| [forgecode](./forgecode) | Shell | `main` | CLEAN | 165 MB | 2026-04-01 |

---

## GitHub-Only Repos (Not Cloned)

See [docs/stabilization/STRATEGY.md](../docs/stabilization/STRATEGY.md) for full 247-repo classification.

### Core Ecosystem (GitHub)
- phenotype-go-kit, phenotype-shared, phenotype-config, phenotype-design
- phenotype-gauge, phenotype-nexus, phenotype-forge, phenotype-cipher
- thegent-plugin-host, thegent-sharecli, thegent-cache, thegent-subprocess
- heliosBench, heliosHarness, helMo
- agileplus-plugin-sqlite, agileplus-plugin-git, agileplus-plugin-core
- Agentora, Authvault, bifrost-extensions, phenoSDK, phenodocs, Tracera
- KodeVibeGo, PolicyStack, Profila, Phench, Kogito, Dino

### Templates & Kits
- template-commons, template-lang-typescript, template-domain-webapp
- Hexacore, HexaGo, HexaPy, HexaType, hexagon-go, hexagon-ts, hexagon-python, hexagon-cs

### Peripheral / Archive Candidates
- agentapi-deprec (deprecated), tehgent (typo), BytePort-TestPortfolio, Byteport-TestZip
- P2, Tokn, argisexec, acp
- odin-dash, odin-TTT, odin-library, odin-recipes, odin-weather, odin-todo, odin-restaurant, odin-Signup, odin-calc, odin-res
- FixitGo, FixitRs, router-docs, heliosBench, QuadSGM, Kogito, Tossy, Frostify, AppGen, TripleM

### Learning / Personal
- koosha-portfolio, KaskMan, dotfiles, vibeproxy, vibeproxy-monitoring-unified
- 340-p2, 340P1, hoohacks, ssToCal-front, canvasApp

---

## Worktrees

| Worktree | Branch | Last Activity | Status |
|----------|--------|---------------|--------|
| `.worktrees/feat/cache-adapter-impl` | (detached?) | 2026-03-31 | NEEDS INVESTIGATION |
| `.worktrees/feat/http-client-core-fixes` | feat/http-client-core-fixes | 2026-03-30 | Active |
| `.worktrees/feat/phenotype-crypto-complete` | feat/phenotype-crypto-complete-v2 | 2026-03-30 | Merge v2 branch |
| `.worktrees/docs/` | — | — | EMPTY (remove) |
| `.worktrees/infrastructure/` | — | — | EMPTY (remove) |
| `.worktrees/phenotype-errors/` | — | — | EMPTY (remove) |

---

## Stabilization Status

See [Spec 021](../AgilePlus/kitty-specs/021-polyrepo-ecosystem-stabilization/) for full stabilization plan.

### Phase 1 Progress (Days 1-7)
- [ ] P1.1: Close/merge 10 open PRs in phenotype-infrakit
- [ ] P1.2: Delete 8 obvious test/typo repos
- [ ] P1.3: Clean 22 GB build artifacts locally
- [ ] P1.4: Enforce .gitignore across 9 cloned repos
- [ ] P1.5: Set up org-level .github repo with reusable workflows
- [ ] P1.6: Audit and enrich 35 AgilePlus specs
- [ ] P1.7: Establish worktree discipline
- [ ] P1.8: Run cargo fmt && cargo clippy on phenotype-infrakit
- [ ] P1.9: Commit all dirty files across 9 repos
- [ ] P1.10: Return canonical repos to main

### Phase 2-4: Planned
See [tasks.md](../AgilePlus/kitty-specs/021-polyrepo-ecosystem-stabilization/tasks.md) for full task list.

---

## Quick Reference

```bash
# Check if directory is a git repo
ls <dir>/.git 2>/dev/null && echo "GIT REPO" || echo "NOT A REPO"

# List worktrees
git worktree list

# Add worktree
git worktree add .worktrees/<name> -b <branch>

# Remove worktree
git worktree remove .worktrees/<name>

# Clean build artifacts
cargo clean
rm -rf */node_modules
rm -rf */.venv
rm -rf heliosCLI/bazel-*

# Run quality checks (phenotype-infrakit)
cargo fmt && cargo clippy --workspace -- -D warnings && cargo test --workspace
```
