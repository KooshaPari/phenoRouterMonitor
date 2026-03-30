# CLAUDE.md — Base Template for Phenotype Ecosystem Projects

This is the canonical base template for `CLAUDE.md` files across the Phenotype ecosystem. Projects should use this as a foundation and customize sections marked **[CUSTOMIZE]** as needed.

## Structure

All Phenotype projects should maintain a `CLAUDE.md` file at their root (for standalone repos) or in key subdirectories (for monorepos). Use sections from this base template and add project-specific customizations.

---

## 1. Project Identity (ALWAYS customize)

**[CUSTOMIZE]** — Update with your project name, description, and location.

```markdown
# PROJECT_NAME — CLAUDE.md

## Project Overview

- **Name**: PROJECT_NAME
- **Description**: One-sentence description
- **Location**: Path in repos shelf (e.g., `platforms/thegent/`, `crates/phenotype-config-core/`)
- **Language Stack**: Rust | Python | TypeScript | Go | (list primary languages)
- **Published**: Yes | No | Internal
```

Example:
```markdown
# thegent — CLAUDE.md

## Project Overview

- **Name**: thegent
- **Description**: Dotfiles manager and governance toolkit for the Phenotype ecosystem
- **Location**: `platforms/thegent/`
- **Language Stack**: Python, Bash, TypeScript
- **Published**: Yes (GitHub Packages)
```

---

## 2. AgilePlus Integration (if applicable)

**[OPTIONAL]** — Include only if this project uses AgilePlus for work tracking.

```markdown
## AgilePlus Mandate (if using AgilePlus)

All work MUST be tracked in AgilePlus:
- Reference: /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
- CLI: `agileplus specify --title "..."` and `agileplus status <id> --wp <wp> --state <state>`
- Specs: `AgilePlus/kitty-specs/<feature-id>/`
- Worklog: `AgilePlus/.work-audit/worklog.md`

**Requirements**:
1. Check for AgilePlus spec before implementing
2. Create spec for new work: `agileplus specify --title "<feature>" --description "<desc>"`
3. Update work package status: `agileplus status <feature-id> --wp <wp-id> --state <state>`
4. No code without corresponding AgilePlus spec
```

---

## 3. Branch Discipline (ALWAYS include)

```markdown
## Branch Discipline

- **Feature branches**: Use `repos/worktrees/<project>/<category>/<branch>` or `<project>-wtrees/<topic>/`
- **Canonical repository**: Always tracks `main` only
- **Return to main**: After merge/integration checkpoints
- **Branch naming**: `feat/`, `fix/`, `docs/`, `chore/`, `refactor/` prefixes
```

---

## 4. Phenotype Federated Hybrid Architecture (if applicable)

**[OPTIONAL]** — Include if this project is part of the federated architecture (uses Phenotype Docs Chassis or AgilePlus Governance Chassis).

```markdown
## Phenotype Federated Hybrid Architecture

This project is part of the **Phenotype Federated Hybrid Architecture**, which provides two complementary chassis systems:

### Phenotype Docs Chassis
Provides VitePress configuration, design tokens, and theme components for consistent documentation.
- **Contributing**: thegent provides VitePress template, config builder, and CSS baseline
- **Package**: `@phenotype/docs` (published to GitHub Packages)
- **Documentation**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`

### AgilePlus Governance Chassis
Defines specification-driven delivery: PRD, ADR, FUNCTIONAL_REQUIREMENTS, PLAN, USER_JOURNEYS with FR traceability.
- **Implementing**: Maintain root-level spec files and worklog entries
- **Documentation**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`

**For this project**:
- Maintain `/PRD.md`, `/ADR.md`, `/FUNCTIONAL_REQUIREMENTS.md` with FR-{PROJECT}-XXX IDs
- Tag all tests with `@pytest.mark.requirement("FR-{PROJECT}-NNN")` or `describe("FR-{PROJECT}-NNN: ...")`
- Map code entities in `docs/reference/CODE_ENTITY_MAP.md`
- Create worklog entries in `docs/worklogs/` per phase
```

---

## 5. Quality Gates & Local Testing

