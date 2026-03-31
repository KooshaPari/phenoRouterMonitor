---
audience: [developers, pms]
---

# Constitution

The project constitution is a living governance document that captures technical standards, architectural principles, quality expectations, and process rules for the project. It guides all development decisions and serves as the enforcement baseline for code review and acceptance.

## What It Is

A living document at `.kittify/memory/constitution.md` (git-tracked) that defines:

- **Code quality standards** — linting, formatting, test coverage targets
- **Architecture principles** — design patterns, crate organization, dependency rules
- **Governance rules** — approval authority, merge policies, release process
- **Agent constraints** — what AI agents can/cannot modify, guardrails
- **Naming conventions** — branch names, commit messages, file paths, ID formats
- **Documentation requirements** — what must be documented and how
- **Performance targets** — response times, throughput, memory expectations

## Creating a Constitution

Initialize an interactive discovery process:

```bash
agileplus constitution --init
```

```
Creating project constitution...

This will ask about your project's standards and practices.

Language/Framework (select one):
  1. Rust
  2. Python
  3. TypeScript/JavaScript
  4. Go
  5. Other

Your choice: 1

Testing approach:
  1. Unit tests + integration tests
  2. BDD with Cucumber/Gherkin
  3. Property-based testing
  4. All of the above

Your choice: 1

Code review requirements:
  1. Approvals required: 1 (any maintainer)
  2. Approvals required: 2 (min 1 architect)
  3. Approvals required: Custom

Your choice: 1

Continuing...

Generated constitution at: .kittify/memory/constitution.md
Review and customize:
  agileplus constitution --edit
```

## Example Constitution

A complete constitution for an auth service:

```markdown
# AgilePlus Auth Service — Project Constitution

## Code Quality Standards

### Rust Toolchain
- Minimum Rust version: 1.75 (specified in `rust-toolchain.toml`)
- Formatter: `cargo fmt --check` (enforced in CI)
- Linter: `cargo clippy -- -D warnings` (no warnings allowed)
- No `unwrap()` or `panic!()` in library code
  → Use `Result`, `Option`, and `?` operator
  → Panics only allowed in tests and binary entry points

### Test Coverage
- Minimum coverage per crate: 85%
- Coverage measured by `cargo tarpaulin`
- All public API must have at least one test
- Happy path + error cases required
- Integration tests for cross-crate flows

### Code Organization
- Maximum function length: 50 lines (soft guideline, can exceed for complex logic)
- Maximum module file length: 400 lines (refactor if larger)
- No circular dependencies between crates
- Private modules use leading underscore: `mod _internal;`

### Documentation
- All public items must have rustdoc comments
- Examples in docs for non-trivial functions
- CHANGELOG.md updated for all user-facing changes

## Architectural Principles

### Port-Based Design
- All external I/O (database, network, file) goes through port traits
- Domain logic in core crate has zero external dependencies
- Ports live in separate crates: `*-ports`, `*-adapters`

### Crate Boundaries
```
agileplus-core/       # Domain model, entities, value objects
  ├── models/         # User, Feature, Spec, Plan
  ├── entities/       # WorkPackage, Requirement
  └── error.rs        # Error types

agileplus-engine/     # Business logic, orchestration
  ├── spec/           # Spec parsing, validation
  ├── plan/           # Planning algorithm
  └── dispatch/       # Agent dispatch logic

agileplus-ports/      # Trait definitions
  ├── storage.rs      # StoragePort trait
  ├── vcs.rs          # VcsPort trait
  └── agent.rs        # AgentPort trait

agileplus-adapters/   # Implementations
  ├── file_storage/   # FileStoragePort
  ├── git_vcs/        # GitVcsPort
  └── claude_agent/   # ClaudeAgentPort
