# Project Stabilization Plan
**Created**: 2026-04-02
**Status**: Draft
**Shelf**: repos

## Executive Summary

This plan addresses stabilization and completion of ~35 projects in the Phenotype ecosystem. Analysis reveals:

- **16 projects** in `remote-clones/` need to be moved to `repos/`
- **14 projects** missing from local environment (need GitHub cloning)
- **~20 projects** need CI/CD workflows added
- **8 hexagon projects** need standardized documentation
- **3 archived projects** need proper archiving workflow

---

## Phase 1: Project Migration (remote-clones → repos)

### 1.1 Hexagon Architecture Templates

Move and integrate into `repos/`:

| Project | Source | Destination | Status |
|---------|--------|-------------|--------|
| hexagon-ts | remote-clones/hexagon-ts | repos/hexagon-ts | Move |
| hexagon-python | remote-clones/hexagon-python | repos/hexagon-python | Move |
| hexagon-rs | remote-clones/hexagon-rs | repos/hexagon-rs | Move |
| hexagon-zig | remote-clones/hexagon-zig | repos/hexagon-zig | Move |
| hexagon-java | remote-clones/hexagon-java | repos/hexagon-java | Move |
| hexagon-kotlin | remote-clones/hexagon-kotlin | repos/hexagon-kotlin | Move |
| hexagon-swift | remote-clones/hexagon-swift | repos/hexagon-swift | Move |
| hexagon-cs | remote-clones/hexagon-cs | repos/hexagon-cs | Move |
| hexagon-elixir | remote-clones/hexagon-elixir | repos/hexagon-elixir | Move |
| hexagon-go | remote-clones/hexagon-go | repos/hexagon-go | Move |

### 1.2 Infrastructure Libraries

| Project | Source | Destination | Status |
|---------|--------|-------------|--------|
| Logify | remote-clones/Logify | repos/Logify | Move |
| Eventra | remote-clones/Eventra | repos/Eventra | Move |
| Metron | remote-clones/Metron | repos/Metron | Move |
| Traceon | remote-clones/Traceon | repos/Traceon | Move |

### 1.3 Phenotype Libraries

| Project | Source | Destination | Status |
|---------|--------|-------------|--------|
| phenotype-auth-ts | remote-clones/phenotype-auth-ts | repos/phenotype-auth-ts | Move (marked archived) |
| phenotype-config-ts | remote-clones/phenotype-config-ts | repos/phenotype-config-ts | Move (marked archived) |
| Portalis | remote-clones/Portalis | repos/Portalis | Move |

### 1.4 Command

```bash
# Move projects (execute from repos root)
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Hexagon projects
for proj in hexagon-ts hexagon-python hexagon-rs hexagon-zig hexagon-java hexagon-kotlin hexagon-swift hexagon-cs hexagon-elixir hexagon-go; do
  git -C "remote-clones/$proj" remote set-url origin git@github.com:phenotype-dev/$proj.git 2>/dev/null || true
  mv "remote-clones/$proj" "./$proj"
  git add "./$proj"
done

# Infrastructure libraries
for proj in Logify Eventra Metron Traceon; do
  git -C "remote-clones/$proj" remote set-url origin git@github.com:phenotype-dev/$proj.git 2>/dev/null || true
  mv "remote-clones/$proj" "./$proj"
  git add "./$proj"
done

# Phenotype libraries
for proj in phenotype-auth-ts phenotype-config-ts Portalis; do
  git -C "remote-clones/$proj" remote set-url origin git@github.com:phenotype-dev/$proj.git 2>/dev/null || true
  mv "remote-clones/$proj" "./$proj"
  git add "./$proj"
done

git commit -m "chore: migrate projects from remote-clones to repos"
```

---

## Phase 2: CI/CD Infrastructure

### 2.1 Hexagon Projects (No CI → Needs CI)

Create `.github/workflows/ci.yml` for each:

#### TypeScript Hexagon (hexagon-ts)
```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
      - run: npm ci
      - run: npm run build
      - run: npm test
```

