# Infrastructure Consolidation Roadmap

## Executive Summary

This roadmap consolidates duplicated infrastructure across 16 projects in the repos shelf.

| Metric | Current | Target | Reduction |
|--------|---------|--------|-----------|
| CI workflow files | 16+ | 3 templates | 81% |
| Config file copies | 79+ cliff.toml, 81+ codecov.yml | 3 templates | 96% |
| Git hook copies | 6+ duplicates | 1 canonical source | 83% |
| Empty workspace declarations | 5+ | 0 | 100% |

---

## Phase 1: Immediate Fixes (Day 1)

### 1.1 Merge Conflicts Resolution ✅
**Status:** COMPLETED

| File | Issue | Resolution |
|------|-------|------------|
| `/Users/kooshapari/CodeProjects/Phenotype/repos/AGENTS.md` | 10+ conflict markers | Replaced with clean shelf-level guidance |
| `/Users/kooshapari/CodeProjects/Phenotype/repos/CLAUDE.md` | 15+ conflict markers | Replaced with clean shelf-level context |

### 1.2 SECURITY.md Creation ✅
**Status:** COMPLETED

| Location | Purpose |
|----------|---------|
| `/Users/kooshapari/CodeProjects/Phenotype/repos/SECURITY.md` | Shelf-level security policy template |
| `/Users/kooshapari/CodeProjects/Phenotype/repos/template-commons/SECURITY.md.template` | Project-level template |

---

## Phase 2: CI/CD Consolidation (Week 1)

### 2.1 Reusable Workflows (Already Exist)

The following reusable workflows exist in `template-commons/.github/workflows/`:

| Workflow | Projects to Migrate |
|----------|---------------------|
| `reusable-rust-ci.yml` | Tokn, phenotype-cipher, phenotype-xdd-lib, phenotype-forge, Tossy, thegent-plugin-host |
| `reusable-python-ci.yml` | HexaPy, PolicyStack, Profila |
| `reusable-typescript-ci.yml` | Quillr, Planify |
| `reusable-security-scan.yml` | All 16 projects |

### 2.2 Migration Template - Rust Projects

**From (Old Pattern):**
```yaml
# Tokn/.github/workflows/ci.yml (BEFORE)
name: CI
on: [push, pull_request]
jobs:
  build:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - run: cargo build -v
      - run: cargo test -v
```

**To (New Pattern):**
```yaml
# Tokn/.github/workflows/ci.yml (AFTER)
name: CI

on:
  push:
    branches: [main]
  pull_request:
    branches: [main]
  workflow_dispatch:

jobs:
  ci:
    name: Rust CI
    uses: KooshaPari/template-commons/.github/workflows/reusable-rust-ci.yml@main
    with:
      crate-name: tokn
      rust-version: stable
      enable-coverage: true
      enable-security: true

  security:
    name: Security Audit
    uses: KooshaPari/template-commons/.github/workflows/reusable-security-scan.yml@main
    with:
      language: rust
      scan-cargo-audit: true
```

### 2.3 Proof-of-Concept: Tokn Migration

**Steps:**
1. Backup existing `Tokn/.github/workflows/ci.yml`
2. Replace with reusable workflow reference
3. Test by pushing to feature branch
4. Verify all checks pass
5. Document results

---

## Phase 3: Config File Consolidation (Week 2)

### 3.1 Centralized Templates

Create `template-commons/config-templates/` with:

| Template | Variables | Projects |
|----------|-----------|----------|
| `cliff.toml.template` | `{{PROJECT_NAME}}` | All Rust projects |
| `codecov.yml.template` | `{{COVERAGE_THRESHOLD}}` (default: 85%) | All projects |
| `deny.toml.template` | None (standard) | All Rust projects |
| `nextest.toml.template` | None (standard) | All Rust projects |

### 3.2 Template Format

**cliff.toml.template:**
```toml
[changelog]
header = "# Changelog"
body = """
## What's Changed
{% for group, commits in commits | group_by(attribute="group") %}
### {{ group | upper_first }}
{% for commit in commits %}
- {{ commit.message | upper_first }} by @{{ commit.author.name }}
{% endfor %}
{% endfor %}
"""
trim = true

[git]
traditional_commit_parsing = true
commit_parsers = [
    { message = "^feat", group = "Features" },
    { message = "^fix", group = "Bug Fixes" },
    { message = "^doc", group = "Documentation" },
    { message = "^perf", group = "Performance" },
    { message = "^refactor", group = "Refactor" },
    { message = "^style", group = "Styling" },
    { message = "^test", group = "Testing" },
    { message = "^chore\\(release\\)", skip = true },
    { message = "^chore", group = "Miscellaneous Tasks" },
    { body = ".*security", group = "Security" },
]
protect_breaking_commits = true
filter_commits = true
tag_pattern = "v[0-9]*"
skip_tags = "v0.1.0-beta.1"
ignore_tags = ""
topo_order = false
sort_commits = "oldest"
```

### 3.3 One-Time Migration Script

