---
audience: [developers, pms, agents]
---

# Checklists

Automated quality gates that validate artifacts at each lifecycle phase. Checklists act as unit tests for requirements writing, ensuring consistency between spec, plan, and implementation.

## How Checklists Work

Each feature gets three checklists generated automatically:

1. **Specification Checklist** — Validates spec before planning (prevents bad specs)
2. **Implementation Checklist** — Validates each WP during code review (prevents bad code)
3. **Acceptance Checklist** — Validates full feature before shipping (prevents incomplete shipping)

Checklists are stored at:

```
kitty-specs/{NNN}-{feature}/
├── checklists/
│   ├── specification.md
│   ├── implementation.md
│   └── acceptance.md
```

## Specification Checklist

Generated after spec is written, validated before planning begins.

```bash
agileplus checklist 001 --type specification
```

Creates `kitty-specs/001-checkout-upsell/checklists/specification.md`:

```markdown
# Specification Checklist — 001: Checkout Upsell

Validates that the specification is clear, testable, and ready for planning.

## Requirements Format

- [ ] All functional requirements numbered (FR-1, FR-2, ...)
- [ ] Each FR has a single responsibility (not compound)
- [ ] Each FR is technology-agnostic (no "use Redis" in a spec)
- [ ] Each FR is unambiguous (use precise language)
- [ ] Each FR is testable (not "should be fast")

### Examples from spec

- [x] FR-1: Users can add items to cart → Clear, testable ✓
- [x] FR-2: Cart persists across sessions → Clear, testable ✓
- [ ] FR-3: "Fast checkout" → Vague! Rewrite with metric.

## Success Criteria

- [ ] All success criteria start with measurable values (not "good")
- [ ] All success criteria are independent (not compound)
- [ ] Criteria reference specific FR numbers
- [ ] All criteria are user-facing or business-relevant

### Examples from spec

- [x] SC-1: Checkout completes in < 3 seconds (p95) → Measurable ✓
- [x] SC-2: Users see upsell 100% of the time → Clear ✓
- [ ] SC-3: "System is reliable" → Vague! Add metric.

## Architecture & Scope

- [ ] Specification avoids mentioning specific frameworks/languages
- [ ] Specification avoids mentioning specific database engines
- [ ] Specification does not prescribe implementation patterns
- [ ] Scope is clearly bounded (no "and integrate with Slack")
- [ ] If scope includes integrations, they are listed separately

### Examples from spec

- [x] "Add guest checkout flow" → Clear boundary ✓
- [ ] "Add guest checkout using OAuth2" → Too prescriptive ✗
- [x] Integrations listed separately: "Integration: Stripe payment processor"

## Completeness

- [ ] Overview section present
- [ ] User scenarios section present (at least 3 scenarios)
- [ ] Functional requirements section present
- [ ] Success criteria section present
- [ ] Edge cases section present
- [ ] Known constraints section present
- [ ] Dependencies/integrations listed

## Clarity

- [ ] Spec is < 3000 words (brief and focused)
- [ ] Acronyms defined on first use
- [ ] No unexplained jargon
- [ ] All links/references are valid
- [ ] Diagrams are clear (if present)

## Consistency

- [ ] User scenarios reference functional requirements
- [ ] All FRs are covered by at least one scenario
- [ ] Success criteria tie back to FRs
- [ ] Edge cases are independent (not overlapping)

## Ready for Planning?

Once all checks pass, spec is ready:

```bash
agileplus checklist 001 --type specification --complete
```

Marks spec as validated and unlocks the `plan` command.
```

## Implementation Checklist

Validates each work package after implementation, before acceptance.

```bash
agileplus checklist 001 --wp WP02 --type implementation
```

Creates per-WP checklist:

```markdown
# Implementation Checklist — WP02: Guest Checkout UI

Validates that the implementation meets its deliverables and the specification.

## Deliverables

All files specified in the plan must exist:

- [ ] src/handlers/checkout_guest.rs
  - [ ] GET /checkout/guest endpoint
  - [ ] POST /checkout/guest endpoint
  - [ ] Tests (>= 10 test cases)

- [ ] src/models/guest_order.rs
  - [ ] GuestOrder model with fields
  - [ ] Validation logic
  - [ ] Tests

- [ ] tests/integration/checkout_guest_flow.rs
  - [ ] Happy path: guest can complete checkout
  - [ ] Error cases: invalid input handling
  - [ ] Integration: Stripe payment flow

Result: All deliverables present? [x] YES

## Code Quality

- [x] All tests pass locally (`cargo test WP02`)
- [x] Code passes linting (`cargo clippy -- -D warnings`)
- [x] Code is formatted (`cargo fmt`)
- [x] No unwrap() in library code
- [x] Functions are < 50 lines
- [x] Test coverage >= 85%
  - Actual: 91% for this WP

## Scope Compliance

- [x] No files modified outside expected dirs (src/handlers/*, tests/*)
- [x] No dependencies added to Cargo.toml
- [x] No breaking changes to existing APIs
- [x] Works in isolation (doesn't require other WPs)

## Specification Alignment

- [x] FR-1 (Add items as guest) — Implemented and tested
- [x] FR-2 (Guest checkout completes) — Implemented and tested
- [x] SC-1 (Checkout < 3s p95) — Tested and passes
- [x] SC-2 (All guests see upsell) — UI implemented
- [x] Edge case: empty cart → Handled, returns 400

## Commit Quality

- [x] All commits reference WP02
  - "feat(WP02): implement guest checkout handler"
  - "test(WP02): add guest order tests"

- [x] Commit messages explain WHY
  - Good: "Rate limit guest checkout to prevent abuse"
  - Bad: "Add checkout code"

## Documentation

- [x] Public functions have rustdoc
- [x] Non-obvious logic has comments
- [x] Tests are clear (names explain what they test)

## Ready for Acceptance?

All checks pass: Ready for merge! ✓

Merge: `git merge --no-ff feat/001-WP02` → main
```

