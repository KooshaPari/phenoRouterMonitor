# Project Stabilization Plan

**Version:** 1.0 | **Status:** Draft | **Date:** 2026-04-02

This plan addresses stabilization and completion of the project portfolio, organized by priority.

---

## Executive Summary

The portfolio contains **30+ projects** across multiple languages (Rust, Python, TypeScript, Go, Zig) with varying maturity levels:

| Category | Count | Status |
|----------|-------|--------|
| Active/Complete | ~15 | Main workspaces (AgilePlus, thegent, phenoSDK, etc.) |
| Standalone Libraries | 5 | Stashly, Settly, Tasken, Evalora, bare-cua |
| Template Projects | 11 | hexagon-*, needs consolidation |
| Stub/Archive | ~10 | Agent-generated, need review |

---

## Project Status Assessment

### Priority 1: Active Projects (No Action Needed)
| Project | Path | Status |
|---------|------|--------|
| AgilePlus | `AgilePlus/` | Production-ready, well-governed |
| thegent | `platforms/thegent/` | Active development |
| phenoSDK | `python/phenosdk/` | Sanitization complete |
| phenotype-infrakit | `crates/` | Complete, well-documented |
| phenotype-shared | `phenotype-shared/` | Active |

### Priority 2: Standalone Libraries (Stabilization Needed)

| Project | Language | Location | Needs |
|---------|----------|----------|-------|
| Stashly | Rust | `Stashly/` | CLAUDE.md, AGENTS.md, CI/CD |
| Settly | Rust | `Settly/` | CLAUDE.md, AGENTS.md, CI/CD |
| Tasken | Rust | `Tasken/` | CLAUDE.md, AGENTS.md, CI/CD |
| Evalora | Rust | `Evalora/` | CLAUDE.md, AGENTS.md, CI/CD |
| Docuverse | Go | `Docuverse/` | CLAUDE.md, AGENTS.md, CI/CD |
| bare-cua | Rust | `bare-cua/` | CLAUDE.md, AGENTS.md |

### Priority 3: Template Projects (Consolidation Needed)

| Project | Status | Action |
|---------|--------|--------|
| hexagon-rs | Stub | Merge into template-lang-rust |
| hexagon-ts | Stub | Merge into template-lang-typescript |
| hexagon-python | Stub | Merge into template-lang-python |
| hexagon-go | Stub | Merge into template-lang-go |
| hexagon-zig | Stub | Merge into template-lang-zig |
| hexagon-swift | Stub | Merge into template-lang-swift |
| hexagon-kotlin | Stub | Archive (low priority) |
| hexagon-java | Stub | Archive (low priority) |
| hexagon-elixir | Stub | Merge into template-lang-elixir-hex |
| hexagon-cs | Stub | Archive (low priority) |

### Priority 4: Archive Candidates

| Project | Reason |
|---------|--------|
| phenotype-rust-* | Stubs - absorbed into phenotype-infrakit |
| phenotype-ts-sdk | Consolidated into phenoSDK |
| phenotype-python-sdk | Consolidated into phenoSDK |
| Holdr | Container utilities - low priority |
| Cryptora | Cryptography - low priority |
| Servion | Service discovery - low priority |
| Guardrail | Rate limiting - low priority |
| Benchora | Benchmarking - low priority |
| Skillforge | Archive - superseded |
| Conft | Archive - superseded |
| Pyron | Archive - superseded |
| Keyra | Archive - superseded |

---

## Missing Infrastructure Components

### 1. Standardized Project Scaffolding

All projects should have:
```
├── CLAUDE.md          # Agent interaction rules
├── AGENTS.md          # Project-specific agent rules
├── README.md          # Project overview
├── STANDARDS.md       # Applied methodologies
├── CHANGELOG.md       # Version history
├── .github/
│   └── workflows/
│       └── ci.yml     # Standard CI pipeline
├── src/ or lib/       # Source code
└── tests/             # Test suite
```

### 2. Standardized CI/CD Pipeline

Standard GitHub Actions CI template provided for all Rust/Go projects.

### 3. Standardized AGENTS.md Template

See `template-program-ops/AGENTS.template.md` for project AGENTS.md template.

---

## Implementation Phases

### Phase 1: Core Infrastructure (Week 1)

