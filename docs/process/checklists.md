---
audience: [developers, pms, agents]
---

# Checklists

Automated quality gates that validate artifacts at each lifecycle phase.

## How Checklists Work

Checklists are generated per-feature and validate specific quality criteria. They act as unit tests for requirements writing.

## Types

### Specification Checklist

Validates the spec before planning:

- [ ] No implementation details (languages, frameworks, APIs)
- [ ] Focused on user value and business needs
- [ ] All mandatory sections completed
- [ ] Requirements are testable and unambiguous
- [ ] Success criteria are measurable and technology-agnostic
- [ ] Edge cases identified
- [ ] Scope clearly bounded

### Implementation Checklist

Validates WP deliverables before review:

- [ ] All deliverable files exist
- [ ] Tests pass locally
- [ ] No files outside WP scope modified
- [ ] Commit messages reference WP ID
- [ ] Code follows project conventions

### Acceptance Checklist

Validates the feature before merge:

- [ ] All WPs in `done` lane
- [ ] Every functional requirement has implementation
- [ ] All success criteria demonstrably met
- [ ] No open clarification markers

## Generating Checklists

```bash
agileplus checklist 001
```

Creates `kitty-specs/001-feature/checklists/requirements.md` with feature-specific validation items.

## Custom Checklists

Add project-specific checklist items via the [Constitution](/process/constitution):

```markdown
## Custom Checklist Items
- [ ] Accessibility audit passed (WCAG 2.1 AA)
- [ ] Performance budget met (<3s LCP)
- [ ] Security review for user-facing endpoints
```
