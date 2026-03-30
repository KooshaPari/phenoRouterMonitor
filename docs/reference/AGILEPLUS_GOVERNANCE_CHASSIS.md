# AgilePlus Governance Chassis

## Overview

The AgilePlus Governance Chassis is a specification-driven delivery framework that enables predictable, traceable product development across the Phenotype organization. It defines how requirements are captured, decomposed, implemented, tested, and verified at every phase of development.

**Package**: AgilePlus (see `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus`)
**CLI**: `agileplus` (see `AgilePlus/CLAUDE.md` for setup)
**Current Version**: 0.3.0+
**Stability**: Stable (backward compatible within major versions)

---

## Section 1: Provided by Chassis

The Governance Chassis provides:

### 1.1 Specification Structure

#### 1.1.1 Product Requirements Document (PRD)

**Location**: `{project-root}/PRD.md`

**Structure**:
```
# PRD — {Project Name}

## Epics
### E1.1 {Epic Title}
- User Stories
  - S1.1.1: {Story Title}
    - Acceptance Criteria
    - Business Value

### E1.2 {Epic Title}
...
```

**Purpose**: High-level product vision, user stories, acceptance criteria.

**Traceability**: Each story (Sn.m.k) traces to one or more Functional Requirements (FR-CAT-NNN).

#### 1.1.2 Architecture Decision Records (ADR)

**Location**: `{project-root}/ADR.md` and `docs/adr/{number}-title/`

**Format**:
```
# ADR-{NNN}: {Title}

## Status
{Proposed | Accepted | Deprecated | Superseded}

## Context
{Why this decision matters}

## Decision
{What we decided}

## Rationale
{Why this is the best choice}

## Alternatives Considered
{Other options and why they were rejected}

## Consequences
{What this enables and what it constrains}

## Implementation
{Code locations, patterns, examples}

## Related ADRs
{Links to related decisions}
```

**Purpose**: Record architectural decisions with context, rationale, and consequences.

#### 1.1.3 Functional Requirements (FR)

**Location**: `{project-root}/FUNCTIONAL_REQUIREMENTS.md`

**Format**:
```
# Functional Requirements

## Category: {CAT} (e.g., AUTH, CORE, API)

### FR-CAT-001: {Requirement Title}
- **Status**: Implemented | Planned | In Review
- **PRD Reference**: S1.2.1
- **ADR Reference**: ADR-005
- **Acceptance Criteria**:
  - [ ] Criterion 1
  - [ ] Criterion 2
- **Test Coverage**: `tests/fr_cat_001_*.py` (75% coverage)
- **Implementation**: `src/auth/login.rs` (lines 42-87)
- **Notes**: Any implementation notes

### FR-CAT-002: {Next Requirement}
...
```

**Purpose**: Granular, verifiable requirements that decompose epics.

**Key**: Each FR has a unique ID (FR-{CATEGORY}-{NNN}) for traceability.

#### 1.1.4 Implementation Plan (PLAN.md)

**Location**: `{project-root}/PLAN.md`

**Structure**:
```
# Implementation Plan

## Phase 1: Discovery (W1-W2)
### P1.1 Stakeholder Interviews
### P1.2 Technical Spike: {Spike Name}

## Phase 2: Design (W3-W4)
### P2.1 Architecture Design
### P2.2 Database Schema
### P2.3 API Contract Definition

## Phase 3: Implementation (W5-W12)
### P3.1 Core Module A
### P3.2 Core Module B
### P3.3 Integration

## Phase 4: Testing & Validation (W13-W14)
### P4.1 Unit Test Coverage
### P4.2 Integration Tests
### P4.3 E2E Tests
### P4.4 Performance Validation

## Dependencies
- P1.2 blocks P2.1
- P2.1, P2.2 block P3.1
- P3.1, P3.2, P3.3 block P4.1
```

**Purpose**: Phased work breakdown with explicit dependencies (DAG).

#### 1.1.5 User Journeys (USER_JOURNEYS.md)

**Location**: `{project-root}/USER_JOURNEYS.md`

