# Sample Feature Plan

**Feature**: sample-feature
**Phase**: Phase 1 — Foundation
**Generated**: 2026-01-15T00:00:00Z

## Work Packages

### WP01 — Domain model and state machine

**Goal**: Implement the `Feature` entity, `FeatureState` enum, and state machine
transition logic in `agileplus-domain`.

**File scope**:
- `crates/agileplus-domain/src/domain/feature.rs`
- `crates/agileplus-domain/src/domain/state_machine.rs`
- `crates/agileplus-domain/src/error.rs`

**Dependencies**: none

**Acceptance criteria**:
- All 8 `FeatureState` variants are implemented with ordinal ordering.
- `Feature::transition()` returns `Ok(TransitionResult)` for valid transitions.
- `Feature::transition()` returns `Err(DomainError::InvalidTransition)` for backward moves.
- Unit tests cover all 7 forward transitions, 8 no-op transitions, and 7 backward transitions.

**FR references**: FR-001, FR-002

---

### WP02 — Audit trail implementation

**Goal**: Implement the `AuditEntry` type, `hash_entry()` function, and
`AuditChain::verify_chain()` in `agileplus-domain`.

**File scope**:
- `crates/agileplus-domain/src/domain/audit.rs`

**Dependencies**: WP01

**Acceptance criteria**:
- `hash_entry()` produces a deterministic SHA-256 hash of the entry fields.
- `AuditChain::verify_chain()` returns `Ok(())` for a valid chain.
- `AuditChain::verify_chain()` returns `Err(AuditChainError::HashMismatch)` for tampered entries.
- A chain of 5 entries can be verified in < 1 ms.

**FR references**: FR-003

---

### WP03 — Governance contract binding

**Goal**: Implement `GovernanceContract`, `GovernanceRule`, and `EvidenceRequirement`
types and the governance evaluation logic.

**File scope**:
- `crates/agileplus-domain/src/domain/governance.rs`

**Dependencies**: WP01

**Acceptance criteria**:
- A `GovernanceContract` can hold multiple rules for different transitions.
- Given a set of collected `Evidence`, the governance evaluator identifies missing requirements.
- Rules for the `implementing -> validated` transition are evaluated by default.

**FR references**: FR-003
