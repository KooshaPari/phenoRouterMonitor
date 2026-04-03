---
audience: [developers, pms]
---

# Analyze

Non-destructive cross-artifact consistency and quality check. Analyzes relationships between specification, plan, and implementation to catch gaps early.

## What It Does

Analyzes three layers of artifacts:

1. **Specification layer** — requirements, success criteria, edge cases
2. **Planning layer** — work packages, architecture, dependencies
3. **Implementation layer** — actual code, tests, deliverables

Checks for:
- **Consistency** — do plan and code match the spec?
- **Completeness** — are all requirements covered?
- **Dependency correctness** — are WP dependencies valid?
- **Quality** — do artifacts meet constitutional standards?
- **Coverage gaps** — what's specified but not planned/implemented?
- **Scope creep** — what's implemented but not in spec?

## Usage

```bash
agileplus analyze 001
```

Scans all artifacts in `kitty-specs/001-feature/` and generates a report.

## Output Example

```
Feature 001: Checkout Upsell

├─ CONSISTENCY CHECK
│  ├─ [✓] Spec requirements → WP mapping
│  │   All 12 FRs mapped to at least one WP
│  │   FR-1 (guest checkout) → WP02
│  │   FR-2 (upsell widget) → WP03
│  │   ... all 12 requirements covered
│  │
│  ├─ [✓] WP architecture → Implementation
│  │   WP02 deliverables match actual code files
│  │   WP03 plan matches realized widget structure
│  │
│  └─ [⚠] Plan mentions "email notifications"
│      ├─ Location: plan.md, WP04 description
│      ├─ Issue: Not in specification
│      ├─ Type: Scope creep
│      └─ Recommendation: Remove from WP04 or add to spec
│
├─ COMPLETENESS CHECK
│  ├─ Functional Requirements
│  │   Total: 12
│  │   Covered by WPs: 12 (100%) ✓
│  │   Covered by tests: 12 (100%) ✓
│  │
│  ├─ Success Criteria
│  │   Total: 5
│  │   With measurable tests: 4 (80%)
│  │   ⚠ SC-5: "Upsell conversion rate improves"
│  │     └─ Issue: No test data/baseline provided
│  │     └─ Recommendation: Add A/B test plan to spec
│  │
│  └─ Edge Cases
│      Total: 6
│      Covered: 5 (83%)
│      ⚠ Missing test: Guest checkout w/ expired promo code
│
├─ DEPENDENCY ANALYSIS
│  ├─ [✓] No circular dependencies detected
│  ├─ [✓] All "blocked by" relationships valid
│  ├─ [✓] Critical path correct: WP01 → WP02 → WP04
│  └─ [⚠] WP04 depends on WP02 and WP03
│      ├─ Can be parallelized? No (sequential required)
│      ├─ Critical path impact: +1 week
│      └─ Recommendation: Consider moving integration tests to WP02/WP03
│
├─ QUALITY ANALYSIS
│  ├─ Specification Checklist: 21/22 (95%) ✓
│  │   └─ Missing: SC-5 baseline data
│  │
│  ├─ Implementation Checklists
│  │   WP01: 18/18 (100%) ✓
│  │   WP02: 17/18 (94%) ✓
│  │   WP03: 15/18 (83%) ⚠
│  │   WP04: 18/18 (100%) ✓
│  │
│  ├─ Code Quality
│  │   Test coverage: 87% (target: 85%) ✓
│  │   Clippy warnings: 0 ✓
│  │   Linting violations: 0 ✓
│  │
│  └─ Documentation
│      All FRs documented: ✓
│      All APIs documented: ✓
│      Edge cases documented: ⚠ (3/6 missing notes)
│
└─ RISK ASSESSMENT
   ├─ Green (Ready)
   │  └─ WP01 and WP02 ready for production
   ├─ Yellow (Minor Issues)
   │  └─ WP03: Missing A/B test baseline
   │  └─ WP04: Integration test coverage at 80%
   └─ Red (Blocking)
      └─ None detected
```

## Detailed Checks

### Consistency Check

Verifies that requirements flow through the pipeline:

#### Spec → Plan

```
For each Functional Requirement (FR):
  1. Is it mentioned in at least one WP plan?
  2. Does the WP architecture support implementing it?
  3. Are the deliverables sufficient to implement it?
```

Example analysis:

```
FR-1: "Users can add items to cart"
  ✓ Mentioned in WP02 plan
  ✓ WP02 architecture (guest_cart.rs) supports it
  ✓ Deliverables include cart data model and API
  Status: COVERED
```

#### Plan → Implementation

```
For each Work Package:
  1. Do the specified deliverables exist in code?
  2. Does the architecture match the plan?
  3. Are dependencies correctly declared?
```

Example:

```
WP02: Guest Checkout UI
  Deliverables specified:
    ✓ src/handlers/checkout_guest.rs (exists, 180 lines)
    ✓ src/models/guest_order.rs (exists, 95 lines)
    ✓ tests/checkout_guest_test.rs (exists, 220 lines)
  Status: ALL PRESENT
```

### Completeness Check

Identifies gaps between spec and implementation:

#### What's Specified But Not Planned?

```
Scan spec → Plan:
  All 12 FRs found in plan: ✓
  All 5 SCs found in plan: ✓
  All 6 edge cases found in plan: ✓
```

#### What's Specified But Not Tested?

```
For each FR, check if implementation has tests:
  FR-1 (add to cart) — tests: checkout_guest_test.rs:23 ✓
  FR-2 (upsell display) — tests: upsell_widget_test.rs ✓
  ...all 12 FRs have tests
```

#### What's Implemented But Not Specified?

```
Scan code → Spec:
  Feature: "Rate limiting on guest checkout"
    ├─ Code: src/handlers/checkout_guest.rs:45 (rate limiter)
    ├─ Found in spec? ✗ NOT FOUND
    ├─ Type: Scope creep
    └─ Action: Add to spec or remove from code
```

### Dependency Analysis

Validates work package dependency declarations:

#### Circular Dependencies?

```
WP dependency graph:
  WP01 → WP02 ✓
  WP02 → WP03 ✓
  WP03 → WP04 ✓
  WP04 → (none)

Cycles detected: 0 ✓
```

#### Valid Blockers?

```
WP02 blocked by: [WP01]
  ✓ WP01 delivers models required by WP02
  ✓ Valid blocker

WP04 blocked by: [WP02, WP03]
  ✓ WP02 delivers integration tests
  ✓ WP03 delivers upsell widget
  ✓ Valid blockers
```

#### Critical Path?

```
Critical path analysis:
  Path 1: WP01 (2d) → WP02 (3d) → WP04 (2d) = 7 days
  Path 2: WP01 (2d) → WP03 (3d) → WP04 (2d) = 7 days

  Parallelizable:
    WP02 and WP03 can run in parallel (both depend on WP01)

  Critical path: 7 days
  Sequential estimate: 10 days
  Parallelization saves: 3 days
```

### Quality Analysis

Checks against the constitution:

```
Code Quality Metrics
  ├─ Test coverage (target >= 85%)
  │   Current: 87% ✓
  │
  ├─ Clippy warnings (target 0)
  │   Current: 0 ✓
  │
  ├─ Function length (target <= 50 lines)
  │   Max found: 48 lines ✓
  │
  └─ Code duplication (target < 10%)
      Current: 3.2% ✓
```

## When to Use

Run analyze:

1. **After generating plan** (before implementation) — validate architecture
2. **After major spec changes** — check impact on plan
3. **Before acceptance** — verify completeness
4. **Weekly health check** — monitor ongoing consistency
5. **Before shipping** — final sanity check

## Analysis by Phase

### Before Planning

```bash
agileplus analyze 001 --phase spec
```

Validates specification alone:
- Checklist completion
- Requirement clarity
- No implementation details

### Before Implementation

```bash
agileplus analyze 001 --phase plan
```

Validates spec + plan:
- All FRs mapped to WPs
- Reasonable architecture
- Dependency correctness

### Before Acceptance

```bash
agileplus analyze 001 --phase full
```

Validates spec + plan + code:
- All FRs implemented
- All tests passing
- No scope creep

## Interpreting Results

### Green Status
✓ Feature is well-defined and ready
- All checks pass
- No gaps or inconsistencies
- Ready to proceed

### Yellow Status
⚠ Feature has minor issues
- Non-blocking gaps (e.g., documentation missing)
- Recommendations for improvement
- Can proceed; fix before shipping

### Red Status
✗ Feature has blocking issues
- Required work is missing
- Gaps that prevent shipping
- Must fix before proceeding

## Output Formats

### Terminal (default)

```bash
agileplus analyze 001
```

Pretty-printed to terminal with colors and tree structure.

### JSON

```bash
agileplus analyze 001 --format json
```

Parseable JSON output for CI/CD integration.

### HTML Report

```bash
agileplus analyze 001 --format html --output /tmp/report.html
```

Generates interactive HTML report (opens in browser).

## Analyze is Non-Destructive

**Important**: `analyze` never modifies artifacts. It only reports findings.

To act on findings, you must manually:
- Edit the spec to add missing requirements
- Edit the plan to cover gaps
- Edit the code to implement discovered gaps

## Key Takeaways

1. **Consistency matters** — catch misalignment early
2. **Run before major gates** — plan, implementation, acceptance
3. **Act on findings** — don't ignore yellow/red status
4. **Parallelize where possible** — identify independent WPs
5. **Track quality trends** — run analyze weekly to see trends
