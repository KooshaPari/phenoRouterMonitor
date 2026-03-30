# WP-004: SLA Enforcement & Latency Timeout

**Work Package ID**: WP-004
**Epic**: eco-fork-003 (LLM Routing Infrastructure)
**Phase**: 1
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Define and enforce SLA per workload with automatic failover on breach.

## Description

Monitor request latency against SLA. Failover to faster model if SLA breached. Track SLA breach rate.

---

## Objectives

- Define SLA per workload (code: <5s, analysis: <10s, writing: <15s)
- Monitor request latency against SLA
- Implement failover on SLA breach
- Track SLA breach rate per workload

---

## Acceptance Criteria

1. **SLA Enforcement**:
   - SLA enforced per request
   - On breach, failover to faster model
   - `/invoke-with-sla` endpoint works

2. **Tracking**:
   - SLA breach tracked in SQLite
   - Reports possible via SQL

3. **Testing**:
   - SLA evaluation tests pass
   - Failover trigger tests pass

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| SLAPolicy | Policy definition struct | Trait impl |
| Monitoring | Latency monitoring | Accurate |
| Failover | Automatic retry logic | Works |
| /invoke-with-sla API | HTTP endpoint | Enforces SLA |
| SQLite schema | SLA tracking | Queries work |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-003 (Model Selection)

**Blocks**:
- WP-005 (Cost Tracking)
- WP-006 (A/B Testing)

---

## Effort Estimate

- **Estimated LOC**: 320
- **Estimated Tool Calls**: 9-11
- **Estimated Duration**: 3 days

---

## Subtasks

- [ ] T023: Create bifrost-routing-core/src/sla.rs
- [ ] T024: Define SLA matrix in TOML
- [ ] T025: Implement SLA monitoring
- [ ] T026: Implement failover logic
- [ ] T027: Add SQLite schema
- [ ] T028: Create HTTP endpoint
- [ ] T029: Unit tests
- [ ] T030: Integration test

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
