---
audience: [developers, agents]
---

# Research

Phase 0 investigation that analyzes your codebase and produces evidence-based technical decisions before architecture planning.

## What It Does

The research phase:

1. **Scans the codebase** for relevant patterns, existing implementations, and reusable code
2. **Evaluates feasibility** of each spec requirement
3. **Identifies risks and dependencies** (internal and external)
4. **Recommends technical decisions** with reasoning
5. **Estimates complexity** and effort
6. **Produces research artifacts** for the planning phase

## Quick Usage

```bash
agileplus research 001
```

Output:

```
Analyzing codebase for spec: 001-email-notifications

Scanning relevant code...
✓ Found 3 existing email implementations
✓ Found event bus system (pub/sub)
✓ Found SendGrid integration stub

Evaluating feasibility...
✓ Dependencies available (sendgrid crate, tokio)
✓ No conflicting requirements
✓ Integration points identified

Generated research artifacts:
  kitty-specs/001-email-notifications/research/
  ├── codebase-scan.md
  ├── feasibility.md
  └── decisions.md

Next: agileplus plan 001
```

## Research Output Structure

Research creates three documents:

```
kitty-specs/001-email-notifications/research/
├── codebase-scan.md       # What exists in the codebase
├── feasibility.md         # Can we build this? Risks?
└── decisions.md           # Recommended technical decisions
```

## 1. Codebase Scan

Maps existing code relevant to the feature:

```markdown
# Codebase Scan: Email Notifications

## Existing Email Code

### SendGrid Integration
- **File**: `src/email/sendgrid.rs`
- **Status**: Partial stub, not production-ready
- **What it does**: Basic template rendering
- **What's missing**: Error handling, retry logic

### Event System
- **File**: `src/events/mod.rs`
- **Type**: Pub/Sub event bus (async-trait)
- **Usage**: Other services publish events here
- **Example**: UserSignedUp, CommentAdded, PaymentProcessed
- **Reusable**: Yes, used by 3 other features

### Database Models
- **File**: `src/models/user.rs`
- **Fields**: id, email, created_at
- **Capability**: Can add email_subscribed, email_preference fields

## Similar Features

### Feature: User Notifications (2023)
- **File**: `src/features/notifications/mod.rs`
- **Sends to**: In-app notifications only
- **Pattern**: Event listener → create notification record
- **Reusable**: Notification pattern applies to email

## Dependencies

### Production Dependencies
- `sendgrid`: 0.20 (for API integration)
- `tokio`: 1.35 (async runtime, already present)
- `sqlx`: 0.7 (ORM, already present)
- `serde_json`: 1.0 (JSON, already present)

### Dev Dependencies
- `mockall`: 0.12 (for mocking SendGrid)

## Patterns This Codebase Uses

- **Async/await**: Heavily used (tokio)
- **Database**: SQLx with migrations
- **Testing**: Unit tests + integration tests
- **Error handling**: Custom error types with thiserror
- **Configuration**: Environment variables

## Integration Points

### With Event System
```
UserSignedUp event
  → email handler listens
  → sends welcome email
```

### With Database
```
Email events
  → stored in email_queue table
  → processed by async worker
  → status updated (pending → sent → failed)
```

### With User Model
```
User entity
  → needs email_subscribed boolean
  → needs email_preferences JSON
```
```

## 2. Feasibility Assessment

Evaluates if we can build each requirement:

```markdown
# Feasibility Assessment: Email Notifications

## Requirement Analysis

### REQ-1: Send Welcome Email
**Can we build?** ✓ Yes
**Effort**: 2-4 hours
**Dependencies**: SendGrid API, User model
**Risks**: None identified
**Pattern**: Event listener → SendGrid send

### REQ-2: Send Mention Email
**Can we build?** ✓ Yes
**Effort**: 4-6 hours
**Dependencies**: Mention event, SendGrid, user email lookup
**Risks**: Performance — notify on every mention may spike load
**Mitigation**: Queue emails, batch sends

### REQ-3: Unsubscribe Support
**Can we build?** ✓ Yes
**Effort**: 3-4 hours
**Dependencies**: Database preferences table, email links
**Risks**: Low, straightforward implementation
**Pattern**: Similar to existing notification prefs

### REQ-4: 24-Hour Retry
**Can we build?** ✓ Yes
**Effort**: 2-3 hours
**Dependencies**: Job queue (tokio-cron or similar)
**Risks**: Clock synchronization, job persistence
**Mitigation**: Use SQLx job table + background worker

## Overall Feasibility

**Summary**: ✓ All requirements are feasible

**Complexity**: Medium
- Not trivial (multiple components)
- Not complex (clear patterns exist)
- Estimated effort: 16-24 hours (2-3 days with full team)

**Confidence**: High (85%)
- Codebase has similar implementations
- Dependencies are available and stable
- No architectural blockers

## Risks

| Risk | Severity | Mitigation |
|------|----------|-----------|
| SendGrid API downtime | Medium | Implement retry + circuit breaker |
| Email bounces not handled | Medium | Monitor bounces, disable on threshold |
| Performance under load | Low | Async queue + batch processing |
| Database size | Low | Archive old emails quarterly |

## Dependencies

**Blocked by?** Nothing
**Blocks?** Nothing (independent feature)
**Can start?** Yes, immediately
```

## 3. Recommended Decisions

Technical decisions with reasoning:

```markdown
# Technical Decisions: Email Notifications

## Decision 1: Email Queue Storage

**Choice**: Store in database (not in-memory)

**Rationale**:
- Survives process restarts
- Can retry failed sends
- Audit trail for compliance
- Codebase uses SQLx, not new dependency

**Alternative considered**: Redis queue
- Pro: Faster, simpler
- Con: Loss on crash, no audit trail

**Decision**: Database (SQLx)

---

## Decision 2: Async Email Sending

**Choice**: Non-blocking async with tokio

**Rationale**:
- Codebase uses tokio everywhere
- Aligns with existing patterns
- Doesn't block request handling

**Implementation**:
```rust
tokio::spawn(async {
  send_email(recipient, template).await
})
```

---

## Decision 3: Template Engine

**Choice**: Handlebars for email templates

**Rationale**:
- Lightweight, no unsafe code
- Used in 2 other features already
- Good for email templates

**Alternative**: Tera or Minijinja
- More powerful, overkill for email
- Would introduce new dependency

**Decision**: Handlebars

---

## Decision 4: SendGrid Integration

**Choice**: Direct API calls (not wrapper crate)

**Rationale**:
- We only need send + webhook handling
- Reduces dependency bloat
- Simple REST API, easy to maintain

**Implementation**:
```rust
POST https://api.sendgrid.com/v3/mail/send
{
  "personalizations": [...],
  "from": {...},
  "subject": "...",
  "content": [...]
}
```

---

## Summary Table

| Decision | Choice | Confidence |
|----------|--------|------------|
| Queue storage | SQLx database | High |
| Async runtime | tokio (existing) | High |
| Template engine | Handlebars | Medium |
| SendGrid API | Direct REST | High |
```

## When to Research

Research is most valuable for:

**1. Unfamiliar Parts of Codebase**

```bash
# Feature touches auth system we don't know well?
agileplus research 001
```

**2. Integration with External Systems**

```bash
# Feature integrates with SendGrid, Stripe, or 3rd party?
agileplus research 001
```

**3. Performance-Sensitive Features**

```bash
# Feature touches data pipeline or real-time system?
agileplus research 001
```

**4. Unclear Feasibility**

```bash
# Not sure if we can build this?
agileplus research 001
```

## When to Skip Research

You can skip research for:

**1. Simple, well-understood features**

```bash
# Adding a simple form field?
agileplus research 001 --skip
agileplus plan 001
```

**2. Features replicating existing patterns**

```bash
# Adding 10th similar notification type?
agileplus research 001 --skip
agileplus plan 001
```

**3. Purely frontend changes**

```bash
# Adding a UI component?
agileplus research 001 --skip
agileplus plan 001
```

## Research Quality Checklist

Good research includes:

- ✓ Existing code mapped and analyzed
- ✓ Dependencies identified (internal and external)
- ✓ Risks documented with mitigations
- ✓ Similar features referenced
- ✓ Complexity estimated (low/medium/high)
- ✓ Decision rationale documented
- ✓ Confidence level stated

## Example: Complete Research Artifact

Real example research output:

```markdown
# Research: User Authentication

## Codebase Scan

### Existing Auth Code
- `src/auth/jwt.rs` - JWT token generation and validation
- `src/auth/password.rs` - Password hashing with argon2
- `src/auth/middleware.rs` - Request guard for protected routes
- `src/models/user.rs` - User entity with password_hash field

### Event System
- `src/events/mod.rs` - Pub/sub for events
- Available events: UserCreated, LoginAttempted, PasswordChanged

### Similar Features
- Session management (implemented 2023)
- Account settings page (implemented 2024)
- Both use JWT tokens, similar patterns

## Feasibility

**Summary**: ✓ Feasible, medium complexity

**Requirements**:
1. Login endpoint - ✓ Easy (JWT pattern exists)
2. Signup endpoint - ✓ Easy (User model exists)
3. Password reset - ✓ Medium (email integration needed)
4. Session management - ✓ Easy (JWT already used)

**Risks**:
- Password reset requires email (depends on email feature)
- Rate limiting needed (add to middleware)
- Password validation rules need to be strict

**Effort estimate**: 12-16 hours

## Decisions

1. Use existing JWT pattern for sessions
2. Require password reset via email (not in-app)
3. Hash passwords with argon2 (already in use)
4. Enforce rate limiting on login attempts
```

## Tips for Thorough Research

**1. Look for Patterns**

```bash
# Search for similar implementations
grep -r "pub struct" src/ | grep -i "handler"
grep -r "impl.*Trait" src/ | grep -i "event"
```

**2. Check Dependencies**

```bash
cat Cargo.toml | grep -A 50 "\\[dependencies\\]"
# See what's already available
```

**3. Read Recent Code**

```bash
git log --oneline -20 src/
# What did team work on recently?
```

**4. Identify Integration Points**

```bash
# Where does this feature touch other systems?
# (database, cache, external APIs, events)
```

## Next Steps

After researching:

```bash
agileplus plan 001
```

This uses the research to create:
- Architecture decisions
- File changes list
- Build sequence
- Dependency graph

## Related Documentation

- **[Specify](/workflow/specify)** — Create the specification
- **[Clarify](/workflow/clarify)** — Resolve ambiguities
- **[Plan](/workflow/plan)** — Architecture planning
- **[Getting Started](/guide/getting-started)** — Full workflow
