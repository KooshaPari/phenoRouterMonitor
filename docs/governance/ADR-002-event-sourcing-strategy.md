# ADR-002: Event Sourcing Strategy for Audit-Heavy Domains

- **Date**: 2026-03-30
- **Status**: Accepted
- **Deciders**: Phenotype Platform Team

---

## Context

Phenotype-infrakit operates across multiple audit-heavy domains where maintaining a complete, **immutable historical record** is essential for compliance, debugging, and regulatory requirements:

- **Policy Engine**: Tracks all policy evaluations, modifications, and enforcement decisions to demonstrate governance coverage and detect policy drift.
- **State Machine**: Records all state transitions to enable temporal analysis, rollback scenarios, and post-hoc auditing of workflow progressions.
- **Configuration Management**: Logs all configuration changes (who, when, what, why) to detect unauthorized modifications and support change reversals.
- **Access Control & Secrets Management**: Maintains audit trails of all access attempts, secret rotations, and permission changes for security compliance.

### Current Approach

The phenotype-infrakit codebase already implements **append-only event sourcing** with **SHA-256 hash chains** for immutability verification:

```
Event 1 (hash: A) → Event 2 (hash: B, prev_hash: A) → Event 3 (hash: C, prev_hash: B)
```

This design is present in the `phenotype-event-sourcing` crate with:

- **`EventEnvelope<T>`**: Generic envelope containing event ID, timestamp, actor, payload, sequence number, and hash chain links.
- **`EventStore` trait**: Abstract interface for append-only storage with `append()`, `get_events()`, `get_events_since()`, `get_events_by_range()`, and `verify_chain()` operations.
- **SHA-256 hash computation**: Each event is hashed with inputs including previous event hash, creating an tamper-evident chain.
- **Sequence numbering**: Monotonically increasing sequence IDs per entity ensure event ordering and detect gaps.

### The Problem

While this foundation is sound, **applying event sourcing across the entire platform introduces trade-offs**:

1. **Storage overhead**: Event logs grow unbounded (no truncation without CQRS read models).
2. **Query complexity**: Reconstructing domain state requires replaying event streams, which is expensive for high-frequency queries.
3. **Consistency guarantees**: Event stores guarantee eventual consistency, not strong consistency.
4. **Operational complexity**: Debugging and incident response require understanding event replay semantics.

The question is: **When should we use event sourcing, and when should we use traditional CRUD with audit tables?**

---

## Decision

Event sourcing is **adopted as the standard for audit-heavy domains** and **optional for others**. The following decision tree guides adoption:

### 1. Mandatory Event Sourcing Domains

Event sourcing MUST be used for:

| Domain | Rationale |
|--------|-----------|
| **Policy Evaluation & Enforcement** | Every policy application affects authorization; audit trail proves governance coverage. |
| **State Machine Transitions** | Business workflow correctness depends on valid state progressions; event replay enables temporal debugging. |
| **Secrets & Access Control** | Regulatory (SOC 2, ISO 27001) requires immutable audit logs of who accessed what and when. |
| **Change Management** | Configuration changes must be traced to actors and reversible; hash chains detect unauthorized modifications. |

### 2. Optional Event Sourcing Domains

Event sourcing MAY be adopted for domains where audit trails are desired but not mandated:

- **User Activity Logs** (read-heavy, high volume) — consider CQRS (event store + read model cache).
- **Analytics & Metrics** — traditional append-only tables with TTL are usually sufficient.
- **Transient Data** (caches, temporary state) — not candidates for event sourcing.

### 3. CQRS (Command Query Responsibility Segregation)

For high-volume event streams (>100K events/day), **offer optional CQRS on top of event sourcing**:

```
┌─────────────────────────────────────────────────────────┐
│                      CQRS Pattern                        │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  Write Side (Command)        Read Side (Query)          │
│  ├─ Event Store              ├─ Materialized Views      │
│  │  (source of truth)        │  (denormalized)          │
│  └─ Append Events            ├─ Cache Layer            │
│     (immutable)              │  (Redis, Memcached)      │
│                              └─ Projection Handlers    │
│                                                          │
│  Eventual Consistency Link:                            │
│  Event Stream → Projection Handler → Read Model        │
│                                                          │
└─────────────────────────────────────────────────────────┘
```

