# WP-006: Provider Performance Metrics & Feedback Loop

**Work Package ID**: WP-006
**Epic**: eco-fork-001 (Custom Providers & Subagent Management)
**Phase**: 2
**Status**: Pending
**Priority**: Medium
**Created**: 2026-03-30

---

## Overview

Collect provider performance metrics (latency, token count, error rate) and implement automatic feedback loop to optimize routing based on historical performance.

## Description

Track real-world provider performance across all invocations. Aggregate metrics by model and provider. Auto-tune routing weights based on performance degradation or improvements.

---

## Objectives

- Collect metrics: latency, token count, error rate per call
- Aggregate metrics by provider/model
- Implement feedback loop: read historical metrics → adjust routing weights
- Expose metrics via CLI and HTTP API

---

## Acceptance Criteria

1. **Metrics Collection**:
   - Latency recorded for every call (milliseconds)
   - Token count (input/output) captured from API response
   - Error rate calculated per provider
   - Metrics persisted to SQLite

2. **Aggregation**:
   - `agileplus provider metrics <provider>` shows stats
   - Percentiles: p50, p95, p99 for latency
   - Token count averages
   - Error rates per model

3. **Feedback Loop**:
   - Routing weights auto-adjust if provider latency degrades
   - E.g., if Opus latency exceeds threshold, reduce weight
   - Changes logged to audit table

4. **HTTP API**:
   - `/metrics/providers` endpoint returning Prometheus format
   - JSON alternative: `/metrics/providers?format=json`

5. **Testing**:
   - Metrics accuracy tests
   - Feedback loop trigger tests
   - Percentile calculation tests

---

## Deliverables

| Deliverable | Description | Acceptance |
|-------------|-------------|-----------|
| Metrics tables | provider_metrics, performance_log | Schema correct |
| Collection logic | Hook in Provider trait | All calls tracked |
| Aggregation | ProviderMetrics struct | Stats calculated |
| CLI commands | provider metrics <provider> | Output correct |
| HTTP API | /metrics/providers endpoint | Prometheus format |
| Feedback loop | Auto-tune routing weights | Logic tested |
| Tests | Unit + integration | ≥85% coverage |

---

## Dependencies

**Depends On**:
- WP-004 (Subagent Spawning)
- WP-005 (Capability Discovery)

**Blocks**: None (end of WP chain)

---

## Effort Estimate

- **Estimated LOC**: 340
- **Estimated Tool Calls**: 10-12
- **Estimated Duration**: 3-4 days

---

## Technical Details

### Metrics Structs

```rust
pub struct MetricsCollector {
    latencies: Arc<DashMap<String, Vec<u64>>>,
    errors: Arc<DashMap<String, u64>>,
    token_counts: Arc<DashMap<String, TokenStats>>,
}

pub struct ProviderMetrics {
    pub provider: String,
    pub model: String,
    pub latency_p50: u64,
    pub latency_p95: u64,
    pub latency_p99: u64,
    pub avg_tokens_in: f64,
    pub avg_tokens_out: f64,
    pub error_rate: f64,
}
```

### SQLite Schema

```sql
CREATE TABLE provider_metrics (
    provider_id TEXT,
    model TEXT,
    metric_date DATE,
    latency_p50_ms INTEGER,
    latency_p95_ms INTEGER,
    latency_p99_ms INTEGER,
    avg_input_tokens REAL,
    avg_output_tokens REAL,
    error_rate REAL,
    PRIMARY KEY (provider_id, model, metric_date)
);

CREATE TABLE provider_performance_log (
    log_id INTEGER PRIMARY KEY,
    provider_id TEXT,
    model TEXT,
    timestamp TIMESTAMP,
    latency_ms INTEGER,
    input_tokens INTEGER,
    output_tokens INTEGER,
    success BOOLEAN,
    error_type TEXT
);
```

### CLI Usage

```bash
# Show metrics for specific provider
agileplus provider metrics claude
# Output:
# Provider: claude
# ├─ opus
# │  ├─ Latency: p50=450ms, p95=800ms, p99=1500ms
# │  ├─ Tokens: in=250 avg, out=150 avg
# │  └─ Error rate: 0.5%
# ├─ sonnet
# │  ├─ Latency: p50=250ms, p95=400ms, p99=900ms
# │  ├─ Tokens: in=200 avg, out=120 avg
# │  └─ Error rate: 0.2%
# └─ haiku
#    ├─ Latency: p50=80ms, p95=200ms, p99=500ms
#    ├─ Tokens: in=150 avg, out=80 avg
#    └─ Error rate: 0.1%

# Show feedback loop status
agileplus provider feedback-status
# Output: Shows current routing weights + recent adjustments
```

---

## Feedback Loop Logic

```
For each provider:
  1. Calculate latency p95 from last 1000 calls
  2. Compare to baseline p95
  3. If degradation > 20%:
     - Reduce routing weight by 10%
     - Log adjustment to audit trail
  4. If improvement > 10%:
     - Increase routing weight by 5%
     - Log adjustment to audit trail
  5. Update at: daily or after N calls (configurable)
```

---

## Subtasks

- [ ] T031: Create `provider_metrics` and `provider_performance_log` tables
- [ ] T032: Implement metrics collection hook in Provider trait
- [ ] T033: Create `ProviderMetrics` aggregation struct with percentile calculation
- [ ] T034: Add `agileplus provider metrics <provider>` CLI command
- [ ] T035: Implement feedback loop: read metrics → adjust routing weights
- [ ] T036: HTTP API endpoint `/metrics/providers` returning Prometheus format
- [ ] T037: Unit tests for metric aggregation and feedback logic

---

## Metrics Export Format (Prometheus)

```
# HELP provider_latency_ms Provider request latency
# TYPE provider_latency_ms histogram
provider_latency_ms_bucket{provider="claude",model="opus",le="100"} 10
provider_latency_ms_bucket{provider="claude",model="opus",le="500"} 450
provider_latency_ms_bucket{provider="claude",model="opus",le="1000"} 980
provider_latency_ms_bucket{provider="claude",model="opus",le="+Inf"} 1000

# HELP provider_error_total Provider error count
# TYPE provider_error_total counter
provider_error_total{provider="claude",model="opus"} 5

# HELP provider_tokens_total Provider token count
# TYPE provider_tokens_total counter
provider_tokens_total{provider="claude",model="opus",direction="input"} 250000
provider_tokens_total{provider="claude",model="opus",direction="output"} 150000
```

---

## Success Metrics

| Metric | Target | Measure |
|--------|--------|---------|
| Metrics Accuracy | ±2% | Validate against logs |
| Feedback Loop | Responsive | Adjusts within 1 day |
| Export Format | Prometheus-compliant | Scrape test passes |
| Latency Overhead | <5% | Metric collection overhead |
| Test Coverage | ≥85% | `cargo tarpaulin` |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| Metrics storage overhead | Medium | Low | Partition tables by date, archive old data |
| Feedback loop oscillation | Low | Low | Use smoothing, minimum adjustment threshold |
| Metric staleness | Low | Low | Refresh daily or after N calls |

---

## Traceability

**Functional Requirements**:
- FR-PROV10: Provider performance metrics collection
- FR-PROV11: Feedback-driven routing optimization

---

## Notes

- Initial baseline: first 100 calls establish baseline metrics
- Feedback loop minimum adjustment: ±5%
- Metrics retention: 90 days, archive older data
- Future: Real-time metrics streaming via WebSocket

---

**Owner**: TBD
**Last Updated**: 2026-03-30
**Status**: Pending Implementation