```

### Dependency Rules
- ✗ Never import from adapters into core
- ✗ Never import from CLI into engine
- ✓ Core → Ports (always safe)
- ✓ Ports → Adapters (always safe)
- ✓ Engine → Core + Ports (always safe)

## Governance Rules

### Code Review
- All PRs require 1 approval from a maintainer
- Breaking API changes require 2 approvals
- Governance file changes (constitution, CLAUDE.md) require 2 approvals
- Agent-generated code uses automated review; humans can veto

### Commit Messages
Format: `type(scope): description`

```
feat(spec): add validation for required fields
fix(plan): handle circular work package dependencies
docs(guide): add getting-started section
refactor(engine): extract planning logic to separate module
test(integration): add end-to-end workflow tests
chore(deps): bump serde to 1.0.200
```

Types: `feat`, `fix`, `docs`, `refactor`, `test`, `chore`

### Merge Policy
- Merge commits preferred: `git merge --no-ff feat/xyz`
- Branch deleted after merge
- Main branch always deployable (CI must pass)
- Hotfixes branch from latest release tag

### Release Process
1. Update version in `Cargo.toml` and `package.json`
2. Update `CHANGELOG.md` with user-facing changes
3. Create git tag: `git tag v0.2.0`
4. Push tag to trigger release workflow
5. Release artifacts available on GitHub Releases

## Agent Constraints

### What Agents CAN Do
- ✓ Create/modify source code in `src/`
- ✓ Write tests in `tests/`
- ✓ Update `Cargo.toml` (dependencies, metadata)
- ✓ Update documentation (docs/, README.md)
- ✓ Create new modules and public APIs
- ✓ Run `cargo test` locally
- ✓ Commit code with proper messages

### What Agents CANNOT Do
- ✗ Modify governance files: `.kittify/`, `CLAUDE.md`, `constitution.md`
- ✗ Execute system commands that affect production data
- ✗ Run database migrations outside the planned scope
- ✗ Modify code from other work packages without explicit merge
- ✗ Force-push or rewrite history
- ✗ Deploy or cut releases

### Configuration
```yaml
agent:
  allowed_paths:
    - "src/"
    - "tests/"
    - "docs/"
    - "Cargo.toml"
    - "README.md"
  forbidden_paths:
    - ".kittify/"
    - "env_secrets/"
    - ".github/workflows/"
    - "CLAUDE.md"
    - "constitution.md"
  max_file_size: 1000000  # 1 MB
  max_commit_size: 5000000  # 5 MB
```

## Naming Conventions

### Branches
- Feature: `feat/{feature-id}-{short-slug}`
  - Example: `feat/001-oauth-auth`, `feat/WP02-login-endpoint`
- Fix: `fix/{issue-id}-{short-slug}`
  - Example: `fix/147-login-500-error`
- Docs: `docs/{topic}`
  - Example: `docs/getting-started`

### Commits
- Format: `type(scope): description`
- Scope is optional but recommended
- Capitalize description (Implement, not implement)
- Keep description under 72 characters
- Use imperative mood (Add, not Added)

### Files & Modules
- Rust files: `snake_case.rs`
- Modules: `pub mod snake_case;`
- Constants: `SCREAMING_SNAKE_CASE`
- Types: `PascalCase`
- Functions: `snake_case()`
- Test functions: `test_descriptive_name()`

### IDs
- Features: `{NNN}-{slug}` (e.g., `001-oauth-auth`)
- Work packages: `WP{NN}` within a feature (e.g., `WP01`, `WP02`)
- ADRs: `ADR-{NNN}` (e.g., `ADR-001`)
- Issues: GitHub issue numbers (e.g., `#147`)

## Performance Targets

### Response Times
- CLI commands: < 2 seconds for most operations
- Spec parsing: < 500ms for <5KB specs
- Plan generation: < 2 seconds for <20 work packages

### Test Execution
- Unit tests: < 30 seconds for full suite
- Integration tests: < 60 seconds
- Full test suite (unit + integration): < 90 seconds

### Database
- Spec queries: < 100ms (p99)
- Sync operations: < 5 seconds per issue tracker

## Updating the Constitution

Constitution updates are handled via pull request:

```bash
agileplus constitution --edit
# Opens $EDITOR to modify constitution.md
# When saved, creates a PR with the changes
```

All maintainers are notified. Requires 2 approvals to merge (ensures consensus).

Constitution changes take effect immediately on merge. New implementations must follow updated rules.

## Enforcement

The constitution is enforced at multiple points:

1. **CI/CD** — Linting, formatting, coverage checks
2. **Code Review** — Reviewers check compliance
3. **Automated Review** — Agent-generated code validated against constitution
4. **Status Dashboard** — Shows compliance metrics per feature

Track constitution compliance:

```bash
agileplus constitution --compliance
```

```
Constitution Compliance Report

Code Quality
  ✓ All code passes clippy (0 warnings)
  ✓ Test coverage: 86% (target: 85%)
  ✗ Function length: 3 violations (max 50 lines)
    → src/plan.rs:147 (73 lines)
    → src/dispatch.rs:89 (68 lines)

Architecture
  ✓ No circular dependencies
  ✓ Domain logic has zero external deps
  ✓ All I/O goes through ports

Governance
  ✓ All merges require 1+ approval
  ✓ Commit messages follow format
  ✓ Release process followed

Overall Compliance: 94% (3 minor violations)
