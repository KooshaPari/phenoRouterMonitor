---
audience: [developers, pms]
---

# Accept

Final validation that a feature meets all specification requirements and success criteria.

## What It Does

The accept phase:

1. **Verifies all work packages completed** — every WP is in `done` state
2. **Validates against spec** — every requirement from spec has implementation
3. **Checks success criteria** — success metrics are measurably met
4. **Confirms no ambiguities** — no `[NEEDS CLARIFICATION]` markers remain
5. **Produces acceptance report** — detailed validation results
6. **Marks feature accepted** — ready for merge to main

## Quick Usage

```bash
agileplus accept 001
```

Output:

```
Validating feature: 001-email-notifications

Pre-Merge Checks:
✓ All 7 work packages completed (done)
✓ All commits present and signed
✓ No merge conflicts detected

Specification Compliance:
✓ REQ-1: Send welcome email — IMPLEMENTED
✓ REQ-2: Send mention email — IMPLEMENTED
✓ REQ-3: User preferences — IMPLEMENTED
✓ REQ-4: 24-hour retry logic — IMPLEMENTED
✓ REQ-5: Unsubscribe support — IMPLEMENTED

Success Criteria Validation:
✓ 95% of emails sent within 5 minutes — PASSED (measured: 98%)
✓ Unsubscribe links work — PASSED (manual verification)
✓ Zero impact on API latency — PASSED (measured: <1ms added)
✓ Email delivery rate >98% — PASSED (measured: 99.2%)

Overall Status: ✅ ACCEPTED

Ready to merge:
  agileplus merge 001
```

## Acceptance Checklist

The accept phase validates:

### 1. Work Packages Complete

```markdown
## WP Completion Check
- [ ] WP01 status: done (database schema)
- [ ] WP02 status: done (email models)
- [ ] WP03 status: done (SendGrid integration)
- [ ] WP04 status: done (email queue)
- [ ] WP05 status: done (event handlers)
- [ ] WP06 status: done (API endpoints)
- [ ] WP07 status: done (integration tests)

All WPs complete? → YES, proceed
```

### 2. Specification Coverage

```markdown
## Requirement Implementation
- [ ] REQ-1: Send welcome email on signup
  - Implemented in: WP03, WP05
  - Test coverage: test_welcome_email()
  - Status: VERIFIED

- [ ] REQ-2: Send mention notifications
  - Implemented in: WP05 (event handler)
  - Test coverage: test_mention_notification()
  - Status: VERIFIED

- [ ] REQ-3: User email preferences
  - Implemented in: WP02, WP06 (API endpoint)
  - Test coverage: test_email_preferences()
  - Status: VERIFIED

- [ ] REQ-4: 24-hour retry with exponential backoff
  - Implemented in: WP04 (queue processor)
  - Test coverage: test_retry_backoff()
  - Status: VERIFIED

- [ ] REQ-5: Unsubscribe support
  - Implemented in: WP06 (unsubscribe endpoint)
  - Test coverage: test_unsubscribe_link()
  - Status: VERIFIED
```

Every requirement must have:
- ✓ Implementation in code
- ✓ Test coverage
- ✓ Verification status

### 3. Success Criteria Validation

From spec:

```markdown
## Success Criteria

✓ 95% of emails sent within 5 minutes
  Measured: 98.5% (99 of 101 test emails)
  Status: PASSED

✓ Unsubscribe links work and are respected
  Tested: 5 unsubscribe flows
  Status: PASSED

✓ No performance impact to API
  Baseline latency: 45ms
  With email feature: 46ms (1ms added)
  Status: PASSED (<5ms acceptable)

✓ Email delivery rate >98%
  Measured: 99.2% (1 bounce out of 131)
  Status: PASSED

✓ Proper error handling for API failures
  Tested: SendGrid timeout scenarios
  Status: PASSED
```

All success criteria must be:
- Measured or tested
- Show passing status
- Have supporting evidence

### 4. No Ambiguities Remain

```markdown
## Specification Clarity
- [ ] No [NEEDS CLARIFICATION] markers in spec
- [ ] All questions answered and integrated
- [ ] Edge cases documented
- [ ] Assumptions validated

Check spec:
  cat kitty-specs/001-email-notifications/spec.md | grep NEEDS
  # Should return: (no results)
```

### 5. Code Quality Standards

```markdown
## Quality Gates
- [ ] All tests pass
  cargo test --all → 127/127 passed

- [ ] No compiler warnings
  cargo clippy → 0 warnings

- [ ] Code follows conventions
  cargo fmt --check → clean

- [ ] Security audit passes
  cargo audit → 0 vulnerabilities

- [ ] Documentation complete
  All public items have doc comments
```

## Manual Acceptance Process

### Step 1: Verify All WPs Done

```bash
agileplus show 001 --all-wps
```

Output:

```
WP01: Database      [████████] done
WP02: Models        [████████] done
WP03: SendGrid      [████████] done
WP04: Queue         [████████] done
WP05: Handlers      [████████] done
WP06: API           [████████] done
WP07: Tests         [████████] done
```