#### Python Hexagon (hexagon-python)
```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - run: pip install -e ".[dev]"
      - run: pytest
```

#### Rust Hexagon (hexagon-rs)
See existing Rust CI patterns in `.github/workflows/ci.yml`

#### Other Hexagons (Java, Kotlin, Swift, C#, Elixir, Zig, Go)
Create language-appropriate CI templates

### 2.2 Rust Infrastructure Libraries

**Logify** - No CI
**Eventra** - Has 1 workflow (review)
**Metron** - Has 1 workflow (review)
**Traceon** - Has 1 workflow (review)

Add full CI to all:
```yaml
# For Eventra, Metron, Traceon, Logify
name: CI
on:
  push:
    branches: [main]
  pull_request:

env:
  CARGO_TERM_COLOR: always
  RUSTFLAGS: "-D warnings"

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
        with:
          components: rustfmt, clippy
      - uses: Swatinem/rust-cache@v2
      - name: Format check
        run: cargo fmt --check
      - name: Clippy
        run: cargo clippy -- -D warnings
      - name: Test
        run: cargo test
      - name: Build
        run: cargo build
```

### 2.3 TypeScript Projects

**phenotype-auth-ts** - No CI (marked archived)
**phenotype-config-ts** - No CI (marked archived)
**Portalis** - No CI

Add CI:

#### Portalis (Python)
```yaml
name: CI
on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: '3.12'
      - run: pip install -e ".[dev]"
      - run: pytest
      - run: ruff check .
      - run: ruff format --check .
```

---

## Phase 3: Documentation Standardization

### 3.1 Projects Missing Documentation

| Project | README | CLAUDE.md | ADR.md | PRD.md | FR.md |
|---------|-------|-----------|--------|--------|-------|
| hexagon-ts | ✓ | ✓ | ✓ | ✓ | ✓ |
| hexagon-python | ✓ | ✓ | ✓ | ✓ | ✓ |
| hexagon-rs | ✓ | ✓ | ✓ | ✓ | ✓ |
| hexagon-cs | ✓ | ✓ | ✓ | ✓ | ✓ |
| hexagon-zig | ✓ | ✓ | ✓ | ✓ | ✓ |
| hexagon-java | ✓ | ✗ | ✗ | ✗ | ✗ |
| hexagon-kotlin | ✓ | ✗ | ✗ | ✗ | ✗ |
| hexagon-swift | ✓ | ✗ | ✗ | ✗ | ✗ |
| hexagon-elixir | ✓ | ✗ | ✗ | ✗ | ✗ |

### 3.2 Documentation Template for Missing Projects

Create `CLAUDE.md` for hexagon-java, hexagon-kotlin, hexagon-swift, hexagon-elixir:

```markdown
# CLAUDE.md — hexagon-<lang>

## Project Identity

- **Name**: hexagon-<lang>
- **Type**: Hexagonal Architecture Template
- **Language**: <Language>
- **Location**: `repos/hexagon-<lang>`
- **Status**: Active

## Architecture

This is a Hexagonal Architecture (Ports & Adapters) template following the pattern from:
- Domain: Pure business logic, no external dependencies
- Application: Use cases, orchestrates domain
- Infrastructure: Adapters (DB, HTTP, etc.)
- Primary: Driving adapters (REST, CLI, etc.)
- Secondary: Driven adapters (Repository interfaces)

## Quality Standards

- Tests: Unit tests for domain, integration tests for adapters
- Formatting: Language-specific formatter (e.g., `rustfmt`, `black`, `prettier`)
- Linting: Language-specific linter with strict rules
- Type Safety: Full type coverage where possible

## Common Tasks

```bash
# Build
<build-command>

# Test
<test-command>

# Format
<format-command>

# Lint
<lint-command>
```

## Dependencies

Minimal dependencies in domain layer. Infrastructure may use:
- Database drivers
- HTTP frameworks
- Serialization libraries
```

