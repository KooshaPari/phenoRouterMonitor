# WP-003: Rate Limiting & Circuit Breaker

**Work Package ID**: WP-003
**Epic**: eco-fork-002 (Consolidated API Monitoring & Routing)
**Phase**: 1
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Implement token-bucket rate limiting and circuit breaker with half-open recovery state.

## Description

Prevent cascading failures with circuit breaker pattern. Rate limit by endpoint and return 429 with Retry-After header when exceeded.

---

## Objectives

- Implement token-bucket rate limiter (per-endpoint)
- Add circuit breaker with states (closed, open, half-open)
- Support adaptive backoff for recovery
- Return 429 + Retry-After header when rate limit exceeded

---

## Acceptance Criteria

1. **Rate Limiting**:
   - Endpoint with limit=100 req/sec rejects 101st request (429)
   - Retry-After header set correctly on 429

2. **Circuit Breaker**:
   - Opens after 10 consecutive failures
   - Half-open state allows 1 probe every 30s
   - Closes when probe succeeds

3. **Testing**:
   - `cargo test -p phenotype-router-limiter` all pass
   - Exceed rate limit test passes
   - Circuit breaker state machine tested

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| RateLimiter | Token bucket implementation | Thread-safe |
| CircuitBreaker | State machine (closed/open/half-open) | States correct |
| Config | TOML rate limits per endpoint | Loads correctly |
| Headers | X-RateLimit-* and Retry-After | All present |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Router Core)

**Blocks**:
- WP-05 (CLI & Dashboard)

---

## Effort Estimate

- **Estimated LOC**: 360
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3-4 days

---

## Subtasks

- [ ] T017: Create `phenotype-router-limiter/src/lib.rs` with RateLimiter
- [ ] T018: Implement token-bucket algorithm with atomic operations
- [ ] T019: Create CircuitBreaker with states
- [ ] T020: Implement backoff logic (half-open retries every 30s)
- [ ] T021: Add rate limit headers to responses
- [ ] T022: TOML configuration for rate limits
- [ ] T023: Unit tests for token bucket + circuit breaker
- [ ] T024: Integration test: exceed rate limit, verify 429

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