CQRS decouples writes (events) from reads (projections), allowing:

- **Write-optimized event store** (append-only, small footprint)
- **Read-optimized materialized views** (denormalized, indexed, cached)
- **Temporal queries** (event replay for historical state reconstruction)
- **Multiple read models** (domain-specific views from the same event stream)

Implementation notes:

- **Projection handler**: Listens to event stream, updates read model(s).
- **Eventual consistency**: Read model lags behind write model by milliseconds to seconds.
- **Snapshots**: For large event streams, store periodic snapshots to avoid full replay.

---

## Consequences

### Positive

- **Immutability**: Once written, events cannot be altered (hash chain detects tampering).
- **Complete audit trail**: Every state change is recorded with timestamp and actor for compliance.
- **Temporal queries**: Reconstruct state at any point in time ("What was the config on 2026-03-15?").
- **Replay capability**: Replay events to derive current state or debug issues without external data.
- **Compliance-ready**: SHA-256 hashes and actor tracking satisfy regulatory audit requirements.
- **Decoupling**: Event-driven architecture decouples services; policy engine doesn't need to know about state machine.
- **CQRS option**: For high-volume domains, CQRS adds read scalability without complexity.

### Negative

- **Storage cost**: Event logs grow unbounded; compaction/archival strategy required.
- **Query latency**: Reconstructing state from events is slower than direct state queries.
- **Operational overhead**: Debugging requires understanding event replay; incident response more complex.
- **Consistency trade-offs**: Eventual consistency (especially with CQRS) means stale reads for short windows.
- **Learning curve**: Team unfamiliar with event sourcing patterns needs training.
- **Snapshot management**: For large event streams, snapshots add complexity (validity, version management).

---

## Three Use-Case Examples

### Use Case 1: Policy Engine — Compliance Audit

**Scenario**: Regulatory auditor asks, "Prove that user Alice never had access to production secrets between 2026-01-15 and 2026-02-20."

**With Event Sourcing**:

```
Policy Application Events:
[2026-01-15T10:30Z] DenyAliceSecret { policy: "prod-access-control", result: DENY, actor: "system" }
[2026-01-15T10:35Z] DenyAliceSecret { policy: "prod-access-control", result: DENY, actor: "system" }
... (continues for entire period)
[2026-02-20T17:00Z] DenyAliceSecret { policy: "prod-access-control", result: DENY, actor: "system" }

Query: get_events_by_range("policy", "alice-secret-access", 2026-01-15, 2026-02-20)
Result: All DENY events, proving non-access.
Hash chain: Verifies no tampering occurred (each event hash chains to previous).
```

**Verdict**: Immutable, auditor-friendly, compliant.

---

### Use Case 2: State Machine — Incident Response

**Scenario**: A workflow got stuck in state `WaitingForApproval` unexpectedly. Operator needs to debug why.

**With Event Sourcing**:

```
State Transition Events:
[2026-03-30T14:00Z] Submitted { actor: "user:alice", workflow_id: "wf-123" }
[2026-03-30T14:05Z] ApprovalRequested { actor: "system", approver: "bob" }
[2026-03-30T14:07Z] ApprovalRejected { actor: "bob", reason: "missing-doc" }
[2026-03-30T14:08Z] Resubmitted { actor: "alice", workflow_id: "wf-123" }
[2026-03-30T14:09Z] ApprovalRequested { actor: "system", approver: "bob" }
[2026-03-30T15:30Z] (no more events after this)

Query: get_events("workflow", "wf-123")
Result: Full event history shows the exact sequence.
Operator can now see:
  - Bob approved at 14:09
  - The approval event may have failed to persist due to service crash
  - Or approval handler is stuck retrying
```

**Verdict**: Root cause easily identified via immutable record.

---

### Use Case 3: Configuration Management — Change Reversal

**Scenario**: A configuration change (bump Redis timeout from 100ms to 5000ms) introduced cascading failures. Operator needs to understand what changed and by whom.

**With Event Sourcing**:

```
Config Change Events:
[2026-03-30T12:00Z] ConfigUpdated {
  actor: "user:devops-alice",
  key: "redis.timeout",
  old_value: "100ms",
  new_value: "5000ms",
  reason: "reducing timeouts"
}
[2026-03-30T12:05Z] ServiceRestarted { actor: "system", reason: "config-hot-reload" }
[2026-03-30T12:06Z] AlertFired { alert: "p99-latency-spike", actor: "monitoring" }
[2026-03-30T12:10Z] ConfigReverted {
  actor: "user:devops-charlie",
  key: "redis.timeout",
  old_value: "5000ms",
  new_value: "100ms",
  reason: "revert cascading latency failures"
}

Query: get_events_by_range("config", "redis", 2026-03-30T11:55Z, 2026-03-30T12:15Z)
Result:
  - Identifies the actor who made the bad change (alice)
  - Identifies when the change occurred (12:00Z)
  - Identifies who reverted it (charlie) and when (12:10Z)
  - Hash chain verifies the revert transaction was not tampered
```

**Verdict**: Full change lineage, accountability, and immutability.

---

## When NOT to Use Event Sourcing

Event sourcing is **not appropriate** for:

1. **Transient state** (HTTP session tokens, in-flight caches) — expires naturally.
2. **High-volume analytics** (>1M events/day) — costs exceed benefits unless CQRS is in place.
3. **Read-heavy, write-light domains** (user profiles, product catalogs) — traditional DB with audit tables is simpler.
4. **Sensitive personally identifiable information (PII)** — GDPR "right to be forgotten" conflicts with immutability.

For these cases, **traditional CRUD with audit tables** is preferred:

```sql
-- Traditional audit table
CREATE TABLE config_audit (
  id UUID PRIMARY KEY,
  config_id UUID,
  old_value JSONB,
  new_value JSONB,
  actor VARCHAR,
  timestamp TIMESTAMPTZ DEFAULT NOW(),
  INDEX idx_config_actor (config_id, actor),
  INDEX idx_timestamp (timestamp)
);
```

This provides auditability without immutability or storage overhead.

---

## Implementation Guidelines

### 1. New Domains: Assume Event Sourcing

When designing a new audit-heavy domain, **default to event sourcing** unless explicitly ruled out.

```rust
// Policy Engine: Use Event Sourcing
pub struct PolicyEvaluationEvent {
    policy_id: String,
    result: PolicyResult,    // ALLOW, DENY
    context: HashMap<String, String>,
}

let event = EventEnvelope::new(
    PolicyEvaluationEvent { /* ... */ },
    "system"
);
event_store.append(&event, "policy", &policy_id)?;
```

### 2. Snapshot Optimization

For event streams with >10,000 events, implement periodic snapshots:

```rust
pub trait EventStore {
    fn append_snapshot(
        &self,
        snapshot_id: Uuid,
        entity_type: &str,
        entity_id: &str,
        state: &serde_json::Value,
    ) -> Result<()>;

    fn get_latest_snapshot(
        &self,
        entity_type: &str,
        entity_id: &str,
    ) -> Result<Option<(i64, serde_json::Value)>>;
}
```

When reconstructing state:
- Load latest snapshot (if available).
- Replay events since snapshot sequence onward.
- Reduces replay latency from O(n) to O(n - snapshot_sequence).

### 3. CQRS Implementation (Optional)

For high-volume streams, add projection handlers:

```rust
pub trait ProjectionHandler {
    fn handle_event(&self, event: &JsonEnvelope) -> Result<()>;
    fn build_read_model(&self, events: Vec<JsonEnvelope>) -> Result<ReadModel>;
}

// Example: Policy Application Projection
pub struct PolicyApplicationProjection {
    cache: Arc<Mutex<HashMap<String, PolicyState>>>,
}

impl ProjectionHandler for PolicyApplicationProjection {
    fn handle_event(&event) {
        match event.payload {
            PolicyEvaluationEvent { policy_id, result } => {
                self.cache.lock().unwrap()
                    .insert(policy_id, PolicyState { result, ... });
            }
            _ => {}
        }
    }
}
```

### 4. Hash Chain Verification

