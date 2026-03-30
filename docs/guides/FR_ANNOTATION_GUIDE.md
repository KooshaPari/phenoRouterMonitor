# FR Annotation Guide

**Purpose**: Add Functional Requirement trace IDs to test files
**Updated**: 2026-03-30

---

## Why FR Annotations?

FR annotations enable:
- Finding which tests cover a given FR
- Identifying missing test coverage
- Automated traceability reports
- Compliance verification

---

## Standard Pattern

All tests should include FR ID comments:

```rust
#[tokio::test]
async fn test_feature_transition() {
    // Traces to: FR-API-002, FR-DOMAIN-003
    // Verify that feature state transitions emit audit entries

    // ... test implementation ...
}
```

---

## Annotation Format

### Single FR
```rust
// Traces to: FR-CATEGORY-NNN
```

### Multiple FRs
```rust
// Traces to: FR-CATEGORY-001, FR-CATEGORY-002
```

### With Description
```rust
// Traces to: FR-API-002, FR-DOMAIN-003
// Verify that feature state transitions emit audit entries
```

---

## By Test Type

### Unit Tests
```rust
#[test]
fn test_domain_event_serialization() {
    // Traces to: FR-DOMAIN-005
    // Ensure domain events serialize/deserialize correctly

    let event = DomainEvent::Created { id: 42 };
    let json = serde_json::to_string(&event).unwrap();
    let parsed: DomainEvent = serde_json::from_str(&json).unwrap();
    assert_eq!(event, parsed);
}
```

### Integration Tests
```rust
#[tokio::test]
async fn test_api_returns_audit_entries() {
    // Traces to: FR-API-002, FR-AUDIT-001
    // Verify API endpoint returns audit trail for feature lifecycle

    let response = client.get("/api/features/123/audit").send().await;
    assert_eq!(response.status(), 200);
    assert!(response.json::<Vec<AuditEntry>>().await.len() > 0);
}
```

### Async Tests
```rust
#[tokio::test]
async fn test_concurrent_event_handling() {
    // Traces to: FR-EVENTS-002, FR-P2P-003
    // Verify events are processed correctly under concurrent load

    let handlers: Vec<EventHandler> = (0..100).map(|_| MockHandler::new()).collect();
    // ... test implementation ...
}
```

---

## Category Prefixes

| Prefix | Category |
|--------|----------|
| FR-DOMAIN | Domain entities and state machines |
| FR-AUDIT | Audit trail and logging |
| FR-CLI | Command-line interface |
| FR-API | HTTP REST API |
| FR-GRPC | gRPC service layer |
| FR-STORAGE | SQLite and cache storage |
| FR-EVENTS | NATS event bus |
| FR-GRAPH | Neo4j graph storage |
| FR-IMPORT | Import manifest system |
| FR-TRIAGE | Triage classifier/router |
| FR-PLANE | Plane.so integration |
| FR-GIT | Git worktree and PR management |
| FR-GOVERN | Governance engine |
| FR-P2P | Peer-to-peer replication |
| FR-AGENT | Agent dispatch and review |
| FR-CONTENT | Content storage port |

---

## Examples by Category

### API Tests
```rust
#[tokio::test]
async fn test_create_feature_endpoint() {
    // Traces to: FR-API-001, FR-DOMAIN-005
    // POST /api/features creates new feature with default state

    let response = client.post("/api/features")
        .json(&CreateFeatureRequest { name: "Test" })
        .send()
        .await;
    assert_eq!(response.status(), 201);
}
```

### Domain Tests
```rust
#[test]
fn test_state_machine_valid_transition() {
    // Traces to: FR-DOMAIN-003, FR-DOMAIN-008
    // Verify valid state transitions are accepted

    let mut feature = Feature::new("test");
    assert_eq!(feature.state(), State::Draft);
    feature.transition(State::Active).unwrap();
    assert_eq!(feature.state(), State::Active);
}
```

### Storage Tests
```rust
#[tokio::test]
async fn test_sqlite_event_persistence() {
    // Traces to: FR-STORAGE-001, FR-AUDIT-005
    // Events are persisted and retrievable from SQLite

    let store = SqliteStore::new(":memory:").await.unwrap();
    store.append(event.clone()).await.unwrap();
    let retrieved = store.get_events(feature_id).await.unwrap();
    assert_eq!(retrieved.len(), 1);
}
```

---

## Automated Discovery

Find all tests for a specific FR:

```bash
# Find all tests for FR-API-001
grep -r "Traces to:.*FR-API-001" crates/

# Find all tests without annotations (potential gaps)
grep -rL "Traces to:" crates/*/tests/
```

---

## Commit Pattern

When adding FR annotations:

```bash
git commit -m "test(traceability): add FR annotations to feature module

Traces to: FR-DOMAIN-005, FR-API-001
- Add annotations to domain model tests
- Add annotations to API integration tests
"
```

---

## Enforcement

### Pre-commit Hook (Recommended)
```bash
# .git/hooks/pre-commit
#!/bin/bash
# Check for missing FR annotations in new test files
git diff --cached --name-only | grep '_test\.rs$' | while read file; do
    if ! grep -q "Traces to:" "$file"; then
        echo "Error: $file missing FR annotation"
        exit 1
    fi
done
```

### CI Check
```yaml
# .github/workflows/traceability.yml
- name: Check FR annotations
  run: |
    MISSING=$(grep -rL "Traces to:" crates/*/tests/ || true)
    if [ -n "$MISSING" ]; then
      echo "Missing FR annotations in: $MISSING"
      exit 1
    fi
```

---

## FR Reference

See also:
- `docs/reference/FR_TRACKER.md` - Full FR implementation status
- `docs/reference/CODE_ENTITY_MAP.md` - Code ↔ FR mappings
- `FUNCTIONAL_REQUIREMENTS.md` - Authoritative requirements

---

**Last Updated**: 2026-03-30
