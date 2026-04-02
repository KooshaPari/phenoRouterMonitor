---
audience: [developers, agents, pms]
---

# Specify

The entry point for every feature. `specify` transforms a natural language description into a structured specification through a guided discovery interview.

## What It Does

The specify phase:

1. Runs a **discovery interview** with targeted questions
2. Analyzes your responses to understand scope and intent
3. Generates a `spec.md` with requirements, user scenarios, and success criteria
4. Creates `meta.json` with feature metadata
5. Commits artifacts to your spec directory

## Quick Usage

### With Title & Description

```bash
agileplus specify --title "User Authentication" \
  --description "Add email/password login and signup"
```

Interview output:

```
Feature: User Authentication

Q1: Who are the primary users?
> Regular users and guest accounts

Q2: What authentication methods are required?
> Email/password login, signup, password reset

Q3: What's the success metric?
> Users can log in within 2 clicks
```

Generated:

```
✓ kitty-specs/001-user-authentication/spec.md
✓ kitty-specs/001-user-authentication/meta.json
✓ Feature ready to clarify or research
```

### Interactive (No Arguments)

```bash
agileplus specify
# → Full discovery interview, step by step
```

### From Queue Item

```bash
agileplus specify 42
# Creates spec from queue item #42
```

## The Discovery Interview

The interview adapts based on feature complexity. Simple features get 3-4 questions, complex features get 8-10.

### Question Categories

**1. Scope & Intent (Required)**

```
Q: What does this feature do?
```

Captures the core purpose.

**2. Users & Personas (Required)**

```
Q: Who will use this?
```

Identifies user groups and roles.

**3. Key Scenarios (Required)**

```
Q: What are the main use cases?
```

Usually 3-5 scenarios.

**4. Success Criteria (Required)**

```
Q: How do you measure success?
```

Metrics or observable outcomes.

**5. Constraints & Out-of-Scope (Optional)**

```
Q: What's explicitly NOT included?
```

Prevents scope creep.

**6. Dependencies (Optional)**

```
Q: What existing features does this depend on?
```

Identifies integration points.

**7. Technical Context (Optional)**

```
Q: Any technical constraints?
```

Performance, security, compliance.

**8. Timeline (Optional)**

```
Q: When is this needed?
```

Deadlines or milestone targets.

### Example Full Interview

```bash
agileplus specify "Add email notifications"
```

Output:

```
Project Name: Email Notifications

Q1: What problem does this solve?
> Users want to be notified about important events via email

Q2: Who are the key users?
> Application users (registered and paying)

Q3: What events trigger notifications?
> New comments, mentions, payment receipts, and account alerts

Q4: What's success for this feature?
> Users receive emails within 5 minutes of event
> Unsubscribe link works
> Email templates are branded

Q5: What email services will you use?
> SendGrid (we have API keys)

Q6: Are there performance constraints?
> Must not impact request latency
> Email sending should be async

Q7: Anything out of scope?
> SMS notifications, push notifications, email templates UI builder

Accept generated spec? (Y/n) y

✓ Specification generated successfully
  kitty-specs/001-email-notifications/spec.md
```

## Generated Specification Document

The interview output becomes a `spec.md` with these sections:

### 1. Overview

```markdown
# Email Notifications

## Problem Statement
Users want to be notified about important events via email.

## Value Proposition
- Keep users informed without app login
- Increase engagement and retention
- Provide audit trail via email archive

## Scope
In-scope:
- Event-triggered emails (comments, mentions, receipts)
- Email subscription management
- Unsubscribe handling

Out-of-scope:
- SMS, push notifications
- Email template UI builder
- Bounce/complaint handling
```

### 2. User Scenarios

