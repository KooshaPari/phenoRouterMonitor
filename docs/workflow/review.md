---
audience: [developers, agents]
---

# Review

Structured code quality review against specification, plan, and coding standards.

## What It Does

The review phase:

1. **Loads context** — WP prompt, plan, spec, design decisions
2. **Verifies deliverables** — all expected files created/modified
3. **Checks quality** — tests pass, code follows conventions
4. **Validates scope** — no files modified outside WP boundaries
5. **Assesses plan adherence** — implementation matches design
6. **Checks dependencies** — downstream WPs not blocked
7. **Produces decision** — approve or request changes with feedback

## Quick Usage

```bash
agileplus review WP01
```

Output:

```
Reviewing WP01: Email Models

Loading context...
✓ Spec: 001-email-notifications
✓ Plan: Implementation Plan
✓ WP prompt: Email Models task

Analyzing implementation...
✓ Email struct created
✓ EmailStatus enum defined
✓ sqlx FromRow implementations added
✓ Unit tests written (8 tests)
✓ Tests pass (8/8 ✓)

Quality checks...
✓ No compiler warnings
✓ Code follows project conventions
✓ Files within WP scope
✓ Commits reference WP01

Dependency analysis...
✓ WP02 waiting on this
✓ Can unblock after approval

Ready for decision:
  agileplus move WP01 --to done (approve)
  agileplus move WP01 --to planned (request changes)
```

## Review Checklist

A thorough review validates:

### Deliverables Completeness

```markdown
## Deliverables
- [ ] All expected files exist
  - src/models/email.rs
  - src/models/email_preference.rs
  - src/models/email_template.rs

- [ ] All expected files modified (if MODIFY action)
  - src/models/mod.rs — exports added

- [ ] No unexpected files modified
  - git diff main --stat — only listed files changed
```

### Functional Correctness

```markdown
## Functionality
- [ ] Tests pass
  - cargo test → all tests pass
  - cargo test --all → including integration tests

- [ ] No panics or unwraps in critical paths
- [ ] Error handling is present
- [ ] Edge cases handled (empty strings, null, etc.)
- [ ] Database operations use transactions where needed
```

### Code Quality

```markdown
## Code Quality
- [ ] No compiler warnings
  - cargo clippy → no warnings

- [ ] Follows project conventions
  - Naming: snake_case for functions, CamelCase for structs
  - Error types: custom errors with thiserror
  - Async: tokio/await patterns

- [ ] Well-documented
  - Public items have doc comments
  - Complex logic has inline comments

- [ ] DRY principle
  - No significant code duplication
  - Reuses patterns from codebase
```

### Scope & Boundaries

```markdown
## Scope Validation
- [ ] Only files listed in WP modified
  - git diff main --name-only | wc -l == expected count

- [ ] No out-of-scope changes
  - No changes to unrelated modules
  - No database schema changes (if not in WP)

- [ ] Commit messages reference WP
  - Each commit includes "WP01" or similar
```

### Plan Adherence

```markdown
## Plan Adherence
- [ ] Implementation matches architecture decisions
  - Design decision 1: ✓ implemented as specified
  - Design decision 2: ✓ implemented as specified

- [ ] File changes match plan
  - Plan listed X files → implemented X files

- [ ] Build sequence respected
  - Dependencies completed before this WP
  - No blockers on dependencies
```

### Dependency Validation

```markdown
## Dependencies
- [ ] All dependencies met before this started
- [ ] No unmet dependencies blocking downstream WPs
- [ ] Dependent WPs can proceed after approval
```

## Review Outcomes

### Approve: Move to Done

```bash
agileplus move WP01 --to done
```

Effect:
- WP marked as `done`
- Dependent WPs unblocked
- Can proceed to next phase

Example:

```
✓ WP01 approved and moved to done

Unblocked WPs:
  - WP02: Email Models (can start now)
  - WP04: Email Queue (still waiting on WP03)

Next: Proceed to implement WP02
```

### Request Changes: Return to Planned

```bash
agileplus move WP01 --to planned \
  --review-feedback-file /tmp/feedback.md
```

Feedback file example:

```markdown
# Review Feedback for WP01: Email Models

## Issues Found

### 1. Missing Error Types [CRITICAL]
The Email model uses generic Result types instead of custom error type.

Fix: Create EmailError enum with proper error variants

### 2. Incomplete Tests [HIGH]
Only 8 tests, but EmailStatus has 4 variants.

Fix: Add tests for status transitions and invalid conversions

### 3. Code Style [MEDIUM]
Some variable names don't follow convention (use snake_case)

Fix: Rename `emailAddr` → `email_addr`

### 4. Documentation [LOW]
Some public methods lack doc comments

Fix: Add /// doc comments to public methods

## Summary
Good work overall. Address the critical and high issues, then resubmit.

Estimated additional work: 2-3 hours
```

Effect:
- WP marked as `planned` with feedback
- Implementer addresses feedback
- Resubmits for review

## Detailed Review Process

### Step 1: Load Context

```bash
# View the WP prompt
cat kitty-specs/001-feature/tasks/WP01-*.md

# View the plan
cat kitty-specs/001-feature/plan.md

# View the spec
cat kitty-specs/001-feature/spec.md
```

### Step 2: Check Deliverables

```bash
# See what was supposed to be delivered
grep -A 20 "## Deliverables" kitty-specs/001-feature/tasks/WP01-*.md

# Verify files exist
ls -la src/models/email.rs
ls -la src/models/email_preference.rs

# Check files modified
git diff main --name-only
```