---

## Phase 4: Missing Projects (GitHub → Local)

### 4.1 Projects to Clone

| Project | Language | Status | GitHub URL |
|---------|----------|--------|------------|
| phenotype-types | Python | Active | phenotype-dev/phenotype-types |
| phenotype-research-engine | Python | Active | phenotype-dev/phenotype-research-engine |
| phenotype-cli-extensions | TypeScript | Active | phenotype-dev/phenotype-cli-extensions |
| Datamold | TypeScript | Active | phenotype-dev/Datamold |
| Duple | Python | Archived | phenotype-dev/Duple |
| phenotype-ops | Python | Archived | phenotype-dev/phenotype-ops |
| Seedloom | TypeScript | Archived | phenotype-dev/Seedloom |
| BytePort | Go | Archived | phenotype-dev/BytePort |
| Flowra | TypeScript | Archived | phenotype-dev/Flowra |
| Guardis | TypeScript | Archived | phenotype-dev/Guardis |
| phenotype-validation | Unknown | Archived | phenotype-dev/phenotype-validation |
| phenotype-cache | Unknown | Archived | phenotype-dev/phenotype-cache |
| phenotype-go-sdk | Go | Archived | phenotype-dev/phenotype-go-sdk |

### 4.2 Clone Command

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Active projects
gh repo clone phenotype-dev/phenotype-types
gh repo clone phenotype-dev/phenotype-research-engine
gh repo clone phenotype-dev/phenotype-cli-extensions
gh repo clone phenotype-dev/Datamold

# Archived projects (move to .archive after clone)
for proj in Duple phenotype-ops Seedloom BytePort Flowra Guardis phenotype-validation phenotype-cache phenotype-go-sdk; do
  gh repo clone "phenotype-dev/$proj" -- --depth 1
  mv "$proj" ".archive/$proj-$(date +%Y%m%d)"
done
```

---

## Phase 5: Organizational Practices

### 5.1 Missing Infrastructure Checklist

For each project, ensure:

- [ ] `.github/workflows/ci.yml` - Primary CI workflow
- [ ] `.github/CODEOWNERS` - Code ownership
- [ ] `CLAUDE.md` - Agent instructions
- [ ] `CONTRIBUTING.md` - Contribution guidelines
- [ ] `SECURITY.md` - Security policy
- [ ] `LICENSE` - Apache 2.0 or MIT
- [ ] `.gitignore` - Language-appropriate
- [ ] `.pre-commit-config.yaml` - Pre-commit hooks (optional)

### 5.2 CI/CD Workflow Template

Create reusable workflow templates in `.github/workflows/templates/`:

```
.github/workflows/templates/
├── rust.yml          # Rust CI template
├── python.yml        # Python CI template
├── typescript.yml    # TypeScript CI template
├── java.yml          # Java CI template
├── swift.yml         # Swift CI template
├── dotnet.yml        # .NET CI template
├── elixir.yml        # Elixir CI template
└── zig.yml           # Zig CI template
```

### 5.3 AgilePlus Integration

For active projects, ensure AgilePlus specs exist:
- Check `.agileplus/specs/` for corresponding specs
- Link PRs to work packages

---

## Execution Order

1. **Phase 1**: Move projects from `remote-clones/` to `repos/`
2. **Phase 2**: Add CI/CD to all hexagon projects (parallel)
3. **Phase 3**: Add documentation to incomplete hexagon projects
4. **Phase 4**: Clone missing GitHub projects
5. **Phase 5**: Standardize organizational practices

---

## Rollback Plan

If issues arise:
1. Projects can be restored from `remote-clones/` backup
2. CI changes are non-destructive (additive)
3. Documentation can be regenerated from templates

---

## Success Metrics

- [ ] All 16 projects moved successfully
- [ ] All 16 projects have CI/CD
- [ ] All hexagon projects have standardized documentation
- [ ] All 13 missing projects cloned/archived
- [ ] `projects/INDEX.md` updated with all projects
