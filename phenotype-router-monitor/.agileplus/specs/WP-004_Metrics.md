# WP-004: Metrics Collection & Prometheus Export

**Work Package ID**: WP-004
**Epic**: eco-fork-002 (Consolidated API Monitoring & Routing)
**Phase**: 1
**Status**: Pending
**Priority**: High
**Created**: 2026-03-30

---

## Overview

Collect and export request metrics (latency, status codes, throughput) in Prometheus format.

## Description

Provide observability via Prometheus-compatible metrics endpoint. Track latency percentiles, error rates, and in-flight request counts.

---

## Objectives

- Collect request latency, response status codes, error rates per endpoint
- Aggregate metrics by service, path, status
- Export Prometheus-compatible metrics
- Expose HTTP endpoint `/metrics`

---

## Acceptance Criteria

1. **Metrics Collection**:
   - Latency histogram with p50, p95, p99 buckets
   - Status code counters (2xx, 4xx, 5xx)
   - In-flight requests gauge

2. **Export Format**:
   - `/metrics` returns Prometheus text format
   - Scrape format compliance verified

3. **Testing**:
   - `cargo test -p phenotype-router-metrics` all pass
   - Prometheus format validation test

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| Metric collectors | Latency, status, in-flight | Data accurate |
| Prometheus exporter | Text format export | Spec compliant |
| /metrics endpoint | HTTP endpoint | Scrapeable |
| Middleware integration | Metrics hooks in router | No perf overhead |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-001 (Router Core)

**Blocks**:
- WP-05 (CLI & Dashboard)

---

## Effort Estimate

- **Estimated LOC**: 340
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3 days

---

## Subtasks

- [ ] T025: Create `phenotype-router-metrics/src/lib.rs`
- [ ] T026: Implement histogram for request latency (p50, p95, p99)
- [ ] T027: Implement counters for requests by status code
- [ ] T028: Implement gauge for in-flight requests
- [ ] T029: Prometheus exporter format + `/metrics` endpoint
- [ ] T030: Integrate metrics middleware into router
- [ ] T031: Unit tests for metric collection accuracy
- [ ] T032: Verify Prometheus scrape format compliance

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