**[CUSTOMIZE]** — Adjust commands for your project's language and tools.

```markdown
## Local Quality Checks

From this repository root:

### Python Projects
```bash
task quality          # Lint (ruff), type check, run tests
task quality:full     # quality + ruff format check
```

### Rust Projects
```bash
task quality          # clippy, fmt check, tests
cargo test            # Run all tests
cargo clippy --all-targets -- -D warnings
```

### TypeScript/JavaScript Projects
```bash
task quality          # oxlint, prettier, tests
pnpm lint
pnpm test
```

### Vale Markdown Linting (all projects)
```bash
task vale:install     # Install Vale via Homebrew (macOS)
vale .                # Run Vale on all markdown files
```
```

---

## 6. Test & Spec Traceability

**[ALWAYS include]** — Every project must have traceability from requirements to tests.

```markdown
## Testing & Specification Traceability

All tests MUST reference a Functional Requirement (FR):

### Python (pytest)
```python
@pytest.mark.requirement("FR-{PROJECT}-NNN")
def test_feature_name():
    """Test description. Traces to: FR-{PROJECT}-NNN"""
    # Test body
```

### Rust (cargo test)
```rust
// Traces to: FR-{PROJECT}-NNN
#[test]
fn test_feature_name() {
    // Test body
}
```

### TypeScript/JavaScript (vitest)
```typescript
describe("FR-{PROJECT}-NNN: feature description", () => {
  test("should do X", () => {
    // Test body
  });
});
```

**Verification**:
- Every FR-{PROJECT}-XXX in FUNCTIONAL_REQUIREMENTS.md MUST have >=1 test
- Every test MUST reference >=1 FR-{PROJECT}-XXX
- Run: `task quality` to verify traceability
```

---

## 7. Specification Documents

**[CUSTOMIZE]** — Adjust for your project structure.

```markdown
## Specification Documents (Root Level)

**Maintain these files at project root**:

| File | Purpose |
|------|---------|
| `PRD.md` | Product Requirements Document (epics, user stories) |
| `ADR.md` | Architecture Decision Records |
| `FUNCTIONAL_REQUIREMENTS.md` | Granular FR-{PROJECT}-NNN requirements |
| `PLAN.md` | Phased WBS with DAG dependencies |
| `USER_JOURNEYS.md` | User journeys with ASCII flow diagrams |

**Maintain these trackers in `docs/reference/`**:

| File | Purpose |
|------|---------|
| `FR_TRACKER.md` | FR implementation status, test coverage |
| `CODE_ENTITY_MAP.md` | Forward/reverse mapping: code ↔ requirements |
| `ADR_STATUS.md` | ADR implementation status |
| `PLAN_STATUS.md` | Phase/task completion status |
```

---

## 8. Design System (Impeccable)

**[INCLUDE if using Impeccable]** — For projects with UI or documentation.

```markdown
## Design System (Impeccable)

Impeccable provides design automation and enforcement:
- **Skills**: `frontend-design`, `audit`, `critique`, `polish`, `normalize`, `animate`, etc.
- **Setup**: Run `/teach-impeccable` in this project to establish persistent design context
- **Global Context**: `/Users/kooshapari/CodeProjects/Phenotype/repos/.impeccable.md`
- **CSS Baseline**: Add to all VitePress `custom.css` and app `globals.css`:

```css
/* impeccable CSS baseline — github.com/pbakaus/impeccable */
*, *::before, *::after { box-sizing: border-box; }
html { -webkit-font-smoothing: antialiased; -moz-osx-font-smoothing: grayscale; text-rendering: optimizeLegibility; }
img, video { max-width: 100%; height: auto; }
input, button, textarea, select { font: inherit; }
p, h1, h2, h3, h4, h5, h6 { overflow-wrap: break-word; }
```
```

---

## 9. UTF-8 Encoding (if applicable)

**[INCLUDE if managing AgilePlus specs]**

```markdown
## UTF-8 Encoding

All markdown files must use UTF-8. Validate with:
```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
agileplus validate-encoding --all --fix
```
```

---

## 10. Cross-Repo References

