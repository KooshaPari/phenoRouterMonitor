# ADR-015: Phenotype Crate Organization & PR Guidelines

## Status

Proposed - 2026-03-31

## Context

The phenotype-infrakit repository has grown to 30+ crates with accumulated technical debt:
- Large PRs (>100 files) block effective review
- Duplicate/error-handling crates (phenotype-error-core vs phenotype-errors)
- Mixed ownership (AgilePlus + Phenotype crates in same workspace)
- No clear architectural boundaries between domain/infrastructure/support

## Decision

### 1. Crate Categorization

```
phenotype-infrakit/
├── domain/           # Core business logic
│   ├── phenotype-contracts/      # Port/trait definitions
│   ├── phenotype-port-traits/    # Interface contracts
│   ├── phenotype-policy-engine/  # Business rules
│   └── phenotype-state-machine/  # State management
│
├── infrastructure/   # Technical capabilities
│   ├── phenotype-git-core/        # Git operations
│   ├── phenotype-cache-adapter/   # Caching
│   ├── phenotype-event-sourcing/  # Event sourcing
│   └── phenotype-http-client-core/
│
├── observability/   # Monitoring/tracing
│   ├── phenotype-telemetry/       # Metrics
│   ├── phenotype-health/          # Health checks
│   └── phenotype-logging/         # Logging
│
├── utility/         # Reusable helpers
│   ├── phenotype-error-core/      # Error types
│   ├── phenotype-iter/            # Iterator utils
│   ├── phenotype-string/           # String utils
│   ├── phenotype-crypto/          # Crypto ops
│   └── phenotype-time/            # Time utilities
│
└── validation/      # Input validation
    └── phenotype-validation/       # Validation rules
```

### 2. Crate Size Limits

- **Hard limit**: 500 lines per source file
- **Target**: 350 lines per source file
- **Recommended**: <10 files per crate
- **Recommended**: <20 crates per category

### 3. PR Size Guidelines

| Size | Files | Lines | Review Time |
|------|-------|-------|-------------|
| XS   | 1-5   | <100  | 5 min       |
| S    | 5-15  | <500  | 15 min      |
| M    | 15-50 | <2000 | 30 min      |
| L    | 50-100| <5000 | 60 min      |
| XL   | 100+  | 5000+ | Requires split |

### 4. PR Categories

```bash
# Feature PR
feat(<scope>): <description>

# Bug Fix
fix(<scope>): <description>

# Refactor (no behavior change)
refactor(<scope>): <description>

# Documentation
docs(<scope>): <description>

# Chore (tooling, deps)
chore(<scope>): <description>

# Test
test(<scope>): <description>
```

### 5. Stacked PR Process

```
origin/main
    ├── stack/domain-contracts (PR #1)
    │   └── stack/domain-state-machine (PR #2)
    │       └── stack/domain-policy (PR #3)
    │           └── stack/infrastructure-git (PR #4)
    └── stack/observability-telemetry (PR #5)
```

Commands:
```bash
# Create stacked PR
gh pr create --base stack/domain-contracts --head stack/domain-state-machine

# Update stacked PR
git fetch origin
git push --force-with-lease
```

### 6. Code Review Checklist

- [ ] PR size is appropriate (<100 files)
- [ ] Single responsibility per commit
- [ ] No breaking changes without migration path
- [ ] Tests pass locally
- [ ] Clippy warnings resolved
- [ ] Documentation updated if public API changed
- [ ] Architecture patterns followed (ports/adapters, etc.)

### 7. Conflict Resolution

Small PRs merge cleanly. For conflicts:
1. Rebase onto target branch
2. Resolve conflicts incrementally
3. Force-push with lease only to feature branches

## Consequences

### Positive
- Faster, more effective code reviews
- Clearer architectural boundaries
- Easier to understand codebase
- Reduced cognitive load

### Negative
- More PRs to manage
- Requires discipline to keep PRs small
- Stacked PRs need coordination

## References

- [CLAUDE.md](../CLAUDE.md) - Project guidelines
- [GOVERNANCE.md](../GOVERNANCE.md) - Decision making
- [ADR-001](../adr/ADR-001-architecture.md) - Initial architecture
