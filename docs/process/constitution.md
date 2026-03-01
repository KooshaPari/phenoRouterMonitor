---
audience: [developers, pms]
---

# Constitution

The project constitution captures governance rules, technical standards, and quality expectations.

## What It Is

A living document at `.kittify/memory/constitution.md` that defines:

- **Code quality standards** — linting rules, test coverage expectations
- **Architecture principles** — port-based design, crate boundaries
- **Governance rules** — who can approve, merge, and deploy
- **Agent constraints** — what AI agents can and cannot do
- **Naming conventions** — branch names, commit messages, file organization

## Creating a Constitution

```bash
agileplus constitution
```

This runs an interactive discovery process that asks about your project's:

1. Language and framework choices
2. Testing philosophy
3. Code review requirements
4. Deployment strategy
5. Documentation standards

## Example

```markdown
# Project Constitution

## Code Quality
- All code must pass `cargo clippy -- -D warnings`
- Minimum test coverage: 80% per crate
- No `unwrap()` in library code

## Architecture
- All I/O goes through port traits
- Domain logic has zero external dependencies
- New crates require architect approval

## Governance
- PRs require 1 approval from a maintainer
- Breaking changes require 2 approvals
- AI agents cannot modify governance files
```

## Usage in Workflow

The constitution is referenced during:
- **Plan** — ensures design decisions align with principles
- **Review** — validates code against quality standards
- **Accept** — confirms the feature meets governance requirements
