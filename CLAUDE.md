# CLAUDE.md — repos shelf root

**This project is managed through AgilePlus.**

## AgilePlus Mandate

All work MUST be tracked in AgilePlus:
- Reference: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`
- CLI: `cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus && agileplus <command>`

## Work Requirements

1. **Check for AgilePlus spec before implementing**
2. **Create spec for new work**: `agileplus specify --title "<feature>" --description "<desc>"`
3. **Update work package status**: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. **No code without corresponding AgilePlus spec**

## Branch Discipline

- Feature branches in `repos/worktrees/<project>/<category>/<branch>`
- Canonical repository tracks `main` only
- Return to `main` for merge/integration checkpoints

## UTF-8 Encoding

All markdown files must use UTF-8.

---

## Shelf Identity

This is the **repos shelf** — a polyrepo containing ~175+ independent projects (now ~102 after consolidation).
A shelf is a top-level organizational unit containing related but independent
project repositories.

**NOT a single project.** Each subdirectory is an independent git repository.
Always identify which project you're working in before taking action.

## Repository Consolidation Status

### Completed Merges

| Registry | Merged From | Status |
|----------|-------------|--------|
| **hexagon** | hexagon-go, hexagon-rust, hexagon-zig, HexaKit/*, template-lang-* | ✅ Complete |
| **Kogito** | bifrost-extensions | ✅ Complete |
| **thegent** | thegent-mesh, thegent-plugin-host, thegent-shm, thegent-subprocess | ✅ Intentionally decomposed |
| **Tracely** | Traceon, helix-logging, helix-tracing | ✅ Absorbed |
| **Stashly** | thegent-cache | ✅ Merged (singleflight) |
| **PhenoConfig** | Settly, phenotype-config, Configra | ✅ Complete |
| **phenotype-infrakit** | thegent-crypto, fs, jsonl, utils, resources, path | ✅ Extracted 6 crates |
| **Tasken** | phenotype-task-engine | ✅ Referenced |
| **agentapi-plusplus** | agentapi, agentapi-deprec, agentapi-temp | ✅ Merged |
| **cliproxyapi-plusplus** | CLIProxyAPI | ✅ Merged |

### Pheno* Naming
- **GitHub**: Uses `pheno*` (camelCase) - e.g., phenoCipher, phenoGoKit
- **Local**: Empty `phenotype-*` shells deleted (18 removed)

### Current Counts
- **Total git repositories**: ~102
- **phenotype-infrakit crates**: 80
- **thegent crates**: 7 (agent-specific, intentional decomposition)
- **pheno* repositories**: 19 active

---

Use the repository `README.md`, `docs/index.md`, and `docs/sessions/` as the
canonical entry points for repo scope, active work, and resumable session-led
execution.

---

## Dependency Audit & 2026 Modernization

### Critical Updates (P0 - Security/Performance)
| Package | From | To | Projects |
|---------|------|----|----------|
| **chrono** (Rust) | 0.4 | **time** | All Rust projects |
| **env_logger** | 0.11 | **tracing** | All Rust projects |
| **reqwest** | 0.11 | **0.12** | API clients |
| **eslint** | 8.x | **9.x flat** | All TS projects |

### High Value (P1 - Major Versions)
| Package | From | To | Benefit |
|---------|------|----|---------|
| **anyhow** → **error-stack** | 1.0 | latest | Better error context |
| **vitest** | 1.0 | **3.0** | 3x faster tests |
| **ruff** | 0.2 | **0.11** | New Python checks |

### Custom → External Library Migrations
| Custom | Replacement | Effort |
|--------|-------------|--------|
| **thegent-crypto** | **ring** | 2 weeks |
| **phenoCipher** | **rustls** | 2 weeks |
| **thegent-gauge** | **criterion** | 1 week |

---

## CI Requirements

- Always evaluate and fix ALL CI check failures on a PR, including pre-existing failures inherited from main.
- Never dismiss a CI failure as "pre-existing" or "unrelated to our changes" — if it fails on the PR, fix it in the PR.
- This includes: build, lint, test, docs build, security scanning (CodeQL), code review gates (CodeRabbit), workflow guard checks, and any other CI jobs.
- When a failure is caused by infrastructure outside the branch (e.g., rate limits, external service outages), implement or improve automated retry/bypass mechanisms in CI workflows.
- After fixing CI failures, verify locally where possible (build, vet, tests) before pushing.