```markdown
## User Scenarios

### Scenario 1: New User Signs Up
When Alice signs up, she receives a welcome email within 1 minute.
She can unsubscribe from non-critical emails in her account settings.

### Scenario 2: Mentioned in Comment
Bob mentions Alice in a comment.
Alice receives an email within 5 minutes with the comment text and a link to respond.

### Scenario 3: Payment Processed
User makes a payment.
They receive a receipt email with transaction details within 10 minutes.
```

### 3. Functional Requirements

```markdown
## Functional Requirements

| Requirement | Details |
|-------------|---------|
| Send welcome email | On signup, within 1 minute |
| Send mention email | When mentioned, include context |
| Send receipt email | On payment, include transaction details |
| Unsubscribe | Users can disable any email category |
| Retry logic | Retry failed sends 3 times over 24 hours |
| Rate limiting | Max 10 emails per user per hour |
```

### 4. Success Criteria

```markdown
## Success Criteria

- [ ] 95% of emails sent within 5 minutes
- [ ] Unsubscribe links work and respected
- [ ] No emails sent to bounced addresses
- [ ] Zero impact on API request latency (<5ms added)
- [ ] Email delivery rate >98%
```

### 5. Assumptions

```markdown
## Assumptions

- SendGrid API is available and authorized
- Database supports email queue table
- No legal requirement for GDPR-specific handling
- Event bus/pub-sub exists for triggering emails
```

### 6. Key Entities

```markdown
## Key Entities

### Email
- id: unique identifier
- event_type: mention, payment, welcome, etc.
- recipient: user email address
- subject, body: email content
- status: pending, sent, failed, bounced
- created_at, sent_at: timestamps

### EmailPreference
- user_id: who this applies to
- category: which type of email
- enabled: boolean subscription status
- unsubscribe_token: for unsubscribe links
```

## Meta.json

The system also creates `meta.json` with metadata:

```json
{
  "id": "001",
  "title": "Email Notifications",
  "description": "Send emails when users take actions",
  "status": "specified",
  "created_at": "2026-03-01T10:30:00Z",
  "created_by": "alice",
  "complexity": "medium",
  "estimated_hours": 16,
  "dependencies": ["authentication", "event-bus"],
  "tags": ["emails", "notifications", "user-engagement"]
}
```

## Full Specification Example

Here's what a complete `spec.md` looks like:

```markdown
---
title: Two-Factor Authentication
description: Enable users to secure their accounts with 2FA
status: Specified
---

# Two-Factor Authentication

## Overview

Users can enable two-factor authentication (2FA) on their accounts using
time-based one-time passwords (TOTP), such as Google Authenticator or Authy.

### Problem Statement
Accounts with weak passwords are vulnerable to brute-force attacks and
credential leaks. 2FA adds a second layer of security.

### Value Proposition
- Users feel confident their accounts are secure
- Reduces account takeover incidents by 95%+
- Supports regulatory compliance (SOC2, ISO27001)

### Scope
**In-scope:**
- TOTP setup and validation
- Backup codes for account recovery
- Disabling 2FA

**Out-of-scope:**
- SMS-based 2FA, hardware keys, biometric
- Forced 2FA for all users
- Admin enforcement policies

## User Scenarios

### Scenario 1: Enable 2FA
1. Alice logs in to her account
2. She goes to Security Settings
3. She clicks "Enable 2FA"
4. She scans a QR code with Google Authenticator
5. She enters a 6-digit code to confirm
6. She saves 10 backup codes in a safe place
✓ 2FA is now enabled

### Scenario 2: Login with 2FA
1. Bob goes to login
2. He enters email and password
3. He's prompted for his TOTP code
4. He opens Google Authenticator and enters the code
5. He's logged in
✓ Takes 30 seconds total

### Scenario 3: Recover Account with Backup Code
1. Carol has lost her phone with Authenticator
2. She tries to log in
3. She's prompted for TOTP
4. She enters a backup code instead
5. She's logged in
6. The backup code is marked as used
✓ One-time recovery works

## Functional Requirements

| # | Requirement | Details |
|---|-------------|---------|
| 1 | TOTP Setup | Generate secret, show QR code, confirm with code |
| 2 | TOTP Validation | Accept 6-digit codes, check timing window ±1 step |
| 3 | Backup Codes | Generate 10 codes, hash them, track usage |
| 4 | Login Flow | Enforce TOTP after password, before session |
| 5 | Disable 2FA | Require password confirmation |
| 6 | Recovery | Allow backup codes during login |
| 7 | Export Codes | User can download backup codes as PDF |

## Success Criteria

- [ ] 2FA can be set up in <2 minutes
- [ ] Login with 2FA takes <1 minute
- [ ] Backup code recovery works reliably
- [ ] No false positives in TOTP validation
- [ ] Clock drift ±30s is tolerated
- [ ] 100% test coverage for auth flow

## Assumptions

- Users have authenticator apps (Google Authenticator, Authy, etc.)
- Backup codes are user responsibility (we don't email them)
- Time sync between server and client is within ±30 seconds
- SMS 2FA is not required

## Key Entities

### User
- id, email, password_hash
- **2fa_secret**: encrypted TOTP secret
- **2fa_enabled**: boolean

### BackupCode
- id, user_id
- code_hash, used_at (nullable)
- created_at

## Security Considerations

- TOTP secrets encrypted at rest (AES-256-GCM)
- Backup codes hashed with bcrypt
- No TOTP codes logged or stored
- Verify 6-digit code only once
- Rate limit login attempts to 5 per minute
```