**Format**:
```
# User Journeys

## UJ-1: {User Role} — {Journey Name}

```
[Actor] → [Action] → [System Response] → [Verification]
   ↓         ↓           ↓                  ↓
 User     Click Login   Auth Check      Redirect Home
  ...
```

**Acceptance**: All steps trace to FRs, no orphan steps.

**Purpose**: End-to-end flows that validate spec completeness.

### 1.2 Worklog Integration

**Location**: `{project-root}/docs/worklogs/{PHASE}-worklog.md`

**Format**:
```
# {Phase Name} — Worklog

## Date: 2026-03-29

### Session 1: Agent X — Discovery (10:00-11:30)
- Completed: FR-CORE-001 research
- Blockers: None
- Next: Start FR-CORE-002

### Session 2: Agent Y — Implementation (11:30-13:00)
- Completed: FR-AUTH-001 implementation
- Tests: 42/50 passing
- Blockers: Waiting on design review for FR-API-002
```

**Purpose**: Transparent activity log for process visibility.

### 1.3 Specification Dashboard

**Location**: AgilePlus dashboard (web UI)

**Provides**:
- **Spec Status**: PRD coverage, ADR acceptance rate, FR implementation %
- **Traceability Matrix**: PRD → FR → Tests → Code (forward and reverse)
- **Test Coverage**: Per-FR test counts and coverage %
- **Worklog Timeline**: Phase progress, blockers, velocity metrics
- **Health Checks**: Orphan tests, orphan FRs, failing CI checks

### 1.4 Governance Hooks

#### 1.4.1 FR Traceability Hook

**When**: On commit/push

**Validates**:
- Every FR-XXX-NNN in FUNCTIONAL_REQUIREMENTS.md has ≥1 test referencing it
- Every test references exactly 1 FR-XXX-NNN
- Test file exists and is syntactically valid

**Blocks**: Commit if orphan FR or orphan test detected (unless suppressed)

#### 1.4.2 Specification Verification Hook

**When**: On PR open/update

**Verifies**:
- PRD and ADR are well-formed (valid Markdown)
- All FRs are linked to PRD stories or ADRs
- No circular dependencies in PLAN.md DAG
- All user journeys map to FRs

**Reports**: Pass/fail with detailed gap list

#### 1.4.3 Worklog Integration

**When**: Phase completion

**Auto-generates**:
- Worklog summary (time spent, WPs completed)
- Phase velocity metrics
- Burndown chart
- Blockers/risks report

---

## Section 2: Consumed by Remotes

Projects using the Governance Chassis must provide:

### 2.1 Root-Level Spec Files

**Required files** (at project root):
- `PRD.md` — Product requirements
- `ADR.md` — Architecture decisions index
- `FUNCTIONAL_REQUIREMENTS.md` — Granular requirements with FR-XXX-NNN IDs
- `PLAN.md` — Phased implementation plan with DAG
- `USER_JOURNEYS.md` — End-to-end user flows

**Format**: Standard Markdown, UTF-8 encoding, no suppressions/comments.

### 2.2 Test File Tagging

Every test MUST reference an FR-XXX-NNN ID. Use one of:

#### Python (pytest)

```python
import pytest

@pytest.mark.requirement("FR-AUTH-001")
def test_user_login_success():
    """Test user login with valid credentials.

    Traces to: FR-AUTH-001
    """
    # Test body
```

#### TypeScript/JavaScript

```typescript
describe("FR-AUTH-001: User login", () => {
  it("should authenticate with valid credentials", () => {
    // Test body
  })
})
```

#### Rust

```rust
#[test]
fn test_fr_auth_001_user_login() {
    // Test body
}
```

#### BATS (Bash)

```bash
@test "FR-AUTH-001: User login succeeds" {
  # Test body
}
```

### 2.3 Worklog Entries

Create per-phase worklog files:

```
docs/worklogs/
├── PHASE1-DISCOVERY-worklog.md
├── PHASE2-DESIGN-worklog.md
├── PHASE3-IMPLEMENTATION-worklog.md
└── PHASE4-TESTING-worklog.md
```

**Format** (see Section 1.2 above):
- Date headers
- Per-session entries (Agent name, time, completed work, blockers)
- Cross-references to FRs and PRs

### 2.4 Trackers in `docs/reference/`

Maintain these trackers to show progress:

#### `docs/reference/PRD_TRACKER.md`

```markdown
# PRD Tracker