On service startup, verify event chains:

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let store = get_event_store()?;

    // Verify all entity chains
    store.verify_chain("policy", "global-policy")?;
    store.verify_chain("workflow", "wf-123")?;
    store.verify_chain("config", "redis-timeout")?;

    println!("All event chains verified ✓");
    Ok(())
}
```

---

## Migration Path

### Phase 1: Mandatory Domains (Weeks 1-2)

Implement event sourcing for:
1. Policy engine (high-security)
2. State machine (business-critical)
3. Secrets management (compliance-critical)

### Phase 2: Optional CQRS (Weeks 3-4)

For domains with >100K events/day:
1. Add projection handlers.
2. Set up read models (Redis or PostgreSQL materialized views).
3. Benchmark read latency improvements.

### Phase 3: Existing Services (Weeks 5-6)

Migrate existing audit-table patterns to event sourcing:
1. Generate events from audit table rows (one-time migration).
2. Gradually switch new writes to event store.
3. Archive old audit tables.

---

## Alternatives Considered

### 1. Traditional CRUD with Audit Tables (Rejected)

```sql
CREATE TABLE configs (id UUID, value JSONB, ...);
CREATE TABLE config_audit (config_id UUID, old_value JSONB, new_value JSONB, ...);
```

**Rejected because**:
- Audit table and main table can diverge (accidental deletes, data loss).
- No temporal queries (can't easily ask "what was state at time T").
- No built-in tamper detection (audit rows can be modified or deleted).

### 2. Blockchain (Over-engineered)

**Rejected because**:
- Overkill for internal audit logs; consensus overhead not needed.
- Poor query performance (merkle tree traversals).
- Higher operational complexity for minimal benefit over SHA-256 chains.

### 3. Event Sourcing Everywhere (Over-adoption)

**Rejected because**:
- Storage costs are prohibitive for transient data.
- Query complexity unnecessary for write-heavy, read-light domains.
- Eventual consistency issues with CQRS unsuitable for consistency-critical operations.

### 4. Hybrid: Event Sourcing for Compliance, CRUD for Everything Else (Accepted Pattern)

This ADR adopts a **pragmatic hybrid**:
- Mandatory for audit-heavy domains (compliance, security, governance).
- Optional for others.
- CQRS as a scalability lever (not required initially).

---

## References

### Event Sourcing Patterns

- **Event Sourcing Pattern**: https://martinfowler.com/eaaDev/EventSourcing.html
- **CQRS Pattern**: https://martinfowler.com/bliki/CQRS.html
- **Event Store Pattern**: https://eventstore.com/

### Phenotype Implementation

- `phenotype-event-sourcing` crate: `/crates/phenotype-event-sourcing/`
  - `EventEnvelope<T>`: Generic event wrapper with hash chain support
  - `EventStore` trait: Abstract interface for append-only storage
  - `compute_hash()`, `verify_chain()`: SHA-256 utilities
  - `InMemoryEventStore`: Reference implementation

### Security & Compliance

- **OWASP Audit Logging**: https://cheatsheetseries.owasp.org/cheatsheets/Logging_Cheat_Sheet.html
- **SOC 2 Audit Log Requirements**: https://www.aicpa.org/resources/downloading-aicpa-soc-2-report
- **GDPR & Right to Be Forgotten**: https://gdpr-info.eu/art-17-gdpr/ (conflicts with immutability)

### Related ADRs

- **ADR-001: External Package Adoption** — Adopted `koanf`, `go-chi` ecosystem, `orval` for cross-cutting concerns.

---

## Appendix: Decision Matrix

| Domain | Event Sourcing? | CQRS? | Notes |
|--------|-----------------|-------|-------|
| Policy Engine | ✅ MUST | Optional | Audit-heavy; immutability required for compliance |
| State Machine | ✅ MUST | Optional | Temporal queries support debugging; enables rollback design |
| Secrets Management | ✅ MUST | Optional | Security-critical; every access must be logged immutably |
| Config Management | ✅ MUST | Optional | Change lineage; reversibility; tamper detection |
| User Activity | ✅ Optional | ⭐ Recommended | High-volume; CQRS handles read scaling |
| Analytics | ❌ NO | N/A | TTL-based append-only tables are sufficient |
| Transient State | ❌ NO | N/A | Not audited; expires naturally |
| PII Data | ❌ NO | N/A | Immutability conflicts with GDPR "right to be forgotten" |

---

## Sign-Off

- **Accepted by**: Phenotype Platform Team
- **Implemented by**: `phenotype-event-sourcing` crate (v0.2.0+)
- **Status**: Active
- **Last reviewed**: 2026-03-30
