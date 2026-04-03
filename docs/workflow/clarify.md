---
audience: [developers, agents, pms]
---

# Clarify

Identify and resolve underspecified areas in a feature specification through targeted clarification questions.

## What It Does

The clarify phase:

1. **Analyzes the spec** for ambiguous statements, missing edge cases, and underspecified requirements
2. **Generates targeted questions** (up to 5, prioritized by impact)
3. **Records answers** back into the spec, replacing `[NEEDS CLARIFICATION]` markers
4. **Validates the updated spec** against quality criteria
5. **Reports readiness** for next phase (research or plan)

## Quick Usage

```bash
agileplus clarify 001
```

Output:

```
Analyzing spec: 001-email-notifications

Found 3 ambiguous areas:

Q1: Email Rate Limits (SCOPE)
Context: Spec mentions "rate limiting" but doesn't define limits.
Need: What's the max emails per user per hour?

Options:
  (A) 5 emails/hour
  (B) 10 emails/hour
  (C) 20 emails/hour
  (D) No limit, queue them all

Your answer? > B

Q2: Bounce Handling (EDGE CASE)
Context: Spec doesn't define what happens when emails bounce.
Need: Should we disable email for bounced addresses?

Your answer? > Yes, mark as bounced and skip future emails

Q3: Retry Strategy (TECHNICAL)
Context: How many retries for failed sends?
Need: Retry policy?

Your answer? > Retry 3 times over 24 hours, then mark failed

✓ Spec updated with answers
✓ 3 ambiguities resolved

Next: agileplus research 001
```

## How It Finds Ambiguities

Clarify analyzes the spec for:

**1. Explicit Markers**

Looks for `[NEEDS CLARIFICATION]` comments:

```markdown
## Rate Limiting

Users should not be rate-limited [NEEDS CLARIFICATION: what's the limit?]
```

**2. Vague Language**

Detects unclear statements:

```markdown
# ✗ Vague (detected as ambiguous)
- The system should handle errors gracefully
- Performance should be reasonable
- Users should get good results

# ✓ Clear (no ambiguity)
- Failed emails retry 3 times over 24 hours
- Search queries complete within 500ms
- Search results show first 10 matches, sorted by relevance
```

**3. Missing Details**

Identifies incomplete scenarios:

```markdown
# ✗ Missing details
- Users can subscribe/unsubscribe from notifications

# ✓ Complete
- Users can subscribe/unsubscribe from notifications in Account Settings
- Unsubscribe action takes effect immediately
- Changes sync to all active sessions within 10 seconds
```

**4. Edge Cases Not Covered**

Finds gaps:

```markdown
# ✗ Misses edge cases
- Users can reset their password

# ✓ Covers edge cases
- Users can reset password via email
- Reset link expires after 1 hour
- Old reset links become invalid after new reset
- No email sent if account doesn't exist (security)
```

## Question Categories

Questions are prioritized by implementation impact:

### 1. Scope (Highest Priority)

Defines what's in/out:

```
Q: The spec mentions "user notifications". Does this include:
   (A) In-app notifications only
   (B) Email only
   (C) Both email and in-app
   (D) Email, in-app, and push

Your answer determines architecture and effort.
```

### 2. Outcomes & Success (High Priority)

Defines measurable goals:

```
Q: Spec says "timely notifications". What's "timely"?
   (A) Within 10 minutes
   (B) Within 1 minute
   (C) Real-time (<5 seconds)

This affects queue design and cost.
```

### 3. Risks & Security (High Priority)

Identifies failure modes:

```
Q: What happens if email sending fails?
   (A) Retry infinitely
   (B) Retry 3 times then discard
   (C) Retry 3 times then alert admin

This affects reliability and data loss.
```

### 4. User Experience (Medium Priority)

Defines behavior in edge cases:

```
Q: User receives 100 emails in one minute. What happens?
   (A) All sent immediately
   (B) Queued, sent over time
   (C) Dropped, user sees error

This affects perceived quality.
```

### 5. Technical Constraints (Medium Priority)

Identifies hard limits:

```
Q: Any performance requirements?
   (A) <100ms per email send
   (B) <1s per email send
   (C) No requirement, send when possible

This affects architecture.
```

