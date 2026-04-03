---
audience: [developers, agents, pms]
---

# Plan

Generate an implementation blueprint with architecture decisions, file changes, and build sequence.

## What It Does

The plan phase:

1. **Analyzes spec + research** to understand requirements and codebase context
2. **Makes architecture decisions** (with reasoning)
3. **Identifies all file changes** (create, modify, delete)
4. **Defines build sequence** (dependency order)
5. **Creates dependency graph** (what depends on what)
6. **Produces plan.md** — the blueprint for implementation

## Quick Usage

```bash
agileplus plan 001
```

Output:

```
Analyzing spec and research...
✓ Processing 8 requirements
✓ Checking codebase patterns
✓ Identifying file changes
✓ Building dependency graph

Generated:
  kitty-specs/001-email-notifications/plan.md

Ready to generate work packages:
  agileplus tasks 001
```

## Plan Document Structure

Plan creates a detailed `plan.md` with these sections:

### 1. Architecture Decisions

```markdown
## Architecture Decisions

### Decision 1: Async Email Queue
**Choice**: Queue emails in database, process async
**Rationale**: Decouples email sending from request handling
**Trade-off**: More complex (queue + worker) vs simpler (sync send)
**Alternative**: Sync SendGrid API calls (simpler but blocks requests)

### Decision 2: Event-Driven Dispatch
**Choice**: Event system triggers email handlers
**Rationale**: Reuses existing event architecture
**Trade-off**: More flexible but requires event schema changes

### Decision 3: Template Storage
**Choice**: Email templates in database
**Rationale**: Allows non-code template updates
**Trade-off**: Database query per template vs hardcoded strings

### Decision 4: Retry Strategy
**Choice**: Exponential backoff, 5 retries over 72 hours
**Rationale**: Tolerates temporary SendGrid downtime
**Trade-off**: More resilient but adds complexity
```

### 2. File Changes

Comprehensive list of all files touched:

```markdown
## File Changes

| File | Action | Size | Purpose |
|------|--------|------|---------|
| src/models/email.rs | CREATE | ~200L | Email data model |
| src/models/email_preference.rs | CREATE | ~150L | User subscription prefs |
| src/db/migrations/001_email_tables.sql | CREATE | ~50L | Schema for email tables |
| src/email/mod.rs | CREATE | ~100L | Email module root |
| src/email/sendgrid.rs | CREATE | ~300L | SendGrid API integration |
| src/email/templates.rs | CREATE | ~200L | Template rendering |
| src/email/queue.rs | CREATE | ~250L | Email queue worker |
| src/events/handlers/email.rs | CREATE | ~150L | Event handlers for email |
| src/handlers/account.rs | MODIFY | +80L | Add email preference endpoint |
| src/models/user.rs | MODIFY | +2L | Add email_preference field |
| tests/email_integration_tests.rs | CREATE | ~400L | Integration tests |
| docs/email-feature.md | CREATE | ~100L | Feature documentation |

**Summary**:
- Files created: 11
- Files modified: 2
- Total new lines: ~2,000
```

### 3. Data Model

Definition of new or modified data structures:

**Email**

```rust
struct Email {
  id: u64,
  user_id: u64,
  event_type: String,      // "welcome", "mention", "payment"
  recipient: String,       // email address
  subject: String,
  body: String,
  status: EmailStatus,     // pending, sent, failed, bounced
  attempts: u32,           // retry count
  created_at: DateTime,
  sent_at: Option<DateTime>,
  failed_reason: Option<String>,
}

enum EmailStatus {
  Pending,
  Sent,
  Failed,
  Bounced,
}
```

**EmailPreference**

```rust
struct EmailPreference {
  user_id: u64,
  category: String,        // "marketing", "transactional", "account"
  enabled: bool,
  unsubscribe_token: String,
  created_at: DateTime,
  updated_at: DateTime,
}
```

**EmailTemplate**

```rust
struct EmailTemplate {
  id: u64,
  event_type: String,
  subject: String,
  body_template: String,   // Handlebars template
  created_at: DateTime,
  updated_at: DateTime,
}
```

### 4. Build Sequence

Step-by-step order to minimize blockers:

**Phase 1: Foundation (Days 1-2)**

1. Database schema migration (001_email_tables.sql)
2. User model updates (add email_preference field)
3. Email and EmailPreference models

*Why first*: Everything else depends on the database schema

**Phase 2: Core (Days 2-3)**

4. SendGrid integration (HTTP client, API calls)
5. Email queue worker (async processing)
6. Template rendering (Handlebars setup)

*Why next*: Implements sending mechanism

**Phase 3: Integration (Days 3-4)**

7. Event handlers (connect to event system)
8. Account settings endpoint (preference management)
9. Email preferences model

