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
```

---

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
```

---

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

