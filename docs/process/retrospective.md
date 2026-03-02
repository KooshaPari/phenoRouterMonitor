---
audience: [developers, pms]
---

# Retrospectives

Structured reflection on completed features to capture learnings and improve the development process. Retrospectives feed insights back into the constitution and inform planning for future work.

## When to Run

Retrospectives are run **after shipping and collecting 1 week of production data**:

```bash
# Week 1: Feature ships
agileplus ship 001

# Week 2: Collect data (any bugs? user feedback?)
# Then run retrospective
agileplus retrospective 001
```

## Process

### 1. Gather Data

Collect facts about the feature's lifecycle:

```bash
agileplus status 001 --history
```

```
Feature 001: Checkout Upsell — Lifecycle Report

TIMELINE
  Specification created:      2026-02-01 09:00
  Specification completed:    2026-02-02 14:30 (1.6 days)
  Planning completed:         2026-02-03 17:00 (1.2 days)
  First WP started:           2026-02-04 08:00
  All WPs complete:           2026-02-12 16:45
  Feature shipped:            2026-02-13 10:00
  Total elapsed: 12 calendar days (9 business days)

WORK PACKAGE BREAKDOWN
  WP01 (Guest models)
    Duration:     2 business days
    Agent:        claude-code
    Review cycles: 1
    Status:       APPROVED (first review)

  WP02 (Checkout UI)
    Duration:     3 business days
    Agent:        claude-code
    Review cycles: 2
    Status:       APPROVED (second review)
    Issue:        Missing error handling in first pass

  WP03 (Upsell widget)
    Duration:     3 business days
    Agent:        codex
    Review cycles: 1
    Status:       APPROVED (first review)

  WP04 (Integration tests)
    Duration:     2 business days
    Agent:        claude-code
    Review cycles: 1
    Status:       APPROVED (first review)

METRICS
  Total commits:       16
  Total files changed: 24
  Lines added:         687
  Test coverage:       87% (target: 85%)
  Critical bugs found: 0
  Hotfixes needed:     0

PARALLELIZATION
  Sequential estimate (all WPs serial):    10 days
  Actual (WP02 & WP03 parallel):          9 days
  Parallelization benefit:                 1 day saved (10%)

AGENT PERFORMANCE
  claude-code:
    WPs completed:     3
    Pass rate:         100% (approved first review)
    Avg completion:    2.3 days

  codex:
    WPs completed:     1
    Pass rate:         100% (approved first review)
    Avg completion:    3 days

ISSUE TRACKER SYNC
  GitHub issues created:  2
  GitHub PRs opened:      4
  Plane issues synced:    12
  Sync reliability:       100%

QUALITY METRICS
  Spec accuracy:         95% (no major requirement changes)
  Rework rate:           25% (WP02 sent back once)
  Build pass rate:       100%
  Test flakiness:        0%
```

### 2. Identify Patterns

Analyze data to find patterns:

#### Went Well (What Should We Keep Doing?)

| Item | Evidence | Impact |
|------|----------|--------|
| Specification-first approach | 95% spec accuracy; minimal changes during implementation | Prevented scope creep, reduced rework |
| Parallelization of WP02/WP03 | Completed 1 day faster than sequential estimate | Saved resources |
| Agent pass rate | 100% first-review approval rate across 4 WPs | Agents producing quality work |
| Clear deliverables | All WP checklists passed first time (except WP02) | Agents knew exactly what to build |

#### Could Improve (What Bottlenecks?)

| Item | Evidence | Impact | Cause |
|------|----------|--------|-------|
| WP02 rework | Sent back once for missing error handling | +0.5 days | Spec lacked edge case coverage (guest checkout w/ failed payment) |
| Review cycle time | 4 hours average review time | Acceptable | Reviews require human review + test execution |
| Spec-to-plan duration | 1.2 days | Reasonable | Planning tool could be faster |

#### Action Items (Constitution Improvements)

1. **Add edge case: "Failed payment in guest checkout"**
   - Where: Constitution checklist for spec validation
   - Why: WP02 missed this; caused rework
   - Action: Add to specification checklist: "All payment flows include failure cases"

