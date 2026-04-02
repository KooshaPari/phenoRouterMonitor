# Template Stabilization Plan
**Date**: 2026-04-02
**Status**: Draft

## Objective

Complete and stabilize all Phenotype template repositories by implementing missing organizational infrastructure and practices.

## Scope

Templates requiring stabilization:
1. template-lang-kotlin
2. template-lang-elixir-hex
3. template-lang-swift
4. template-lang-zig
5. template-lang-python
6. template-lang-mojo
7. template-program-ops
8. template-lang-typescript (partial gaps)

## Gap Categories

### Tier 1: Critical Organizational Files
These files are essential for agent operation and CI/CD:

| File | Purpose | Source Template |
|------|---------|----------------|
| AGENTS.md | Agent guidance and operating procedures | template-lang-rust |
| .pre-commit-config.yaml | Pre-commit hooks for quality gates | template-lang-rust |
| ADR.md | Architecture Decision Record template | template-lang-rust |
| VERSION | Semantic version file | template-lang-rust |
| CHANGELOG.md | Version history | template-lang-rust |

### Tier 2: Documentation & Requirements
| File | Purpose | Source Template |
|------|---------|----------------|
| FUNCTIONAL_REQUIREMENTS.md | Feature requirements tracking | template-lang-rust |
| PRD.md | Product Requirements Document | template-lang-rust |
| README.md | Project documentation | template-lang-rust |

### Tier 3: DevOps Infrastructure
| File | Purpose | Source Template |
|------|---------|----------------|
| .devcontainer/ | Development environment | template-lang-rust |
| .github/scripts/ | CI/CD helper scripts | template-lang-rust |
| .github/workflows/security-guard.yml | Security scanning | template-lang-rust |
| .github/workflows/security-guard-hook-audit.yml | Hook audit | template-lang-rust |
| .github/workflows/workflow-permissions.yml | Permission management | template-lang-rust |

### Tier 4: Domain-Specific Templates
| File | Purpose | Source Template |
|------|---------|----------------|
| .env.example | Environment template | template-domain-webapp |

## Implementation Order

### Phase 1: Core Templates (Rust, Go)
- [x] template-lang-rust (complete reference)
- [ ] template-lang-go (add FR, PRD)

### Phase 2: TypeScript Template
- [ ] template-lang-typescript (add ADR, .devcontainer)

### Phase 3: Domain Templates
- [ ] template-domain-webapp (complete)
- [ ] template-domain-service-api (add FR, PRD)

### Phase 4: Language Templates (Missing Tier 1-3)
- [ ] template-lang-kotlin
- [ ] template-lang-elixir-hex
- [ ] template-lang-swift
- [ ] template-lang-zig
- [ ] template-lang-python
- [ ] template-lang-mojo

### Phase 5: Program Ops Template
- [ ] template-program-ops

## Rollback Plan

If issues arise:
1. Git revert the specific template commit
2. Run `git stash` for uncommitted changes
3. Document issues in worklog

## Success Criteria

- All templates have complete AGENTS.md, .pre-commit-config.yaml, ADR.md, VERSION, CHANGELOG.md
- All templates pass `scripts/scaffold-smoke.sh`
- All templates have consistent .github/workflows structure
- All templates have complete documentation structure