All should show `done`. If any are `planned`, `doing`, or `for_review`, they're not ready yet.

### Step 2: Compare Requirements to Implementation

```bash
# View spec requirements
grep -A 50 "## Functional Requirements" \
  kitty-specs/001-feature/spec.md

# View test coverage
grep "fn test_" tests/email_integration_tests.rs

# Verify mapping:
# - REQ-1 → test_welcome_email
# - REQ-2 → test_mention_notification
# etc.
```

### Step 3: Test Success Criteria

For each success criterion:

```bash
# Example: "95% of emails sent within 5 minutes"

# Run integration test
cargo test email_delivery_time

# Check results
# Expected: Emails arrive within 300 seconds, >95% success rate
```

### Step 4: Run Quality Gates

```bash
# All tests
cargo test --all

# No warnings
cargo clippy

# Format OK
cargo fmt --check

# Security
cargo audit

# Doc coverage
cargo doc --no-deps
```

All should pass.

### Step 5: Prepare Acceptance Report

Document your findings:

```markdown
# Acceptance Report: 001-Email Notifications

## Feature Overview
Email notifications feature to send transactional emails on user events.

## Verification Summary

### Work Packages
- [x] WP01 Database: DONE
- [x] WP02 Models: DONE
- [x] WP03 SendGrid: DONE
- [x] WP04 Queue: DONE
- [x] WP05 Handlers: DONE
- [x] WP06 API: DONE
- [x] WP07 Tests: DONE

### Requirements Coverage
- [x] REQ-1 Welcome email: IMPLEMENTED
- [x] REQ-2 Mention notification: IMPLEMENTED
- [x] REQ-3 User preferences: IMPLEMENTED
- [x] REQ-4 Retry logic: IMPLEMENTED
- [x] REQ-5 Unsubscribe: IMPLEMENTED

### Success Criteria
- [x] 95% delivery within 5 min: PASSED (98.5%)
- [x] Unsubscribe works: PASSED
- [x] No API latency impact: PASSED (<1ms)
- [x] >98% delivery rate: PASSED (99.2%)

### Quality Gates
- [x] Tests pass: 127/127 ✓
- [x] No compiler warnings: 0 ✓
- [x] Format OK: ✓
- [x] Security audit: 0 vulns ✓

### Recommendation
✅ ACCEPT — Feature is ready for production merge.

Acceptance Date: 2026-03-05
Accepted By: Alice (Product Manager)
```

### Step 6: Accept the Feature

```bash
agileplus accept 001
```

System marks feature as `accepted` and creates timestamped acceptance record.

## Scenarios

### Scenario 1: All Criteria Met

```bash
agileplus accept 001

# Output:
# ✅ Feature 001 ACCEPTED
# Status: Accepted
# Date: 2026-03-05 14:30:00 UTC
# Ready to merge: agileplus merge 001
```

### Scenario 2: WP Still in Progress

```bash
agileplus accept 001

# Output:
# ❌ Cannot accept: WP07 still in for_review state
#
# Pending work:
# - WP07: Tests (move to done when review complete)
#
# After WP07 is done:
#   agileplus accept 001
```

### Scenario 3: Success Criteria Not Met

```bash
# If manual testing shows success criterion not met:

# Option 1: Fix and retest
# Go back to WP that affects the criterion, fix it, retest

# Option 2: Update criterion if requirement changed
agileplus spec edit 001
# Update the success criterion if business requirements changed
# Then retest

# When all pass:
agileplus accept 001
```

## Post-Acceptance

After acceptance:

```bash
# Ready for merge
agileplus merge 001

# This:
# - Merges all WP branches to main
# - Removes worktrees
# - Closes tracker issues
# - Creates release tag (if configured)
```

## Best Practices

**1. Don't Skip Acceptance**

Always run accept before merge, even if you're confident:

```bash
# Good: Verify before merging
agileplus accept 001
agileplus merge 001

# Risky: Skipping acceptance
agileplus merge 001  # What if a WP is still for_review?
```

**2. Test Success Criteria Thoroughly**

Don't just trust unit tests. Validate in realistic scenarios:

```bash
# Unit test passes, but...
cargo test test_email_delivery

# Manual verification with real SendGrid account
# Test with various email clients
# Check spam folder, delivery logs, etc.
```

**3. Document Edge Cases**

If you discover edge cases, document them:

```markdown
## Known Limitations
- SendGrid sandbox mode has 1/second rate limit (production: 100/sec)
- Very large emails (>5MB) may timeout
- Unsubscribe links expire after 30 days of inactivity
```

**4. Get Stakeholder Sign-Off**

For important features, have PM or stakeholder accept:

```bash
# Share acceptance report with team
# Get written approval

# Mark with approver
agileplus accept 001 --approved-by "alice@example.com"
```

## Next Steps

After acceptance:

```bash
# Merge to main
agileplus merge 001

# This completes the feature
```

## Related Documentation

- **[Merge](/workflow/merge)** — Integration to main
- **[Review](/workflow/review)** — Code quality review
- **[Core Workflow](/guide/workflow)** — Full pipeline
- **[Specify](/workflow/specify)** — Original specification