2. **Tighten implementation checklist for error handling**
   - Where: Constitution custom checklist items
   - Why: WP02 first pass lacked error handling
   - Action: Add: "[ ] All API endpoints validate input and handle errors"

3. **Consider faster review automation**
   - Where: CI/CD pipeline
   - Why: Review cycle time is longest phase after implementation
   - Action: Add automated code review tools; human reviews remain but with pre-filtering

### 3. Update Constitution

Encode learnings into the constitution:

```bash
agileplus constitution --update
```

```
CONSTITUTION UPDATE: Feature 001 Retrospective

New Specification Checklist Item:
  [ ] All payment flows include both success and failure paths
    Reason: WP02 rework revealed missing edge case

New Implementation Checklist Item:
  [ ] All API endpoints validate input and return meaningful errors
    Reason: WP02 initially lacked error handling

New Quality Target:
  Review cycle time target: < 3 hours (from 4 hours)
    Action: Enable faster CI; reviewers focus on logic, not lint

New Agent Constraint:
  When implementing payment/auth code:
    - Must include happy path AND error cases
    - Must test invalid input scenarios
    - Requires explicit WP callout: "Include error handling"
```

These changes apply to all future features.

## Metrics & Tracking

### Key Metrics

| Metric | Calculation | Unit | Why It Matters |
|--------|-------------|------|-----------------|
| **Cycle Time** | Ship date - Spec date | Days | Measures overall velocity |
| **Phase Time** | Time in each phase | Days | Identifies bottlenecks |
| **Review Cycles** | Times WP went back to "doing" | Count | Measures spec/plan quality |
| **WP Throughput** | WPs completed per day | WPs/day | Agent productivity |
| **Test Coverage** | Lines covered / total lines | % | Code quality proxy |
| **Spec Accuracy** | 1 - (spec changes / original reqs) | % | Planning quality |
| **Agent Pass Rate** | WPs approved on first review / total | % | Agent performance |
| **Rework Rate** | WPs sent back / total WPs | % | Implementation quality |

### Trends Across Features

Track metrics across multiple features to identify trends:

```bash
agileplus metrics --compare 001 002 003 --chart
```

```
Cycle Time Trend (Specify → Ship)
  001 (Checkout Upsell):  9 days
  002 (Guest Auth):       8 days
  003 (Payment Webhooks): 7 days

  Trend: ↓ Getting faster (improvements working!)

Phase Time Distribution
  Specification: ~1-2 days (constant)
  Planning:      ~1 day (constant)
  Implementation: ~5-6 days (↓ improving)
  Review/Ship:   ~1 day (constant)

Test Coverage
  001: 87%
  002: 89%
  003: 91%

  Trend: ↑ Improving (stricter checklist working)

Agent Effectiveness
  Feature 001: 4 WPs, 100% pass rate, 2.3 days/WP
  Feature 002: 3 WPs, 100% pass rate, 2.5 days/WP
  Feature 003: 5 WPs, 100% pass rate, 2.1 days/WP

  Trend: Agents getting better at first-pass quality
```

## Storing Retrospectives

Retrospective notes are stored in the feature directory:

```
kitty-specs/001-checkout-upsell/
├── spec.md
├── plan.md
├── tasks.md
├── checklists/
└── retrospective.md        ← Generated here
```

Content:

