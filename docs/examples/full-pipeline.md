---
audience: [developers, agents, pms]
---

# Full Pipeline Example

Walk through the complete AgilePlus pipeline — from specification to shipped feature.

## Scenario

We're adding OAuth2 authentication to a web application.

## 1. Specify

Create the feature specification:

```bash
agileplus specify \
  --title "OAuth2 Authentication" \
  --description "Add Google and GitHub OAuth2 login flows with session management"
```

This creates a spec artifact with:
- Functional requirements
- User scenarios
- Acceptance criteria

## 2. Research

Scan the codebase for existing auth patterns:

```bash
agileplus research oauth2-authentication
```

Research outputs:
- Existing auth infrastructure (if any)
- Framework-specific patterns detected
- Dependency recommendations

## 3. Plan

Generate work packages:

```bash
agileplus plan oauth2-authentication
```

```
Generated 4 work packages:
  WP01: OAuth2 provider configuration
  WP02: Login flow implementation
  WP03: Session management
  WP04: UI login components

Dependency graph:
  WP01 ← WP02 ← WP04
  WP01 ← WP03
```

## 4. Implement

Execute work packages (can be parallelized where dependencies allow):

```bash
# WP01 first (no dependencies)
agileplus implement oauth2-authentication --wp WP01

# WP02 and WP03 in parallel (both depend only on WP01)
agileplus implement oauth2-authentication --wp WP02
agileplus implement oauth2-authentication --wp WP03

# WP04 last (depends on WP02)
agileplus implement oauth2-authentication --wp WP04
```

Each WP:
- Gets its own git branch (`oauth2-authentication-WP01`, etc.)
- Receives a structured prompt with spec context
- Is dispatched to the configured agent

## 5. Validate

Run governance checks:

```bash
agileplus validate oauth2-authentication
```

Validation checks:
- All WPs complete
- All branches exist and have commits
- Audit trail is intact (hash chain valid)
- No merge conflicts between WP branches

## 6. Ship

Merge all WP branches:

```bash
agileplus ship oauth2-authentication
```

This merges WP branches in dependency order and records the final state transition.

## 7. Retrospective

Generate a post-ship analysis:

```bash
agileplus retrospective oauth2-authentication
```

Output includes:
- Timeline (spec to ship duration)
- WP completion stats
- Agent performance (which agent handled which WP)
- Audit trail summary