*Why next*: Connects to rest of system

**Phase 4: Quality (Days 4-5)**

10. Integration tests
11. Error handling refinements
12. Documentation

*Why last*: All pieces in place to test thoroughly

**Parallelizable Steps:**
- Steps 4-8 can run in parallel (no dependencies between them)
- Only constraint: All of Phase 1 must complete before Phase 2

### 5. Dependency Graph

Visual representation of dependencies:

```
src/models/email.rs (1)
  ↓
src/email/sendgrid.rs (4) ←─ src/db/migrations/ (1)
  ↓
src/email/queue.rs (5)
  ↓                    ↗ src/handlers/account.rs (8)
src/events/handlers/email.rs (7)
  ↓
tests/email_integration_tests.rs (10)

Legend:
(1) = Phase 1: Foundation
(4-8) = Phase 2-3: Core & Integration
(10) = Phase 4: Quality
```

### 6. External Dependencies

Third-party crates and tools required:

```markdown
## External Dependencies

### New Dependencies
- sendgrid: ^0.20
- handlebars: ^4.0
- serde_json: ^1.0 (upgrade if needed)
- tokio-util: ^0.7 (for retry logic)

### Existing Dependencies (Already Used)
- tokio: ^1.35 (async runtime)
- sqlx: ^0.7 (database)
- serde: ^1.0 (serialization)
- log: ^0.4 (logging)

### Total New Dependencies
1 major (sendgrid), 1 moderate (handlebars), rest minor

**No dependency conflicts identified**
```

### 7. Risk Assessment

Potential issues and mitigations:

```markdown
## Risks & Mitigations

| Risk | Severity | Mitigation |
|------|----------|-----------|
| SendGrid API rate limits | Medium | Implement queue + backoff |
| Email deliverability | Medium | Monitor bounce rates, test with Mailtrap |
| Performance under load | Low | Async queue, batch processing |
| Database migration issues | Low | Test migration up/down |
| Spam filter classification | Medium | Use SendGrid templates, proper headers |
| User unsubscribe abuse | Low | Rate limit unsubscribe requests |
```

## Full Example Plan Document

```markdown
---
title: Implementation Plan - Email Notifications
spec: 001-email-notifications
created: 2026-03-01
---

# Implementation Plan: Email Notifications

## Overview

This plan details the architecture and implementation approach for the email
notifications feature, based on specification 001 and research artifacts.

**Scope**: Send transactional emails (welcome, mentions, receipts) with
user preference management and reliability guarantees.

**Effort**: 16-24 hours (2-3 days with full team)
**Complexity**: Medium
**Risk Level**: Low

## Architecture Decisions

### 1. Async Queue for Email Sending
- **Choice**: Store emails in database, process async with tokio background task
- **Rationale**: Non-blocking, survives restarts, audit trail
- **Alternative**: Sync API calls (simpler, but blocks requests)
- **Trade-off**: +300 LOC for queue, -request latency impact

### 2. Event-Driven Dispatch
- **Choice**: Listen to domain events (UserCreated, CommentAdded, etc.)
- **Rationale**: Reuses existing event system, decoupled
- **Alternative**: Direct function calls (simpler, tightly coupled)
- **Trade-off**: +150 LOC for handlers, better architecture

### 3. Database-Stored Templates
- **Choice**: Store email templates in database
- **Rationale**: Non-developers can update templates without code
- **Alternative**: Hardcoded or file-based templates
- **Trade-off**: More queries, but more flexible

### 4. Handlebars for Template Rendering
- **Choice**: Handlebars templating engine
- **Rationale**: Lightweight, no unsafe code, good for email
- **Alternative**: Tera or Minijinja (overkill for our needs)
- **Trade-off**: One more dependency, but widely used

## File Changes

| File | Action | Lines | Purpose |
|------|--------|-------|---------|
| src/models/email.rs | CREATE | 180 | Email data model |
| src/models/email_preference.rs | CREATE | 120 | Preference model |
| src/models/email_template.rs | CREATE | 80 | Template model |
| src/db/migrations/001_email_schema.sql | CREATE | 45 | Database schema |
| src/email/mod.rs | CREATE | 50 | Module root + exports |
| src/email/sendgrid.rs | CREATE | 300 | SendGrid API client |
| src/email/queue.rs | CREATE | 250 | Queue processor |
| src/email/templates.rs | CREATE | 150 | Template rendering |
| src/events/handlers/email.rs | CREATE | 180 | Event handlers |
| src/handlers/account.rs | MODIFY | +90 | Email pref endpoint |
| src/models/user.rs | MODIFY | +2 | email_preferences field |
| src/models/mod.rs | MODIFY | +3 | Export new models |
| tests/email_test.rs | CREATE | 400 | Unit + integration tests |

**Summary**: 11 files created, 2 modified, ~1,700 new lines

## Data Models

### Email
- id, user_id, event_type, recipient, subject, body
- status (pending, sent, failed, bounced)
- attempts (retry count), created_at, sent_at

### EmailPreference
- user_id, category (marketing/transactional/account)
- enabled (boolean), unsubscribe_token

### EmailTemplate
- id, event_type, subject, body_template
- created_at, updated_at

## Build Sequence

1. **Database migration** — Create tables and indexes
2. **Models** — Email, EmailPreference, EmailTemplate
3. **SendGrid client** — HTTP calls to SendGrid API
4. **Queue processor** — Async loop consuming queue
5. **Template rendering** — Handlebars + database lookups
6. **Event handlers** — Listen and dispatch emails
7. **API endpoints** — Preference management, unsubscribe
8. **Integration tests** — End-to-end email flows
9. **Error handling** — Retry logic, monitoring
10. **Documentation** — Feature docs for team

## External Dependencies

### New
- `sendgrid`: ^0.20 (for API)
- `handlebars`: ^4.0 (for templates)

### Existing (No changes)
- `tokio`: ^1.35 (async)
- `sqlx`: ^0.7 (database)

**Total new**: 2 dependencies, no conflicts identified

## Risks

| Risk | Mitigation |
|------|-----------|
| SendGrid API failures | Implement exponential backoff retry, 5 attempts over 72 hours |
| Email bounces | Monitor bounce rate, disable emails for bounced addresses after 3 bounces |
| Performance impact | Use async queue, don't block requests |
| Template injection | Use Handlebars sandboxed render, validate input |

## Next Steps

1. Review and approve this plan
2. Generate work packages: `agileplus tasks 001`
3. Implement in parallel work packages
4. Review and merge
```

