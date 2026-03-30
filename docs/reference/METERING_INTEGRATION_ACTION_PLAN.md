# API Metering Integration Action Plan
## phenotype-router-monitor Enhancement

**Date**: 2026-03-30  
**Status**: Recommendation Phase  
**Priority**: High (Cost tracking, quota management)

---

## Executive Summary

Based on comprehensive research of 10 industry API metering and usage tracking tools, we recommend a **hybrid approach** combining three core components:

1. **Tyk** (API Gateway + Rate Limiting) — Open-source, production-ready
2. **phenotype-router-monitor** (Custom Metering) — Domain-specific, Rust-based
3. **Grafana** (Observability + Dashboards) — Flexible visualization & alerting

This combination provides real-time metering, quota enforcement, and observability with full control and no vendor lock-in.

---

## Problem Statement

**Current State**:
- No real-time API usage tracking
- No per-user, per-endpoint cost tracking
- No quota enforcement mechanisms
- Manual cost tracking via service provider dashboards
- Cannot track per-model usage (OpenAI vs Anthropic, GPT-4 vs Claude)
- No integration with billing systems

**Desired State**:
- Real-time API metering (sub-second latency)
- Per-user, per-endpoint, per-model cost tracking
- Hard quota enforcement (block requests)
- Soft quota alerts (warning notifications)
- Automated cost attribution
- Multi-tenant cost tracking
- Integration with Stripe Metering (optional)

---

## Recommended Architecture

### Layer 1: API Gateway (Tyk)
```
Client → [Tyk API Gateway] → Upstream Service
         ├─ Rate limiting (token bucket)
         ├─ Quota enforcement per API key
         ├─ Analytics (hits, latency, errors)
         └─ Webhook notifications (quota_exceeded)
```

**Responsibilities**:
- Coarse-grained metering (API calls, latency, errors)
- Rate limiting enforcement
- API key management
- Baseline quota tracking

**Metrics Exported**:
- API calls (per endpoint, per key)
- Latency (min, max, avg)
- Error rates (by HTTP status)
- Bandwidth (request/response size)

### Layer 2: Fine-Grained Metering (Custom Phenotype)
```
Request → [phenotype-router-monitor] → Provider (OpenAI, Anthropic)
          ├─ Token counting
          ├─ Cost calculation
          ├─ Per-model tracking
          ├─ Provider cost attribution
          └─ Event streaming → Kafka/Redis
```

**Responsibilities**:
- LLM-specific metrics (tokens, models)
- Per-provider cost tracking
- Hard quota enforcement (custom rules)
- Real-time cost calculation

**Metrics Exported**:
- Tokens used (input, output, total)
- Cost (real-time calculation)
- Model used (GPT-4, Claude 3, etc.)
- Provider (OpenAI, Anthropic, etc.)

### Layer 3: Storage
```
[In-Memory Counters] → [Batch Writer] → Supabase PostgreSQL
[Redis Cache]        → [Hot Storage]  → Redis
[Event Stream]       → [Analytics]    → Kafka
```

**Responsibilities**:
- Time-series storage (metering events)
- Real-time quota counter cache
- Event streaming for analytics
- Historical data retention

### Layer 4: Observability (Grafana + Prometheus)
```
[Metrics Exporter] → Prometheus → Grafana Dashboards
                                 ├─ Usage dashboards
                                 ├─ Cost breakdown
                                 ├─ Quota status
                                 ├─ Alert rules
                                 └─ Anomaly detection
```

**Responsibilities**:
- Metrics visualization
- Real-time dashboards
- Alerting & notifications
- Historical trend analysis

### Layer 5: Billing (Optional)
```
[phenotype-router-monitor] → Webhook → Stripe Metering
                                      ├─ Customer usage tracking
                                      ├─ Billing calculation
                                      └─ Invoice generation
```

---

## Implementation Phases

### Phase 1: Foundation (2-3 weeks)
**Goal**: Deploy Tyk gateway with basic rate limiting

**Tasks**:
- [ ] Deploy Tyk v4.x in Docker Compose (sandbox)
- [ ] Configure Redis backend for quota state
- [ ] Set up basic API key policies
- [ ] Configure rate limiting (token bucket)
- [ ] Expose metrics to Prometheus
- [ ] Create basic Grafana dashboard

