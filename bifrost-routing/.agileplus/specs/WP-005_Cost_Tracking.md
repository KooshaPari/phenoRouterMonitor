# WP-005: Cost Tracking & Budget Enforcement

**Work Package ID**: WP-005
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 1
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Track cost per LLM request and enforce budget caps per user/project.

## Description

Calculate request cost from token count and model pricing. Prevent overspending with hard budget enforcement.

---

## Objectives

- Track cost per LLM request (input tokens × model cost)
- Implement user/project budget caps (monthly + daily)
- Enforce budget (return 429 when exceeded)
- Expose cost tracking via API and CLI

---

## Acceptance Criteria

1. **Cost Tracking**:
   - Cost accurate: 1000 tokens @ Haiku ($0.80/1M) = $0.0008
   - `/budget/check` returns remaining budget
   - Cost ledger auditable

2. **Budget Enforcement**:
   - User exceeding budget gets 429
   - Budget enforced: sum last 30 days

3. **Testing**:
   - Cost calculation tests pass
   - Budget enforcement tests pass

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| CostTracker | Cost calculation logic | Accurate |
| SQLite schema | Cost ledger | Queries work |
| /budget/check API | HTTP endpoint | JSON output |
| CLI commands | bifrost budget set/show | Works |
| Enforcement | Budget cap enforcement | Blocks overspend |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-002 (Model Registry)
- WP-003 (Model Selection)
- WP-004 (SLA Enforcement)

**Blocks**:
- WP-006 (A/B Testing)

---

## Effort Estimate

- **Estimated LOC**: 360
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T031: Create bifrost-routing-analytics/src/cost_tracker.rs
- [ ] T032: Implement cost calculation
- [ ] T033: Add SQLite schema for cost ledger
- [ ] T034: Implement budget enforcement
- [ ] T035: Create HTTP endpoint POST /budget/check
- [ ] T036: Create CLI command bifrost budget set
- [ ] T037: Create CLI command bifrost budget show
- [ ] T038: Unit tests
- [ ] T039: Integration test

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