## From Plan to Tasks

Once the plan is approved, break it into work packages:

```bash
agileplus tasks 001
```

This automatically generates work packages based on the plan:

```
✓ Created WP01-database      (1 day)  - database schema
✓ Created WP02-models        (4 hours) - data models
✓ Created WP03-sendgrid      (1 day)  - SendGrid integration
✓ Created WP04-queue         (1 day)  - Email queue processor
✓ Created WP05-handlers      (1 day)  - Event handlers
✓ Created WP06-api           (1 day)  - Account endpoints
✓ Created WP07-tests         (1 day)  - Integration tests

Work packages can run in parallel (respecting dependencies).
```

## Plan Quality Checklist

A good plan includes:

- ✓ Clear architecture decisions with rationale
- ✓ Complete file change list (create/modify/delete)
- ✓ Data model definitions
- ✓ Logical build sequence (respecting dependencies)
- ✓ Dependency graph (visual representation)
- ✓ External dependency list (crates, packages)
- ✓ Risk assessment with mitigations
- ✓ Effort/timeline estimate
- ✓ Complexity assessment

## Tips for Good Plans

**1. Be Specific**

```markdown
# ✗ Vague
- Create email service

# ✓ Specific
- Create SendGrid HTTP client with exponential backoff retry
- Create database queue table with status tracking
```

**2. Document Trade-offs**

```markdown
# ✓ Shows thinking
**Choice**: Async queue
**Trade-off**: +300 LOC complexity vs -request latency impact
```

**3. Identify Parallelizable Work**

```markdown
# ✓ Helps with scheduling
- Steps 1-2 must complete first (foundation)
- Steps 3-6 can run in parallel (no dependencies)
- Steps 7-9 require steps 1-6 complete
```

**4. Link to Specification**

```markdown
# ✓ Traceability
REQ-1 (Welcome email) → implemented in WP03-sendgrid, WP05-handlers
REQ-2 (Mention email) → implemented in WP05-handlers
```

## When Plan Reveals Issues

If planning discovers the spec is infeasible:

```bash
# Go back and clarify
agileplus clarify 001 --revisit

# Or research more thoroughly
agileplus research 001 --force
```

This is normal! Plans often reveal details that need clarification.

## Next Steps

After planning:

```bash
# Generate work packages
agileplus tasks 001

# This breaks the plan into parallel-safe work packages
```

Then move to implementation:

```bash
agileplus implement WP01
agileplus implement WP02
# etc.
```

## Related Documentation

- **[Specify](/workflow/specify)** — Create the specification
- **[Clarify](/workflow/clarify)** — Resolve ambiguities
- **[Research](/workflow/research)** — Codebase analysis
- **[Tasks](/workflow/tasks)** — Work package generation
- **[Getting Started](/guide/getting-started)** — Full workflow