**Deliverables**:
- Tyk running in sandbox with 2-3 test APIs
- Redis cluster for quota tracking
- Prometheus scraper ingesting Tyk metrics
- Grafana dashboard showing API calls & latency

**Success Criteria**:
- ✅ Tyk accepts valid API keys
- ✅ Tyk rejects invalid keys (HTTP 401)
- ✅ Tyk enforces rate limits (HTTP 429)
- ✅ Prometheus metrics updated every 15 sec
- ✅ Grafana shows API call trends

### Phase 2: Custom Metering (3-4 weeks)
**Goal**: Extend phenotype-router-monitor with metering capabilities

**Tasks**:
- [ ] Define `MeteringPort` trait (hexagonal architecture)
- [ ] Implement in-memory counters
- [ ] Create Supabase schema (usage_metrics, quota_events)
- [ ] Implement token counting logic (per model)
- [ ] Add Redis cache for quota counters
- [ ] Implement batched write to PostgreSQL

**Deliverables**:
- `phenotype-metering` crate with ports/adapters
- In-memory implementation with Redis cache
- PostgreSQL schema with indexes
- Tests (unit + integration)

**Success Criteria**:
- ✅ Token counting matches LLM provider values (±1%)
- ✅ Redis cache hit rate > 95%
- ✅ Batched writes (30 sec intervals) working
- ✅ Unit tests: >80% coverage
- ✅ Integration tests: end-to-end tracking

### Phase 3: Observability (2-3 weeks)
**Goal**: Create comprehensive dashboards & alerting

**Tasks**:
- [ ] Implement Prometheus exporter in phenotype-router-monitor
- [ ] Create 5+ Grafana dashboards
- [ ] Set up alert rules (quota exceeded, cost spike, error rate)
- [ ] Configure notification channels (email, Slack, webhooks)
- [ ] Test end-to-end alerting

**Deliverables**:
- Prometheus `/metrics` endpoint
- Grafana dashboards:
  1. Usage by user/endpoint/model
  2. Cost breakdown and trends
  3. Quota status (per-user)
  4. Error rates and SLA tracking
  5. Token consumption by model
- Alert rules and notification templates

**Success Criteria**:
- ✅ Dashboards update every 15 sec
- ✅ Alerts fire within 1 min of threshold breach
- ✅ Notifications delivered successfully
- ✅ Dashboard drill-down working (click → details)

### Phase 4: Billing Integration (2-3 weeks, Optional)
**Goal**: Integrate with Stripe Metering and AgilePlus dashboard

**Tasks**:
- [ ] Obtain Stripe API credentials
- [ ] Implement webhook to Stripe Metering
- [ ] Build AgilePlus dashboard widgets
- [ ] Expose billing API (`/api/v1/billing/usage`)
- [ ] Implement cost attribution (per-user, per-org)
- [ ] Test invoice generation (E2E)

**Deliverables**:
- Stripe Metering integration (webhook)
- AgilePlus dashboard widgets (cost breakdown, usage trends)
- Billing API with cost attribution
- Integration tests with mock Stripe

**Success Criteria**:
- ✅ Stripe receives usage events within 30 sec
- ✅ AgilePlus widgets display accurate costs
- ✅ Billing API returns cost per user/org
- ✅ Invoice generation working

---

## Metrics to Track

### Real-Time (Sub-second, Updated Every 15 sec)
```
Per-User:
├─ API calls (count)
├─ Tokens used (input, output, total)
├─ Cost (real-time calculation)
├─ Quota remaining
└─ Error count

Per-Endpoint:
├─ API calls (count)
├─ Latency (p50, p95, p99)
├─ Error rate (%)
├─ Bandwidth
└─ Cost

Per-Model:
├─ Tokens used (input, output)
├─ Cost
├─ Error rate
└─ Latency

Per-Provider:
├─ API calls
├─ Tokens used
├─ Cost
└─ Error rate

Per-Org:
├─ Total API calls
├─ Total cost
├─ Quota remaining
└─ Error count
```

### Aggregated (Updated Hourly)
```
├─ Cost per user (daily, weekly, monthly)
├─ Usage trends (day/week/month-over-month)
├─ Top users by cost
├─ Top endpoints by latency
├─ Error rate trends
└─ Anomalies (cost spike, traffic spike)
```

---

## Quota Management Strategy

