# WP-002: Health Checking & Aggregation

**Work Package ID**: WP-002
**Epic**: eco-fork-002 (Consolidated API Monitoring & Routing)
**Phase**: 1
**Status**: Pending
**Priority**: Critical
**Created**: 2026-03-30

---

## Overview

Implement active health checking via HTTP probes and passive observation, with aggregation across all backends.

## Description

Monitor backend health through periodic HTTP probes and observe failures in real traffic. Aggregate status across services and exclude unhealthy backends from routing.

---

## Objectives

- Implement `HealthChecker` trait for active and passive monitoring
- Aggregate status across all backends
- Support custom health endpoints (`/health`, `/status`, custom paths)
- Implement exponential backoff for retry logic
- Expose aggregated status via HTTP endpoint

---

## Acceptance Criteria

1. **Health Checking**:
   - `GET /health` returns JSON with ≥5 backend states
   - Aggregator marks backend unhealthy after 5 consecutive failures
   - Status accurate within 5 seconds of state change

2. **Monitoring**:
   - Active HTTP probes to configured endpoints
   - Passive observation of request failures
   - Unhealthy backends excluded from routing until recovery

3. **Backoff**:
   - Exponential backoff on failures
   - Configurable retry intervals

4. **Testing**:
   - `cargo test -p phenotype-router-health` all pass
   - Backend down scenario tested

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| HealthChecker trait | Active/passive health interface | Trait defined |
| HealthAggregator | Tracks all backend states | Aggregation works |
| HTTP probe | GET health endpoint | Configurable paths |
| Backoff strategy | Exponential retry | Logic correct |
| HTTP API | /health endpoint | JSON output |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Router Core)

**Blocks**:
- WP-003 (Rate Limiting)
- WP-04 (Metrics)

---

## Effort Estimate

- **Estimated LOC**: 380
- **Estimated Tool Calls**: 11-13
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T009: Create `phenotype-router-health/src/lib.rs` with HealthChecker trait
- [ ] T010: Implement HTTP health probe (GET + status code thresholds)
- [ ] T011: Create HealthAggregator for all backend states
- [ ] T012: Add passive observation (mark unhealthy on failures)
- [ ] T013: Implement backoff strategy (exponential retry)
- [ ] T014: HTTP endpoint `GET /health` returning JSON
- [ ] T015: Unit tests for health state transitions
- [ ] T016: Integration test: mark backend down, verify aggregator

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