| Epic | Status | Stories | Assigned To | Notes |
|------|--------|---------|-------------|-------|
| E1.1 | 75% | 3/4 | Agent-A | 1 story in review |
| E1.2 | 0% | 0/2 | Unassigned | Planned for P2 |
```

#### `docs/reference/ADR_STATUS.md`

```markdown
# ADR Status

| ADR | Title | Status | Implemented | Code Location |
|-----|-------|--------|-------------|---------------|
| ADR-001 | Hexagonal Architecture | Accepted | 80% | src/core/ports.rs |
| ADR-002 | Error Handling | Proposed | 0% | - |
```

#### `docs/reference/FR_TRACKER.md`

```markdown
# FR Implementation Tracker

| FR ID | Title | Status | Tests | Coverage | Code |
|-------|-------|--------|-------|----------|------|
| FR-AUTH-001 | User Login | ✅ Complete | 12 | 92% | src/auth/login.rs |
| FR-AUTH-002 | Token Refresh | 🔄 In Progress | 5/8 | 62% | src/auth/token.rs |
```

#### `docs/reference/CODE_ENTITY_MAP.md`

Forward mapping (code → requirements):

```markdown
# Code Entity Map

## src/auth/login.rs

### pub async fn login(credentials: Credentials) -> Result<Token>
- Implements: FR-AUTH-001, FR-AUTH-002
- Tests: tests/auth_test.rs::test_login_success (5), test_login_failure (3)
- ADR: ADR-005 (error handling)
```

Reverse mapping (requirements → code):

```markdown
## FR-AUTH-001: User Login

- **Implementation**: src/auth/login.rs::login() [lines 42-87]
- **Tests**: tests/auth_test.rs::test_login_success(), test_login_failure()
- **Coverage**: 12 tests, 92% coverage
```

### 2.5 AgilePlus Spec Directory (Optional)

If using AgilePlus dashboard:

```
.agileplus/specs/
├── {feature-id}/
│   ├── spec.md           # Feature description
│   ├── meta.json         # ID, title, status
│   └── tasks.md          # Work packages (WP01, WP02, ...)
├── {another-feature}/
│   ├── spec.md
│   ├── meta.json
│   └── tasks.md
```

Use `agileplus specify --title "Feature" --description "Desc"` to scaffold.

---

## Section 3: Integration Points

### 3.1 AgilePlus CLI Commands

**Check for spec before implementing**:
```bash
agileplus list --status proposed
agileplus view {feature-id}
```

**Create spec for new work**:
```bash
agileplus specify --title "Auth: Two-Factor Authentication" --description "Add TOTP and SMS support"
```

**Update work package status**:
```bash
agileplus status {feature-id} --wp WP01 --state completed
agileplus status {feature-id} --wp WP02 --state in_progress
```

**View worklog**:
```bash
agileplus worklog --phase PHASE3
```

### 3.2 File & Directory Paths

| What | Path | Notes |
|------|------|-------|
| Spec files | `.agileplus/specs/{feature-id}/` | Auto-scaffolded by `agileplus specify` |
| PRD | `{root}/PRD.md` | Main spec file |
| ADR index | `{root}/ADR.md` | Points to docs/adr/{nnn}-title/ |
| FR list | `{root}/FUNCTIONAL_REQUIREMENTS.md` | Single source of truth |
| Plan | `{root}/PLAN.md` | Phased WBS with DAG |
| Journeys | `{root}/USER_JOURNEYS.md` | End-to-end flows |
| Worklogs | `docs/worklogs/{PHASE}-worklog.md` | Per-phase entries |
| Trackers | `docs/reference/{TRACKER}.md` | PRD, ADR, FR, PLAN, CODE_ENTITY trackers |

### 3.3 Code Examples: FR Tagging

**Python Example**:

```python
# tests/auth_test.py