### Hard Quotas (Block Requests)
```
Per-User:
├─ Per-minute: 1000 tokens (immediate block)
├─ Per-hour: 1M tokens (enforced via rolling window)
├─ Per-day: 100M tokens (enforced via time-series counter)
└─ Per-month: 1B tokens (enforced via monthly reset)

Per-Endpoint:
├─ Per-minute: 100 requests
├─ Per-hour: 10K requests
└─ Per-day: 1M requests

Per-Provider:
├─ API rate limits (respect upstream limits)
├─ Account-level quotas
└─ Concurrent request limits
```

### Soft Quotas (Alert Only)
```
├─ 50% of quota → Email warning
├─ 75% of quota → Slack notification
├─ 90% of quota → Both email + Slack + webhook
└─ Quota exceeded → Suspend API key + admin notification
```

### Auto-Suspension & Recovery
```
Suspended API Key:
├─ All requests return HTTP 429 (Too Many Requests)
├─ Notification sent to user + admin
├─ Auto-recovery: On next billing cycle reset
└─ Manual recovery: Admin override via Tyk dashboard
```

---

## Cost Analysis

### Development Effort
```
Phase 1 (Foundation):       40-50h  ($2K-3K)
Phase 2 (Custom Metering):  60-80h  ($3K-5K)
Phase 3 (Observability):    30-40h  ($1.5K-2K)
Phase 4 (Billing):          30-40h  ($1.5K-2K)
─────────────────────────────────────────
Total:                      160-210h ($8K-12K)

Wall-Clock Time: 4-6 weeks (with parallel work on phases 2-3)
Team Size: 2-3 engineers
```

### Infrastructure Costs
```
Monthly:
├─ Supabase PostgreSQL:    $50/month (pay-as-you-go)
├─ Redis:                  $20/month (2GB instance)
├─ Prometheus:             $0/month (self-hosted)
├─ Grafana:                $0/month (self-hosted)
└─ Tyk:                    $0/month (community edition)
─────────────────────────────────────────
Total Monthly:             $70/month
```

### Maintenance Effort
```
Weekly:
├─ Monitoring (dashboards): 2-3h
├─ Alert response:          1-2h
├─ Schema updates:          1h
└─ Performance tuning:      2-3h
─────────────────────────────────────────
Total Weekly:              6-9h/week
```

---

## Tools Evaluated & Rejected

### Rejected (Proprietary/SaaS Only)
- **Stripe Metering**: SaaS billing only (no self-hosted), no quota enforcement
  - Use Case: Optional integration for SaaS customers only
  
- **CloudZero**: Cloud cost intelligence (AWS/GCP/Azure), not API metering
  - Use Case: Not applicable for API metering
  
- **Datadog**: APM + cost tracking (SaaS only, expensive at scale)
  - Use Case: Optional APM replacement, but no quota enforcement

### Rejected (Wrong Domain)
- **OpenCost**: Kubernetes cost tracking (batch-based, not real-time)
  - Use Case: K8s cost chargeback, not API metering
  
- **Kubecost**: Kubernetes cost tracking (K8s-specific)
  - Use Case: K8s cost visibility, complementary to API metering
  
- **FinOps Framework**: Meta-framework (best practices, not a tool)
  - Use Case: Cost governance discipline, not metering implementation

---

## Alternative Option: 3scale (Enterprise)

### Use Case
If Phenotype needs:
- Tiered pricing models
- Complex plan management
- Multi-tenant SaaS isolation
- OAuth 2.0 integration

### Comparison with Tyk + Custom
```
Feature                 | Tyk + Custom | 3scale
─────────────────────────────────────────────
Real-time metering      | ✅          | ⚠️ Near-real-time
Per-user tracking       | ✅          | ✅
Per-endpoint tracking   | ✅          | ✅
Per-provider tracking   | ✅          | ✅
Rate limiting           | ✅          | ✅
Quota enforcement       | ✅          | ✅
Pricing flexibility     | Moderate    | High
Maintenance burden      | Low         | High
Setup complexity        | Low         | High
Development effort      | 160-210h    | 250-350h
Recommendation          | ✅ PRIMARY  | ❌ Only if needed
```

**NOT RECOMMENDED** unless enterprise features are specifically required.

---

## Security Considerations