## Example Clarification Session

```bash
agileplus clarify 001
```

**Spec: User Authentication**

```
---

Q1: Password Requirements (SCOPE)
────────────────────────────────
Context: Spec says "strong passwords" but doesn't define strength rules.

Need: What are the password complexity requirements?

Current options in spec:
  (A) Minimum 8 characters
  (B) 8+ chars, uppercase, number, special char
  (C) 8+ chars, uppercase, number
  (D) Custom regex-based validation

Your answer (enter A-D)? > B

✓ Recorded: Passwords must be 8+ characters with uppercase,
  number, and special character.


Q2: Account Lockout (EDGE CASE)
───────────────────────────────
Context: No specification of what happens after failed login attempts.

Need: Should accounts lock after failed attempts?

Your answer? > Yes, 5 failed attempts locks account for 30 min

✓ Recorded: Accounts lock after 5 failed login attempts for 30 minutes.


Q3: Social Login (SCOPE)
────────────────────────
Context: Spec mentions "email/password login" but references social login
        elsewhere without details.

Need: Is social login (Google, GitHub) in scope?

Your answer? > Not for MVP, document as future work

✓ Recorded as future work item (not in this feature).

───────────────────────────────────────────────────────────────
✓ All ambiguities resolved!

Spec status: CLARIFIED
Next step: agileplus research 001
```

## Updating the Spec with Answers

Clarify automatically updates the spec:

**Before (with ambiguities):**

```markdown
## Functional Requirements

- User can log in with email and password
- [NEEDS CLARIFICATION] Password requirements?
- Login should protect against brute force
- [NEEDS CLARIFICATION] How many attempts before lockout?
```

**After (clarified):**

```markdown
## Functional Requirements

- User can log in with email and password
- Password must be 8+ characters, with uppercase, number, special char
- Login should protect against brute force
- Account locks for 30 minutes after 5 failed login attempts
```

## Dry Run (Preview Only)

See what clarify would ask without making changes:

```bash
agileplus clarify 001 --dry-run
```

Output shows questions but doesn't update spec.

## When to Skip Clarify

You can skip clarify if:

**1. Spec is crystal clear** (no ambiguities)

```bash
agileplus clarify 001
# Output: "No ambiguities found. Ready to research or plan."
```

**2. You want fast iteration**

```bash
agileplus clarify 001 --skip
agileplus research 001
# Move directly to research
```

**3. You'll clarify during research**

Research might identify patterns that clarify the spec. That's OK.

```bash
agileplus research 001
# May generate questions during research phase
```

## Manual Clarification

You can also edit the spec directly:

```bash
# Edit spec manually
nano kitty-specs/001-feature/spec.md

# Then validate
agileplus spec validate 001
```

## Validating Clarifications

```bash
agileplus spec validate 001 --clarified-only
```

This ensures all ambiguities are resolved and the spec meets quality criteria:

- ✓ Every requirement is testable
- ✓ Success criteria are measurable
- ✓ User scenarios have concrete steps
- ✓ No vague language (good, reasonable, fast, etc.)

## Next Steps

After clarifying:

**Option 1: Research (Recommended)**

Scan codebase and assess feasibility:

```bash
agileplus research 001
```

**Option 2: Plan Directly**

For well-understood features, jump to planning:

```bash
agileplus plan 001
```

## Troubleshooting

**Q: Clarify asks about things I already specified**

A: The spec may have been unclear. Re-read the answer and update spec if needed:

```bash
agileplus spec edit 001
# Update the relevant section
```

**Q: Want to accept a default answer without reviewing**

A: Use `--auto-accept` to use first/recommended answer:

```bash
agileplus clarify 001 --auto-accept
```

**Q: Want different questions**

A: Clarify analyzes your spec automatically. If different ambiguities exist, edit the spec first:

```bash
agileplus spec edit 001
# Make spec clearer, remove resolved items
agileplus clarify 001
```

## Related Documentation

- **[Specify](/workflow/specify)** — Create the initial specification
- **[Research](/workflow/research)** — Codebase analysis and feasibility
- **[Plan](/workflow/plan)** — Architecture decisions
- **[Getting Started](/guide/getting-started)** — Full workflow walkthrough