import pytest
from src.auth.login import login
from src.auth.credentials import Credentials

@pytest.mark.requirement("FR-AUTH-001")
class TestUserLogin:
    """Test suite for user authentication.

    Traces to: FR-AUTH-001
    """

    def test_login_with_valid_credentials(self):
        """Login succeeds with correct email and password."""
        creds = Credentials(email="user@example.com", password="correct")
        result = login(creds)
        assert result.token is not None

    @pytest.mark.requirement("FR-AUTH-002")
    def test_login_failure_invalid_password(self):
        """Login fails with incorrect password.

        Traces to: FR-AUTH-002 (error handling)
        """
        creds = Credentials(email="user@example.com", password="wrong")
        with pytest.raises(AuthError):
            login(creds)
```

**TypeScript Example**:

```typescript
// tests/auth.spec.ts

describe("FR-AUTH-001: User Login", () => {
  it("should authenticate with valid credentials", () => {
    const result = login({ email: "user@example.com", password: "correct" })
    expect(result.token).toBeDefined()
  })

  describe("FR-AUTH-002: Error Handling", () => {
    it("should reject invalid password", () => {
      expect(() =>
        login({ email: "user@example.com", password: "wrong" })
      ).toThrow(AuthError)
    })
  })
})
```

### 3.4 Worklog Entry Template

```markdown
# PHASE3 — Implementation Worklog

## 2026-03-29

### Session 1: Agent-Impl-A — Core Module (09:00-12:00, 3h)

**Completed**:
- [ ] FR-AUTH-001: User login endpoint (WP03-001)
  - Implemented: src/auth/handlers.rs
  - Tests: 8 passing
- [ ] FR-AUTH-002: Error handling
  - Implemented: src/error/mod.rs
  - Coverage: 85%

**In Progress**:
- [ ] FR-API-001: OpenAPI schema

**Blockers**:
- Waiting on DB schema review from Architecture team

**Next Session**: Continue FR-API-001, resolve blocker on schema

---

### Session 2: Agent-Test-B — Testing (14:00-17:00, 3h)

**Completed**:
- [ ] FR-AUTH-001: Integration tests (20 tests)
- [ ] FR-AUTH-002: Error scenario matrix (15 tests)

**Coverage Summary**:
- FR-AUTH-001: 92% (12/13 acceptance criteria tested)
- FR-AUTH-002: 88% (7/8 error paths tested)

**Blockers**: None

**Next Session**: E2E tests for login flow
```

### 3.5 PLAN.md DAG Example

```markdown
# Implementation Plan

## Phases & Tasks

| Phase | Task | Depends On | Status | WP | Notes |
|-------|------|-----------|--------|-----|-------|
| P1 | Stakeholder interviews | - | ✅ | - | Completed W1 |
| P2 | Auth architecture | P1 | 🔄 | - | In progress |
| P3 | Login endpoint | P2 | ⏳ | WP03-001 | Blocked on P2 |
| P3 | Token refresh | P2 | ⏳ | WP03-002 | Blocked on P2 |
| P4 | Integration tests | P3 | ⏳ | WP04-001 | Blocked on P3 |
| P4 | E2E tests | P3 | ⏳ | WP04-002 | Blocked on P3 |

## Dependencies

```
P1 → P2 → P3 → P4
            ├─ WP03-001 (Login)
            └─ WP03-002 (Token Refresh)
                    ↓
             P4 → WP04-001 (Integration)
                  WP04-002 (E2E)
```
"

---

## Section 4: Success Criteria

### 4.1 Specification Completeness

Projects using the Governance Chassis must have:

- [ ] `/PRD.md` with ≥3 epics, each with ≥2 stories
- [ ] `/ADR.md` referencing ≥2 architecture decisions in `docs/adr/`
- [ ] `/FUNCTIONAL_REQUIREMENTS.md` with ≥10 FR-{CAT}-NNN entries
- [ ] `/PLAN.md` with ≥4 phases and explicit DAG (no cycles)
- [ ] `/USER_JOURNEYS.md` with ≥3 end-to-end user flows
- [ ] `docs/reference/CODE_ENTITY_MAP.md` (forward + reverse mappings)

