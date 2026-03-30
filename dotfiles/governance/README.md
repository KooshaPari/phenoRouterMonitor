# Governance — Consolidated Templates

This directory contains canonical governance and configuration templates for the Phenotype ecosystem. These templates provide standardized patterns for project documentation, development policies, and architectural decisions.

## Contents

### CLAUDE.md Base Template
**File**: `CLAUDE.base.md`

The canonical base template for all `CLAUDE.md` files across Phenotype projects.

**Use for**:
- New projects needing a `CLAUDE.md`
- Existing projects wanting to align with Phenotype standards
- Understanding which sections are required vs. optional

**Key Sections**:
- Project identity and overview
- AgilePlus integration (if applicable)
- Branch discipline and workflow
- Federated architecture (Phenotype Docs Chassis, AgilePlus Governance)
- Quality gates and local testing
- Test & specification traceability
- Language-specific customizations

**How to Use**:
1. Copy `CLAUDE.base.md` to your project root as `CLAUDE.md`
2. Customize sections marked **[CUSTOMIZE]** with your project details
3. Remove sections marked **[OPTIONAL]** if not applicable
4. Commit to version control

**Example for a Rust Library**:
```bash
# Copy base template
cp dotfiles/governance/CLAUDE.base.md path/to/my-rust-lib/CLAUDE.md

# Edit to customize
# - Update "PROJECT_NAME" → "my-rust-lib"
# - Keep Rust-specific section, remove Python/TypeScript sections
# - Update FR prefix to "FR-MYLIB-"
# - Adjust branch naming pattern if needed
```

---

## Templates by Project Type

### Platform Projects
Projects like `thegent` that serve as platforms or provide shared tooling.

**Use**: `CLAUDE.base.md` with:
- AgilePlus Mandate section (if tracking work in AgilePlus)
- Federated Architecture section
- Language-specific sections for all supported languages
- Cross-Repo References for key dependencies

**Example**: `platforms/thegent/CLAUDE.md`

### Library Projects
Reusable libraries and packages (e.g., `phenotype-config-core`, `phenotype-error-core`).

**Use**: `CLAUDE.base.md` with:
- Project Identity customized for the library
- Federated Architecture section
- Language-specific section for primary language
- Reduced Cross-Repo References (only direct dependencies)

**Example**: `crates/phenotype-config-core/CLAUDE.md`

### Application Projects
User-facing applications (e.g., `heliosCLI`, `AgilePlus`).

**Use**: `CLAUDE.base.md` with:
- AgilePlus Mandate section
- All language-specific sections relevant to the app
- Federated Architecture section
- Comprehensive Cross-Repo References

---

## Customization Patterns

### Pattern 1: Minimal Library CLAUDE.md

For small, focused libraries:

```markdown
# my-library — CLAUDE.md

## Project Overview
- **Name**: my-library
- **Description**: Brief one-liner
- **Location**: `crates/my-library/` or `libs/my-library/`
- **Language**: Rust (or Python, TypeScript, etc.)

## Branch Discipline
[Copy from CLAUDE.base.md]

## Quality Checks
[Include only relevant sections from Language-Specific]

## Testing & Traceability
[Copy from CLAUDE.base.md]

## Specification Documents
[Copy from CLAUDE.base.md]
```

### Pattern 2: Full Platform CLAUDE.md

For comprehensive platforms:

```markdown
# platform-name — CLAUDE.md

## Project Overview
[Copy from CLAUDE.base.md]

## AgilePlus Mandate
[Copy from CLAUDE.base.md]

## Phenotype Federated Hybrid Architecture
[Copy from CLAUDE.base.md]

## Branch Discipline
[Copy from CLAUDE.base.md]

## Quality Checks
[Include all relevant language sections]

## Testing & Traceability
[Copy from CLAUDE.base.md]

## Specification Documents
[Copy from CLAUDE.base.md]

## Design System
[Copy from CLAUDE.base.md if using Impeccable]

## Cross-Repo References
[Customize for platform dependencies]

## Language-Specific Sections
[Include for all languages used in platform]
```

---

## Integration with Other Templates

This CLAUDE.base.md works alongside other thegent templates:

| Template | Location | Purpose |
|----------|----------|---------|
| Pre-commit config | `dotfiles/hooks/.pre-commit-config.base.yaml` | Standardized pre-commit hooks |
| Quality gates | `templates/quality/` | Language-specific quality scripts |
| Linter configs | `templates/<language>/` | Language-specific linter templates |

**Workflow**:
1. Start with `CLAUDE.base.md` for project documentation standards
2. Use language-specific hooks from `.pre-commit-config.base.yaml`
3. Copy quality gate scripts from `templates/quality/`
4. Copy linter configs from `templates/<language>/`

---

## Distribution & Updates

### How Projects Reference This Template

**Option 1: Copy** (recommended for most projects)
```bash
cp path/to/thegent/dotfiles/governance/CLAUDE.base.md my-project/CLAUDE.md
# Customize my-project/CLAUDE.md with project-specific details
```

**Option 2: Git Submodule** (for coordinated updates)
```bash
git submodule add https://github.com/KooshaPari/thegent.git shared/thegent
# Reference in CI: cp shared/thegent/dotfiles/governance/CLAUDE.base.md ./CLAUDE.md
```

### Versioning

- **Current Version**: 1.0 (2026-03)
- **Stability**: Stable — breaking changes will increment major version
- **Update Frequency**: Quarterly (last Friday of Q)
- **Changelog**: Updates announced in `worklogs/GOVERNANCE.md`

### How to Stay Updated

1. Subscribe to worklog notifications in `worklogs/GOVERNANCE.md`
2. Review changelog entries when new versions are released
3. Check `CLAUDE.base.md` periodically for improvements
4. Test new versions in a feature branch before adopting

---

## Common Questions

### Q: Should I use the base template or a variant?
**A**: Start with `CLAUDE.base.md`. It covers all cases. Remove sections that don't apply to your project.

### Q: Can I deviate from the template?
**A**: Yes, but sparingly. Phenotype templates are designed for consistency across projects. If you need substantial changes, file an issue in thegent describing why.

### Q: How do I update my existing CLAUDE.md?
**A**: Compare your current file with the base template. Adopt missing sections, update cross-references, and ensure all required sections are present.

### Q: What if my project has a different structure?
**A**: Customize the paths in the template. The structure (sections, FR traceability, testing) should remain consistent.

### Q: Who maintains this template?
**A**: Phenotype Governance Team. Suggestions and improvements welcome via issue or PR.

---

## Related Files

- **Pre-commit Hooks**: `dotfiles/hooks/.pre-commit-config.base.yaml` — Consolidated hook configuration
- **Quality Gates**: `templates/quality/quality-gate.base.sh` — Standardized quality checking
- **Linter Templates**: `templates/<language>/` — Language-specific linter configs
- **Project Templates**: `templates/initialize-project/` — Full project scaffolds

---

## See Also

- **Phenotype Architecture**: `repos/thegent/docs/governance/23_ARCHITECTURAL_GOVERNANCE.md`
- **AgilePlus Governance Chassis**: `docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **Phenotype Docs Chassis**: `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **Global CLAUDE Instructions**: `~/.claude/CLAUDE.md`
