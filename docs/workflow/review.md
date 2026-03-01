---
audience: [developers, agents]
---

# Review

Structured code review against the plan and coding standards.

## What It Does

1. Loads the WP prompt, plan, and spec context
2. Checks deliverable completeness
3. Validates governance constraints
4. Runs dependency checks for downstream WPs
5. Produces approve/reject decision with feedback

## Usage

```bash
agileplus review WP01
```

## Review Checklist

The reviewer validates:

- [ ] All deliverable files exist
- [ ] Tests pass
- [ ] No files outside WP scope modified
- [ ] Commit messages reference the WP ID
- [ ] Code follows project conventions
- [ ] Plan adherence — implementation matches design decisions

## Outcomes

### Approve

```bash
agileplus move WP01 --to done --note "Review passed: clean implementation"
```

Unblocks dependent WPs automatically.

### Request Changes

```bash
agileplus move WP01 --to planned --review-feedback-file /tmp/feedback-WP01.md
```

The WP returns to `planned` with feedback attached. The implementer addresses feedback and resubmits.

## Dependency Awareness

When rejecting a WP that has dependents:
- Dependent WPs are warned to rebase after fixes
- The reviewer should note which downstream WPs are affected
