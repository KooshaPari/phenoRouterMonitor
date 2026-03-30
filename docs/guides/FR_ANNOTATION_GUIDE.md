<<<<<<< HEAD
# FR Annotation Guide for Test Files

Quick guide for adding Functional Requirement (FR) ID annotations to AgilePlus test files.

---

## Overview

All test functions should include `// Traces to: FR-XXX-YYY` comments to establish bidirectional traceability between requirements and their test coverage.

This enables:
- Quick lookup of which tests validate a given FR
- Verification that every FR has at least one test
- Gap identification (missing or untested FRs)
- Automated traceability reports via CI

---

## Pattern

### Basic Annotation

```rust
#[tokio::test]
async fn test_feature_creation() {
    // Traces to: FR-API-001
    // Describe what this test verifies relative to the FR
    let server = setup_test_server().await;
    // ... test body ...
}
```

### Multiple FRs

```rust
#[tokio::test]
async fn test_feature_transition_with_audit() {
    // Traces to: FR-API-002, FR-AUDIT-001, FR-DOMAIN-003
    // Verify that feature state transitions emit audit entries
    // and enforce forward-only state machine rules
    let server = setup_test_server().await;
    // ... test body ...
}
```

### Inline Module Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_clock_merge() {
        // Traces to: FR-P2P-001
        // Verify vector clock merge computes component-wise maximum
        let clock1 = VectorClock::new(vec![1, 2, 3]);
        // ... test body ...
    }
}
=======
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
>>>>>>> origin/main
```

---

<<<<<<< HEAD
## Finding the Right FR IDs

### Option 1: Use FR_TRACKER.md

1. Open `docs/reference/FR_TRACKER.md`
2. Find the category matching your test (e.g., FR-API for API tests)
3. Locate the specific FR matching your test's purpose
4. Note the FR ID

Example:
- Test: `test_list_features_requires_auth()`
- Category: FR-API (HTTP REST API)
- Find: "All REST API routes SHALL reject unauthenticated requests"
- FR ID: **FR-API-007**

### Option 2: Use CODE_ENTITY_MAP.md

1. Open `docs/reference/CODE_ENTITY_MAP.md`
2. Find the code entity your test is testing (e.g., route handler, struct)
3. Note the FR ID(s) in the "Maps To FR" column

Example:
- Entity: `ApiKeyMiddleware` (in middleware/auth.rs)
- Maps To: **FR-API-007**

### Option 3: Search Existing Tests

```bash
grep -r "Traces to: FR-" crates/ | grep -i "auth\|key"
```

This shows tests already referencing authentication FRs.

---

## Test File Organization

### API Integration Tests
**Location:** `crates/agileplus-api/tests/api_integration/`

Map to:
- FR-API-* (HTTP REST API routes)
- FR-DOMAIN-* (domain entity operations)
- FR-AUDIT-* (audit trail verification)
- FR-GOVERN-* (governance enforcement)

Example:
```rust
// File: core_routes.rs
#[tokio::test]
async fn list_features_requires_auth() {
    // Traces to: FR-API-007
    // Verify unauthenticated requests rejected with 401
```

### CLI Tests
**Location:** `crates/agileplus-cli/tests/`

Map to:
- FR-CLI-* (command-line interface)

Example:
```rust
// File: feature_cli.rs
#[test]
fn feature_create_command() {
    // Traces to: FR-CLI-001
    // Verify `agileplus feature create` creates feature in Created state
```

### Domain Tests
**Location:** `crates/agileplus-domain/tests/` and `src/*/tests.rs`

Map to:
- FR-DOMAIN-* (entities, state machines)
- FR-AGENT-* (agent ports and traits)
- FR-CONTENT-* (content storage ports)

Example:
```rust
// File: src/domain/state_machine.rs
#[cfg(test)]
mod tests {
    #[test]
    fn state_transition_forward_only() {
        // Traces to: FR-DOMAIN-003
        // Verify state transitions enforce forward-only rule
```

### Integration Tests
**Location:** `crates/agileplus-integration-tests/tests/` and `crates/agileplus-*/tests/`

Map to:
- Multiple FRs across categories (e.g., FR-DOMAIN-*, FR-EVENTS-*, FR-P2P-*)

Example:
```rust
// File: crates/agileplus-nats/src/bus/tests.rs
#[tokio::test]
async fn event_publish_to_nats() {
    // Traces to: FR-EVENTS-001, FR-EVENTS-002
    // Verify domain events published to NATS with correct subject pattern
```

---

## Documentation Comment

Include a brief description of what the test verifies:

### Good
```rust
// Traces to: FR-API-001
// Verify that POST /features creates a feature with all required fields
#[tokio::test]
async fn create_feature_returns_all_fields() {
```

### Also Good
```rust
// Traces to: FR-DOMAIN-003, FR-API-002
// Verify forward-only state transitions and audit emission
#[tokio::test]
async fn feature_transition_enforces_state_machine() {
```

### Avoid
```rust
// Traces to: FR-API-001
#[tokio::test]
async fn test1() {  // unclear purpose
```

---

## Checklist for Adding Annotations

- [ ] Test file identified and location noted
- [ ] Test function purpose understood
- [ ] Matching FR ID(s) found in FR_TRACKER.md or CODE_ENTITY_MAP.md
- [ ] Comment added: `// Traces to: FR-XXX-YYY`
- [ ] Multiple FRs separated by commas if applicable
- [ ] Brief description added (one line explaining what FR aspect is verified)
- [ ] File committed with message referencing FR IDs

---

## Common FR Groupings by Test Type

### Feature CRUD Tests
- FR-API-001 (feature CRUD endpoints)
- FR-DOMAIN-001 (Feature entity)
- FR-CLI-001, FR-CLI-002 (feature create/list commands)

### State Transition Tests
- FR-API-002 (transition endpoint)
- FR-DOMAIN-002, FR-DOMAIN-003 (FeatureState and forward-only transitions)
- FR-DOMAIN-004 (StateTransition recording)
- FR-CLI-003 (transition command)
- FR-AUDIT-001, FR-AUDIT-002 (audit entry and hashing)

### Work Package Tests
- FR-API-003 (WP endpoints)
- FR-DOMAIN-005, FR-DOMAIN-006 (WorkPackage entity and WpState)
- FR-DOMAIN-007 (WP dependencies and cycle detection)
- FR-CLI-004, FR-CLI-005 (wp create/list/status commands)

### Auth Tests
- FR-API-007 (API key authentication)
- FR-DOMAIN-015 (ApiKey entity)

### Governance Tests
- FR-GOVERN-002 (evidence blocking)
- FR-API-004 (audit trail endpoint)
- FR-AUDIT-001 through FR-AUDIT-006 (audit operations)

### Integration Tests
- FR-PLANE-* (Plane.so sync)
- FR-GIT-* (Git worktree, PR creation)
- FR-EVENTS-* (event publishing)
- FR-P2P-* (peer replication)

---

## Examples from Codebase

### From core_routes.rs (already updated)

```rust
#[tokio::test]
async fn health_no_auth_required() {
    // Traces to: FR-API-005, FR-DOMAIN-014
    // Verify that /health endpoint returns service health without authentication
    let server = setup_test_server().await;
    let resp = server.get("/health").await;
    resp.assert_status_ok();
    // ...
}
```

### Pattern for Other Files

**features_work_packages.rs:**
```rust
#[tokio::test]
async fn create_work_package_for_feature() {
    // Traces to: FR-API-003, FR-DOMAIN-005, FR-DOMAIN-006
    // Verify work package creation with state validation
```

**audit_governance.rs:**
```rust
#[tokio::test]
async fn verify_audit_chain_integrity() {
    // Traces to: FR-AUDIT-003
    // Verify hash chain verification detects tampering
```

**module_cycle.rs:**
```rust
#[tokio::test]
async fn cycle_state_transitions() {
    // Traces to: FR-DOMAIN-011
    // Verify Cycle state machine (Draft→Active→Completed→Archived)
=======
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
>>>>>>> origin/main
```

---

<<<<<<< HEAD
## Automated Checking (Future)

Once all tests are annotated, this command can verify coverage:

```bash
# Find all test functions
grep -r "#\[.*test.*\]" crates/ | wc -l

# Find annotated tests
grep -r "Traces to: FR-" crates/ | wc -l

# Find orphaned tests (no FR)
grep -r "#\[.*test.*\]" crates/ | while read line; do
  file=$(echo $line | cut -d: -f1)
  func=$(echo $line | grep -oE 'fn [a-z_]+' | head -1)
  if ! grep -A2 "$func" "$file" | grep -q "Traces to:"; then
    echo "Missing FR annotation: $file::$func"
  fi
done
```

---

## Next Steps

1. **Start with key test files:**
   - `crates/agileplus-api/tests/api_integration/` (multiple modules)
   - `crates/agileplus-cli/tests/`
   - `crates/agileplus-domain/tests/`

2. **Add annotations** using this guide

3. **Reference FR_TRACKER.md** to ensure coverage

4. **Commit** with message: `"test(traceability): add FR annotations to <module> tests"`

5. **Verify** by grepping for "Traces to:" and comparing to FR categories

---

## Reference

- **FR_TRACKER.md** - Comprehensive list of all 63 FRs with status
- **CODE_ENTITY_MAP.md** - Code entity to FR mapping
- **FUNCTIONAL_REQUIREMENTS.md** - Authoritative requirement specifications
- **FR_TRACEABILITY_COMPLETION.md** - Completion report and gap analysis

=======
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
>>>>>>> origin/main