| Task | Description | Status |
|------|-------------|--------|
| P1.1 | Create project scaffolding template | Pending |
| P1.2 | Add AGENTS.md to Stashly | Pending |
| P1.3 | Add AGENTS.md to Settly | Pending |
| P1.4 | Add AGENTS.md to Tasken | Pending |
| P1.5 | Add AGENTS.md to Evalora | Pending |
| P1.6 | Add AGENTS.md to Docuverse | Pending |
| P1.7 | Add AGENTS.md to bare-cua | Pending |

### Phase 2: CI/CD Standardization (Week 1-2)

| Task | Description | Status |
|------|-------------|--------|
| P2.1 | Add GitHub Actions CI to Stashly | Pending |
| P2.2 | Add GitHub Actions CI to Settly | Pending |
| P2.3 | Add GitHub Actions CI to Tasken | Pending |
| P2.4 | Add GitHub Actions CI to Evalora | Pending |
| P2.5 | Add GitHub Actions CI to Docuverse | Pending |
| P2.6 | Add GitHub Actions CI to bare-cua | Pending |

### Phase 3: Template Consolidation (Week 2)

| Task | Description | Status |
|------|-------------|--------|
| P3.1 | Archive hexagon-rs, use template-lang-rust | Pending |
| P3.2 | Archive hexagon-ts, use template-lang-typescript | Pending |
| P3.3 | Archive hexagon-python, use template-lang-python | Pending |
| P3.4 | Archive hexagon-go, use template-lang-go | Pending |
| P3.5 | Archive hexagon-zig, use template-lang-zig | Pending |
| P3.6 | Archive hexagon-swift, use template-lang-swift | Pending |

### Phase 4: Archive Cleanup (Week 2-3)

| Task | Description | Status |
|------|-------------|--------|
| P4.1 | Archive unused phenotype-* stubs | Pending |
| P4.2 | Archive skillforge, conft, pyron, keyra | Pending |
| P4.3 | Update project INDEX.md | Pending |

---

## Organizational Practices to Implement

### 1. Project Onboarding Checklist

Before a project is considered "complete":

- [ ] README.md with usage examples
- [ ] AGENTS.md with project rules
- [ ] CLAUDE.md with settings
- [ ] STANDARDS.md with applied methodologies
- [ ] CHANGELOG.md (even if empty)
- [ ] GitHub Actions CI workflow
- [ ] All tests pass locally
- [ ] Clippy passes with no warnings
- [ ] Code formatted with rustfmt (or language equivalent)

### 2. Shelf-Level Governance

Maintain at shelf level:
- `projects/INDEX.md` - Master project list
- `AGENTS.md` - Shelf agent rules
- `GOVERNANCE.md` - Governance policies
- `.github/` - Shared CI/CD templates

### 3. Quality Gates

All projects must pass before merge:

| Gate | Tool | Flags |
|------|------|-------|
| Tests | `cargo test` | Required |
| Lint | `cargo clippy` | `-D warnings` |
| Format | `cargo fmt` | `--check` |
| Security | `cargo audit` | No vulnerabilities |
| Docs | `cargo doc` | No warnings |

---

## Quick Wins

1. **Add CI to standalone projects** - 30 min each
2. **Create AGENTS.md templates** - Reuse across projects
3. **Archive hexagon-* stubs** - Redirect to templates
4. **Update INDEX.md** - Document actual state

---

## Success Criteria

- All active projects have complete scaffolding (AGENTS.md, CLAUDE.md, CI)
- All standalone libraries have CI/CD
- Template projects consolidated into template-*
- Archive candidates identified and marked
- Project INDEX.md accurate and current

---

## Appendix: Project Locations

| Project | Current Location | Notes |
|---------|-----------------|-------|
| Stashly | `Stashly/` | Symlink to remote-clones |
| Settly | `Settly/` | Symlink to remote-clones |
| Tasken | `Tasken/` | Symlink to remote-clones |
| Evalora | `Evalora/` | Symlink to remote-clones |
| Docuverse | `Docuverse/` | Symlink to remote-clones |
| bare-cua | `bare-cua/` | Symlink to remote-clones |
| hexagon-* | `remote-clones/` | Templates, need consolidation |