### API Key Management
- Tyk: Built-in encrypted key vault (Redis-backed)
- Custom: Supabase auth with encryption at rest
- Rotation: Monthly key rotation recommended

### Rate Limiting DDoS Protection
- Token bucket algorithm prevents burst attacks
- Per-user limits prevent coordinated attacks
- Anomaly detection alerts on suspicious patterns

### Data Privacy
- All metrics stored internally (no cloud analytics)
- PII-safe: Only track API keys, users, endpoints
- Compliance: Ready for SOC2/GDPR audits
- Audit trail: PostgreSQL audit tables for compliance

### Encryption
- In-transit: TLS 1.3 for all API calls
- At-rest: PostgreSQL encryption (Supabase default)
- Redis: Optional encryption (if sensitive metrics)

---

## Next Steps

### Week 1 (Immediate)
- [ ] Review and approve recommendation
- [ ] Create AgilePlus spec (pheno-metering-system-v1)
- [ ] Set up Tyk sandbox in Docker Compose
- [ ] Brief team on architecture
- [ ] Reserve infrastructure resources

### Weeks 2-4 (Short-term)
- [ ] Deploy Tyk gateway (Phase 1)
- [ ] Implement phenotype-router-monitor metering (Phase 2)
- [ ] Create Grafana dashboards (Phase 3)
- [ ] Test end-to-end metering

### Weeks 5-6 (Medium-term)
- [ ] Set up alerting and notifications
- [ ] Document APIs
- [ ] Internal dogfooding (use for internal cost tracking)

### Weeks 7-8+ (Long-term, Optional)
- [ ] Integrate Stripe Metering (if SaaS billing needed)
- [ ] Build AgilePlus dashboard widgets
- [ ] Implement chargeback/cost attribution
- [ ] Add forecasting and anomaly detection

---

## Success Metrics

### Metering Accuracy
- ✅ Token counting matches provider values ±1%
- ✅ Cost calculation matches manual verification ±2%
- ✅ Per-user cost tracking matches invoices

### System Performance
- ✅ Metering latency < 100ms (p99)
- ✅ Quota enforcement latency < 50ms
- ✅ Dashboard updates < 15 sec
- ✅ Alert notification < 1 min

### Operational Excellence
- ✅ System uptime > 99.9%
- ✅ Zero data loss
- ✅ Audit trail 100% complete
- ✅ MTTR for alerts < 15 min

### Business Value
- ✅ Accurate cost attribution
- ✅ Quota enforcement prevents overspend
- ✅ Visibility enables cost optimization
- ✅ Integration with billing system

---

## Appendices

### A. Detailed Tool Analysis
See: `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/research/API_METERING_TOOLS_ANALYSIS.md`

### B. Executive Summary
See: `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/research/METERING_EXEC_SUMMARY.txt`

### C. Feature Matrix
```
                          Tyk  3scale  Custom  Grafana  OpenCost  Kubecost
Open Source               ✅   ✅      ✅      ✅       ✅        ✅
Self-Hosted              ✅   ✅      ✅      ✅       ✅        ✅
Real-Time Metering       ✅   ⚠️      ✅      ✅       ❌        ✅
Per-User Granularity     ✅   ✅      ✅      ✅       ⚠️        ✅
Per-Endpoint Tracking    ✅   ✅      ✅      ✅       ❌        ⚠️
Per-Provider Tracking    ✅   ✅      ✅      ✅       ❌        ❌
Rate Limiting/Quota      ✅   ✅      ✅      ❌       ❌        ❌
Cost Calculation         ✅   ✅      ✅      ⚠️       ✅        ✅
Alerting/Notifications   ✅   ✅      ✅      ✅       ⚠️        ✅
Custom Dashboards        ✅   ✅      ⚠️      ✅       ⚠️        ✅
API-First Design         ✅   ✅      ✅      ✅       ✅        ✅
Easy Integration         ✅   ⚠️      ⭐     ✅       ⚠️        ⚠️
Production Ready         ✅   ✅      ⚠️      ✅       ⚠️        ✅

Legend: ✅ = Full support, ⚠️ = Partial, ❌ = No support, ⭐ = Excellent
```

---

**Document Status**: Recommendation Phase  
**Approval Required From**: Engineering Leadership  
**Expected Start Date**: 2026-04-01  
**Expected Completion Date**: 2026-05-30  