```bash
#!/bin/bash
# migrate-configs.sh

PROJECTS=("Cmdra" "Tokn" "phenotype-cipher" "phenotype-xdd-lib" "phenotype-forge" "Tossy")
TEMPLATE_DIR="template-commons/config-templates"

for project in "${PROJECTS[@]}"; do
    echo "Migrating $project..."
    
    # Copy templates
    cp "$TEMPLATE_DIR/cliff.toml.template" "$project/cliff.toml"
    cp "$TEMPLATE_DIR/codecov.yml.template" "$project/codecov.yml"
    cp "$TEMPLATE_DIR/deny.toml.template" "$project/deny.toml"
    
    echo "Done: $project"
done
```

---

## Phase 4: Git Hooks Consolidation (Week 2-3)

### 4.1 Current State Analysis

| Source | Files | Duplicates |
|--------|-------|------------|
| `pheno-cli/hooks/` | 3 hooks | 2 copies in AgilePlus/pheno-cli/ |
| `templates/hooks/` | 2 hooks | Should be canonical |
| `platforms/thegent/hooks/` | 2 hooks | Overlapping functionality |

### 4.2 Consolidation Plan

**Canonical Source:** `templates/hooks/` (already exists)

**Structure:**
```
templates/hooks/
├── pre-commit              # Conventional commits + linting
├── pre-push                # Test runner + branch validation
├── commit-msg              # Message format validation
└── install.sh              # One-command install script
```

### 4.3 Migration Steps

1. **Delete duplicates:**
   - `AgilePlus/pheno-cli/internal/hooks/scripts/*`
   - `AgilePlus/pheno-cli/hooks/*`

2. **Update references:**
   - All projects point to `templates/hooks/`

3. **Install script:**
   ```bash
   #!/bin/bash
   # templates/hooks/install.sh
   
   REPO_ROOT=$(git rev-parse --show-toplevel)
   HOOKS_DIR="$REPO_ROOT/templates/hooks"
   
   git config core.hooksPath "$HOOKS_DIR"
   echo "Git hooks configured from $HOOKS_DIR"
   ```

---

## Phase 5: Rust Workspace Cleanup (Week 3)

### 5.1 Remove Empty Workspaces

**Projects affected:**
- Cmdra
- phenotype-cipher
- phenotype-xdd-lib
- phenotype-forge
- thegent-plugin-host

**Current (Incorrect):**
```toml
# Cargo.toml
[workspace]

[package]
name = "cmdra"
```

**Fixed:**
```toml
# Cargo.toml
[package]
name = "cmdra"
```

**Script:**
```bash
#!/bin/bash
for dir in Cmdra phenotype-cipher phenotype-xdd-lib phenotype-forge thegent-plugin-host; do
    if [ -f "$dir/Cargo.toml" ]; then
        # Remove empty [workspace] declarations
        sed -i '/^\[workspace\]$/d' "$dir/Cargo.toml"
        echo "Fixed: $dir/Cargo.toml"
    fi
done
```

---

## Phase 6: Automation & Governance (Week 4)

### 6.1 CI Check for Template Drift

Create `.github/workflows/template-sync-check.yml`:

```yaml
name: Template Sync Check

on:
  schedule:
    - cron: '0 0 * * 1'  # Weekly
  workflow_dispatch:

jobs:
  check:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Check config templates
        run: |
          # Compare project configs against templates
          # Fail if drift detected
          
      - name: Check CI workflows
        run: |
          # Verify all projects use reusable workflows
          # List projects with inline CI for manual review
```

### 6.2 Project Onboarding Template

```yaml
# New projects should:
1. Copy template-commons/SECURITY.md.template to SECURITY.md
2. Use reusable workflows from template-commons
3. Reference templates/hooks/ for git hooks
4. Use config-templates/ for standard configs
```

---

## Implementation Timeline

| Week | Tasks | Deliverables |
|------|-------|--------------|
| 1 | Merge conflicts ✅, SECURITY.md ✅, Tokn PoC | Migrated Tokn CI |
| 2 | Config templates, mass migration script | All Rust configs migrated |
| 3 | Git hooks consolidation, workspace cleanup | Single hooks source |
| 4 | Automation, documentation | Template sync CI |

---

## Success Metrics

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Lines of CI config | ~800 | ~150 | 80% reduction |
| Config file maintenance burden | 79+ files | 3 templates | 96% reduction |
| Time to update all projects | Days | Minutes | 95% reduction |
| Git hook inconsistencies | 4+ sources | 1 source | 75% reduction |

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Breaking changes during migration | Test on feature branch first; keep backups |
| Project-specific customizations | Templates support variables; keep per-project overrides |
| Reusable workflow availability | template-commons is already established |
| Team adoption | Document in AGENTS.md; enforce via CI |

---

## Related Files

| File | Purpose |
|------|---------|
| `template-commons/.github/workflows/reusable-rust-ci.yml` | Rust CI template |
| `template-commons/.github/workflows/reusable-security-scan.yml` | Security scanning |
| `templates/hooks/` | Canonical git hooks |
| `SECURITY.md` | Security policy template |