## Spec Quality Guidelines

### Do's

- ✓ Focus on **what** and **why**, not **how**
- ✓ Write for stakeholders (non-technical is fine)
- ✓ Include concrete examples and scenarios
- ✓ Make requirements testable
- ✓ Document constraints and assumptions
- ✓ Use clear, simple language

### Don'ts

- ✗ Don't specify implementation details
- ✗ Don't assume the reader knows your codebase
- ✗ Don't use vague success criteria ("be fast", "work well")
- ✗ Don't write 20-page specs (aim for 2-5 pages)
- ✗ Don't add unlimited `[NEEDS CLARIFICATION]` markers

### Testability

Every requirement should be testable:

```markdown
# ✗ Bad (not testable)
- Users can search for items

# ✓ Good (testable)
- Users can search by name, tag, or date
- Search results appear within 500ms
- Searches match partial text and are case-insensitive
```

## Next Steps

After specifying your feature:

**Option 1: Review & Clarify**

If the spec has ambiguities:

```bash
agileplus clarify 001
# Identify and resolve unclear points
```

**Option 2: Research First (Recommended)**

Scan your codebase and assess feasibility:

```bash
agileplus research 001
# Identify patterns, dependencies, risks
```

**Option 3: Plan Directly**

For well-understood features, jump to planning:

```bash
agileplus plan 001
# Create architecture decisions and work breakdown
```

## Troubleshooting

**Q: The interview doesn't ask the questions I need**

A: Use `--custom-interview` flag to guide specific questions:

```bash
agileplus specify "title" --custom-interview
# Interactive interview with your questions
```

**Q: Generated spec has gaps**

A: Use clarify to identify and fix ambiguities:

```bash
agileplus clarify 001
```

**Q: Want to modify the spec**

A: Edit the generated `spec.md` directly, then validate:

```bash
agileplus spec validate 001
```

**Q: Need to restart specification**

A: Re-run specify to regenerate:

```bash
agileplus specify 001 --force
# Overwrites existing spec
```

## Related Documentation

- **[Clarify](/workflow/clarify)** — Resolve specification ambiguities
- **[Research](/workflow/research)** — Codebase analysis and feasibility
- **[Plan](/workflow/plan)** — Architecture decisions
- **[Spec-Driven Development](/concepts/spec-driven-dev)** — Learn the philosophy
- **[Getting Started](/guide/getting-started)** — Full walkthrough including specify