### 4.2 Test Traceability

- [ ] **0 orphan FRs**: Every FR-XXX-NNN has ≥1 test
- [ ] **0 orphan tests**: Every test traces to ≥1 FR-XXX-NNN
- [ ] **FR Coverage**: ≥80% of FRs have ≥3 tests each
- [ ] **Test Tags**: All test files use @pytest.mark.requirement(), describe("FR-"), etc.

### 4.3 Worklog Compliance

- [ ] Worklog entries created per phase
- [ ] Each entry documents completed FRs and blockers
- [ ] Entries cross-reference PRs and code locations
- [ ] Phase velocity metrics tracked

### 4.4 Governance Hook Pass

- [ ] `agileplus traceability-check` shows 0 errors
- [ ] `agileplus spec-verification` shows 100% coverage
- [ ] All FRs map to stories in PRD
- [ ] All ADRs marked as "Accepted" or "Deprecated" (no "Proposed" lingering)

### 4.5 Integration Across 3+ Repos

Verified implementations in:
- [ ] AgilePlus (canonical reference)
- [ ] phenotype-shared (test consumer)
- [ ] 1+ additional repo (heliosCLI, thegent, etc.)

---

## Section 5: Extending the Chassis

### 5.1 Custom FR Categories

Projects may define their own FR category prefixes:

**Standard**:
- AUTH — Authentication & security
- CORE — Core business logic
- API — External API contracts
- UI — User interface features
- PERF — Performance optimizations

**Custom Example** (add to FUNCTIONAL_REQUIREMENTS.md header):

```markdown
# Functional Requirements

**FR Categories**:
- AUTH: Authentication & security
- CORE: Core logic
- API: API contracts
- **CUSTOM: Domain-specific features** ← Project-specific

### FR-CUSTOM-001: {Title}
...
```

### 5.2 Custom Work Package Templates

Projects can extend PLAN.md with domain-specific work packages:

```markdown
## Phase 3: Implementation

### P3.1 — Core API Endpoints
- **WP**: WP03-001, WP03-002, WP03-003
- **Dependencies**: Blocked by P2 (schema design)
- **Deliverables**: FR-API-001, FR-API-002, FR-API-003
- **Duration**: 1 week
```

### 5.3 Custom Trackers

Projects can add project-specific trackers to `docs/reference/`:

Examples:
- `PERFORMANCE_TRACKER.md` — Benchmark results per FR
- `SECURITY_AUDIT.md` — Security review status per FR
- `DEPENDENCY_MAP.md` — External libraries and versions

---

## Section 6: FAQ

**Q: Does every test need a unique FR?**
A: No. Multiple tests can trace to one FR (e.g., 10 tests for FR-AUTH-001). But every test must trace to at least one FR.

**Q: What if I have a legacy project without FRs?**
A: Start with Phase 1 (Discovery) and retrofit FRs as you document requirements. Chassis is flexible.

**Q: Can I use AgilePlus specs instead of FUNCTIONAL_REQUIREMENTS.md?**
A: AgilePlus specs and FUNCTIONAL_REQUIREMENTS.md serve different purposes. Specs are for work tracking; FRs are for requirement traceability. Use both.

**Q: What if a requirement doesn't map to an FR?**
A: All PRD stories should map to FRs. If a story exists without FRs, either create new FRs or merge the story back into the PRD.

**Q: How do I version specifications?**
A: Use git branches and tags. Each phase can be tagged (e.g., `phase-1-complete`, `phase-2-design-approved`).

---

## Related Docs

- **Phenotype Docs Chassis**: See `docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md`
- **Global CLAUDE.md**: Specification documentation system (see `~/.claude/CLAUDE.md`)
- **AgilePlus CLAUDE.md**: CLI and workflow (see `AgilePlus/CLAUDE.md`)