### Step 3: Run Tests

```bash
# Run all tests
cargo test --all

# Run with output
cargo test -- --nocapture

# Run specific test
cargo test email_model --
```

### Step 4: Check Code Quality

```bash
# Check for warnings
cargo clippy

# Format check
cargo fmt -- --check

# Security audit
cargo audit
```

### Step 5: Review Commits

```bash
# See all commits in this WP
git log main..HEAD --oneline

# See detailed changes
git diff main

# Review by file
git diff main -- src/models/email.rs
```

### Step 6: Assess Plan Adherence

Compare implementation to plan:

```markdown
Plan says:
- Create Email struct
- Add EmailStatus enum
- Implement sqlx FromRow
- Write unit tests

Implementation review:
✓ Email struct created with all fields
✓ EmailStatus enum with 4 variants
✓ FromRow impl using custom deserialization
✓ 8 unit tests covering main scenarios

Verdict: Plan adhered to correctly
```

### Step 7: Check Dependency Impact

```bash
# See what WPs depend on this
agileplus show 001 --wp WP01 --dependents

# Verify this WP doesn't have unmet dependencies
agileplus show 001 --wp WP01 --dependencies
```

### Step 8: Make Decision

**If everything looks good:**

```bash
agileplus move WP01 --to done
```

**If issues found:**

```bash
# Create feedback document
cat > /tmp/feedback-WP01.md << 'EOF'
# Review Feedback

## Issue 1: [CRITICAL] Missing error handling
...

## Issue 2: [HIGH] Incomplete test coverage
...

## Summary
Good foundation, but address critical issues before resubmit.
EOF

# Return to planned with feedback
agileplus move WP01 --to planned --review-feedback-file /tmp/feedback-WP01.md
```

## Review Severity Levels

Use these to categorize issues:

| Level | Meaning | Action | Example |
|-------|---------|--------|---------|
| **CRITICAL** | Breaks functionality or violates design | MUST fix | Missing error handling, security bug |
| **HIGH** | Significantly impacts quality | Should fix | Incomplete tests, performance issue |
| **MEDIUM** | Minor quality issue | Nice to fix | Code style, minor refactoring |
| **LOW** | Documentation or polish | Optional | Comment clarity, naming consistency |

### CRITICAL Example

```
❌ CRITICAL: No error handling for SendGrid API failures
The code doesn't handle network errors or API timeouts.

Fix: Add retry logic with exponential backoff and error propagation
Blocks: Cannot approve without this
```

### HIGH Example

```
⚠ HIGH: Incomplete test coverage for EmailStatus enum
Only 2 of 4 status variants are tested. Missing tests for Failed and Bounced.

Fix: Add unit tests for untested branches
Impact: Can't be confident in status transition logic
```

### MEDIUM Example

```
💡 MEDIUM: Variable naming doesn't match conventions
Used `emailAddr` instead of `email_addr` in several places.

Fix: Rename to follow snake_case convention
Impact: Code style consistency
```

## Tips for Effective Reviews

**1. Be Specific**

```markdown
# ❌ Vague
"This code is bad"

# ✓ Specific
"This function doesn't handle the case where user_id is 0.
Add a check: if user_id == 0 { return Err(...) }"
```

**2. Provide Examples**

```markdown
# ❌ No example
"Function names should be clearer"

# ✓ With example
"Rename `proc_email()` to `process_pending_email()`.
It's clearer what the function does."
```

**3. Explain Why**

```markdown
# ❌ No rationale
"Add more tests"

# ✓ With rationale
"Add tests for the failure case. Currently only happy path is tested.
If SendGrid API returns 400, we need to verify error handling works."
```

**4. Distinguish Must-Fix vs Nice-to-Have**

```markdown
# ✓ Clear priority
CRITICAL (must fix):
- Error handling missing

HIGH (should fix):
- Tests incomplete

MEDIUM (nice to have):
- Code could be more DRY
```

**5. Acknowledge Good Work**

```markdown
# ✓ Balanced feedback
"Good work on the error handling. The retry logic is well-implemented.

One issue: Missing tests for the bounced email scenario. Please add those."
```

## Dependency Cascade

When rejecting a WP that blocks others:

```bash
agileplus show 001 --wp WP02 --dependents
# Output:
# WP02 blocks: WP04, WP05, WP07

# If you reject WP02:
agileplus move WP02 --to planned --review-feedback-file /tmp/feedback.md

# System notifies:
# ⚠ Warning: Rejecting WP02 delays WP04, WP05, WP07
# Consider critical severity — work will be blocked
```

This helps reviewers understand the downstream impact of rejection.

## Next Steps

**If approved:**

```bash
agileplus move WP01 --to done
# Dependent WPs are unblocked
```

**If changes requested:**

```bash
# Implementer addresses feedback
agileplus move WP01 --to doing

# Make fixes
# Commit changes
git commit -m "fix(WP01): address review feedback

- Add comprehensive error handling for API failures
- Expand test coverage for all EmailStatus variants
- Refactor variable naming for clarity"

# Resubmit
agileplus move WP01 --to for_review

# Second review
agileplus review WP01
```

## Related Documentation

- **[Implement](/workflow/implement)** — Code implementation
- **[Accept](/workflow/accept)** — Feature acceptance
- **[Merge](/workflow/merge)** — Integration to main
- **[Core Workflow](/guide/workflow)** — Full pipeline