**[OPTIONAL]** — Include if this project references other key projects.

```markdown
## Cross-Project References

| Project | Purpose | Path |
|---------|---------|------|
| thegent | Dotfiles & governance toolkit | `platforms/thegent/` |
| AgilePlus | Work tracking & spec management | `repos/AgilePlus/` |
| phenotype-docs | Documentation chassis | GitHub Packages: `@phenotype/docs` |
| phenotype-shared | Shared libraries & utilities | `libs/phenotype-shared/` |
```

---

## 11. Language-Specific Customizations

### Rust Projects

```markdown
## Rust-Specific Notes

- **Workspace**: All crates in `crates/` or `rust/`
- **Testing**: `cargo test --all` and `cargo test --lib <crate>`
- **Clippy**: `cargo clippy --all-targets -- -D warnings`
- **Formatting**: `cargo fmt -- --check`
- **Pre-commit**: See `.pre-commit-config.yaml` in repo root
```

### Python Projects

```markdown
## Python-Specific Notes

- **Package Manager**: Use `uv` for dependency management
- **Linting**: `ruff check .` and `ruff format --check .`
- **Type Checking**: `basedpyright` or `pyright`
- **Testing**: `pytest` with `pytest-xdist` for parallelization
- **Pre-commit**: See `.pre-commit-config.yaml` in repo root
- **Layout**: `src/` layout with `tests/` at root
```

### TypeScript/JavaScript Projects

```markdown
## TypeScript/JavaScript-Specific Notes

- **Package Manager**: `pnpm` (see `pnpm-lock.yaml`)
- **Bundler**: VitePress for docs, Vite for apps
- **Linting**: `oxlint` + `prettier`
- **Type Checking**: Strict TypeScript config
- **Testing**: `vitest` with coverage tracking
- **Pre-commit**: See `.pre-commit-config.yaml` in repo root
```

---

## 12. See Also

**[CUSTOMIZE]** — Link to relevant docs and standards.

```markdown
## See Also

- **Global Instructions**: `~/.claude/CLAUDE.md` (system-wide governance)
- **Phenotype Docs Chassis**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **AgilePlus Governance**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **Architectural Governance**: `repos/thegent/docs/governance/23_ARCHITECTURAL_GOVERNANCE.md`
- **Pre-commit Hooks**: `.pre-commit-config.yaml` (see `thegent/dotfiles/hooks/`)
- **Quality Templates**: `thegent/templates/quality/` and `thegent/templates/<language>/`
```

---

## Usage Instructions

### For New Projects

1. Copy this template to your project root as `CLAUDE.md`
2. Customize sections marked **[CUSTOMIZE]** with your project details
3. Remove sections marked **[OPTIONAL]** if not applicable
4. Update language-specific sections with your tech stack
5. Commit to version control

### For Existing Projects

1. Review your current `CLAUDE.md`
2. Compare with relevant sections in this base template
3. Adopt missing sections (especially testing & traceability)
4. Update cross-references to point to canonical locations
5. Ensure all required sections are present

### For Updates

When this base template is updated:
1. Phenotype governance team will notify via worklog entry
2. Projects should review and adopt improvements
3. Use `diff` to compare your CLAUDE.md with the updated base
4. Merge improvements while preserving project-specific customizations

---

## Template Customization Points

Below are the main areas you should customize for your project:

| Section | Customization |
|---------|---------------|
| Project Identity | Name, description, location, language stack |
| AgilePlus Mandate | Include only if using AgilePlus |
| Quality Checks | Adjust commands for your language |
| FR Prefix | Change `{PROJECT}` to your project ID (e.g., `FR-THEGENT-`) |
| Specification Documents | Adjust file paths for your structure |
| Language-Specific | Include only relevant language sections |
| Cross-Repo References | Link to projects your codebase depends on |

---

## Document Info

- **Template Version**: 1.0 (2026-03)
- **Location**: `thegent/dotfiles/governance/CLAUDE.base.md`
- **Canonical URL**: See thegent repository
- **Last Updated**: 2026-03-29
- **Maintained By**: Phenotype Governance Team
