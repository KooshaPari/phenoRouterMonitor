# WP-006: A/B Testing & Shadow Routing

**Work Package ID**: WP-006
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 2
**Status**: Pending
**Priority**: Medium
**Created**: 2026-03-30

---

## Overview

Implement shadow routing for A/B testing new models without affecting user traffic.

## Description

Route requests to experiment models in background, collect metrics, analyze results without impacting production.

---

## Objectives

- Shadow route requests to alternate model
- Collect metrics (latency, cost) without affecting user traffic
- Support A/B experiment configuration
- Expose shadow routing results via API

---

## Acceptance Criteria

1. **Shadow Routing**:
   - Shadow requests don't affect user response
   - Metrics collected: latency, cost, quality
   - `/invoke-with-shadow` supports sampling percentage

2. **Results**:
   - `bifrost shadow results` shows aggregated metrics
   - Avg latency, cost delta, success rate

3. **Testing**:
   - Shadow routing tests pass
   - Metric collection tests pass

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| ShadowRouter | Shadow routing logic | Trait impl |
| SQLite schema | Shadow metrics table | Queries work |
| /invoke-with-shadow API | HTTP endpoint | Works |
| CLI commands | bifrost shadow list/results | Output correct |
| Sampling | Experiment sampling logic | Configurable |
| Tests | Unit + integration | ≥90% coverage |

---

## Dependencies

**Depends On**:
- WP-003 (Model Selection)
- WP-005 (Cost Tracking)

**Blocks**: None

---

## Effort Estimate

- **Estimated LOC**: 340
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T040: Create bifrost-routing-core/src/shadow.rs
- [ ] T041: Implement shadow request logic
- [ ] T042: Add SQLite schema for shadow metrics
- [ ] T043: Create HTTP endpoint POST /invoke-with-shadow
- [ ] T044: Implement experiment sampling
- [ ] T045: Create CLI command bifrost shadow list
- [ ] T046: Create CLI command bifrost shadow results
- [ ] T047: Unit tests
- [ ] T048: Integration test

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