## Acceptance Checklist

Validates the complete feature before shipping.

```bash
agileplus checklist 001 --type acceptance
```

```markdown
# Acceptance Checklist — 001: Checkout Upsell

Final gate before merging to main and shipping.

## Completeness

- [x] All 4 WPs complete and merged
  - WP01: Guest models ✓
  - WP02: Guest checkout UI ✓
  - WP03: Upsell widget ✓
  - WP04: Integration tests ✓

- [x] All functional requirements have implementation
  - FR-1 (guest checkout) → WP02 ✓
  - FR-2 (upsell display) → WP03 ✓
  - FR-3 (payment processing) → WP02 ✓

- [x] All success criteria met and tested
  - SC-1 (< 3s checkout) — E2E tests pass ✓
  - SC-2 (upsell shown) — UI tests verify ✓
  - SC-3 (error handling) — Error path tests pass ✓

## Quality

- [x] All tests pass (42 tests, 0 failures)
- [x] Test coverage: 87% (target: 85%) ✓
- [x] Code quality: 0 clippy warnings ✓
- [x] Performance: All E2E tests under 3 seconds ✓

## Governance

- [x] All commits use proper message format ✓
- [x] No security vulnerabilities detected ✓
- [x] No breaking changes to existing APIs ✓
- [x] All dependencies reviewed and approved ✓

## Documentation

- [x] README.md updated with guest checkout feature
- [x] API documentation (OpenAPI) updated
- [x] Changelog entry prepared
- [x] Migration notes (if needed) prepared

## Ready to Ship?

All checks pass: Feature is ready for production! ✓

Ship: `agileplus ship 001`
```

## Project-Specific Checklists

Add custom checklist items via the constitution. These apply to all features:

```yaml
# .kittify/memory/constitution.md

custom_checklists:
  implementation:
    - "[ ] Accessibility audit (WCAG 2.1 AA) for UI changes"
    - "[ ] Performance testing (load test >100 concurrent)"
    - "[ ] Security review for auth/payment code"

  acceptance:
    - "[ ] Product review by PM"
    - "[ ] Marketing messaging ready"
    - "[ ] Customer support docs updated"
```

These items are automatically added to the generated checklists for each feature.

## Running Checklists

### View Checklist Status

```bash
agileplus checklist 001 --status
```

```
Checklist Status for 001: Checkout Upsell

Specification Checklist
  Progress: 21/22 items complete (95%)
  Status: READY
  Next: Submit for planning

Implementation Checklist
  WP01: 18/18 complete (100%) — APPROVED
  WP02: 17/18 complete (94%) — IN PROGRESS
    ✗ Missing: Upsell widget A/B tests
  WP03: 13/18 complete (72%) — BLOCKED
  WP04: 0/18 complete (0%) — NOT STARTED

Acceptance Checklist
  Progress: 12/20 items complete (60%)
  Status: AWAITING WP COMPLETION
```

### Mark Item Complete

```bash
agileplus checklist 001 WP02 --item "Upsell widget tests" --complete
```

### Generate Report

```bash
agileplus checklist 001 --report
```

Generates `kitty-specs/001/checklists/report.md` with summary and metrics.

## Checklist as Governance

Checklists are enforced in the workflow:

1. **Spec checklist** must be 100% before `agileplus plan` runs
2. **Implementation checklist** blocks `agileplus review APPROVE`
3. **Acceptance checklist** blocks `agileplus ship` unless all items complete

Try to plan without specification checklist:

```bash
agileplus plan 001
```

```
Error: Specification checklist incomplete (21/22)
  Missing: Success criteria must be measurable (SC-3)

Requirement: Fix checklist before planning.
  agileplus checklist 001 --type specification --edit
```

## Tips for Checklists

1. **Be specific** — Generic items are hard to validate (avoid "code is good")
2. **Reference artifacts** — Link to specs, code files, tests
3. **Make items binary** — "Done" or "Not done" (avoid "mostly done")
4. **Update during development** — Don't wait until the end to fill them out
5. **Use for learning** — Patterns in checklist failures reveal process gaps