```markdown
# Retrospective — 001: Checkout Upsell

Feature shipped 2026-02-13. Retrospective conducted 2026-02-20.

## Executive Summary

Feature shipped on time (9 days, on estimate). One rework cycle (WP02 error handling).
Quality excellent: 0 critical bugs in first week. Learnings feed into constitution updates.

## Timeline

| Phase | Start | End | Duration | Status |
|-------|-------|-----|----------|--------|
| Specification | 2/1 | 2/2 | 1.6 days | On time |
| Planning | 2/2 | 2/3 | 1.2 days | On time |
| Implementation | 2/4 | 2/12 | 8 days* | 1 day rework (WP02) |
| Review & Ship | 2/12 | 2/13 | 1 day | On time |
| **Total** | | | **9 days** | **On estimate** |

*WP02 took 3.5 days (1 day rework for error handling)

## What Went Well

### Specification-First Prevented Scope Creep
- Spec was detailed and clear
- 95% accuracy: only 1 requirement clarified during implementation
- No scope creep items added during development
- **Impact**: Saved ~2 days of unplanned work

### Agent Quality Improved
- 4 WPs, 4 first-pass approvals (100% pass rate)
- Agents understood specifications and delivered to plan
- Minimal human intervention needed
- **Impact**: On-time delivery

### Parallelization Worked
- WP02 (checkout) and WP03 (upsell) ran in parallel
- Both completed within 1 day of each other
- Saved ~1 day vs. sequential execution
- **Impact**: Feature completed faster

## What Could Improve

### WP02 Error Handling Gap
- **Issue**: First implementation lacked error handling for failed payments
- **Cause**: Spec didn't explicitly call out failure paths
- **Fix**: Spec checklist now requires "all payment flows include failure cases"
- **Impact**: Prevented second rework cycle

### Review Cycle Time
- Average: 4 hours from submission to approval
- Bottleneck: Manual code review (human time-limited)
- **Action**: Automate lint/formatting checks in CI; humans focus on logic
- **Impact**: Target review time reduced to < 3 hours

## Lessons Learned

1. **Edge cases matter** — Failed payment path was as important as happy path
   → Add to future spec checklists

2. **Spec clarity drives speed** — 95% accuracy meant minimal rework
   → Invest time in spec writing

3. **Parallelization pays off** — Saved 1 day by running WP02/WP03 together
   → Continue parallelizing independent WPs

4. **Agent performance is reliable** — 100% pass rate across 4 WPs
   → Can increase agent responsibilities with confidence

## Constitution Updates

Based on this retrospective, the following updates were made:

1. **Spec Checklist**
   - Added: "All payment/auth flows include both happy and error paths"
   - Added: "Edge cases include system failures (network, database, service timeouts)"

2. **Implementation Checklist**
   - Added: "All API endpoints validate input and return meaningful errors"
   - Added: "Payment code includes both success and failure test cases"

3. **Process**
   - Created automated linting gate in CI (reduces review time)
   - Increased parallelization guidance (identify independent WPs early)

## Metrics for Next Feature

Use these targets from this feature:
- **Cycle time**: 9 days (set as target for similar features)
- **Test coverage**: 87% (continue targeting 85%+)
- **Agent pass rate**: 100% (continue expecting first-pass quality)
- **Review time**: <3 hours (new target, down from 4)

## Appendix: Raw Data

[Detailed timeline, commit logs, test coverage breakdown, agent logs]
```

## Running Retros at Different Scales

### Feature Retrospective

```bash
agileplus retrospective 001
```

Individual feature analysis (as described above).

### Sprint Retrospective

```bash
agileplus retrospective --sprint SPRINT-09
```

Aggregate data across all features completed in the sprint:

```
Sprint 09 Retrospective (Feb 10 – Feb 21)

Features Completed
  001: Checkout Upsell (9 days)
  002: Guest Auth (8 days)
  Total: 2 features, 17 days

Metrics
  Avg cycle time: 8.5 days
  Avg test coverage: 88%
  Agent pass rate: 100% (8 WPs)
  Zero critical bugs

Patterns
  ✓ Specification-first working well
  ✓ Agent quality consistent
  ✗ Review cycle time higher in 002 (stakeholder review blocked)

Recommendations
  → Involve stakeholders earlier (in planning, not review)
  → Continue current spec-first approach
  → Consider weekly retrospectives (not just per-feature)
```

## Tips for Effective Retrospectives

1. **Do them soon** — While memory is fresh (1-2 weeks post-ship)
2. **Use data** — Let metrics guide reflection, not just opinions
3. **Identify specific causes** — Don't just say "could improve communication"
4. **Act on findings** — Update constitution, change process, don't just record
5. **Celebrate wins** — Acknowledge what's working before improvement areas
6. **Track trends** — Compare this retro to previous ones to see progress
