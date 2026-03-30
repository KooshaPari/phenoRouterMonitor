# Work Packages: phenotype-error-core Extraction

**Feature**: phenotype-error-core
**Total WPs**: 7
**Estimated Duration**: 10-15 minutes (agent-driven, parallel-eligible)

## WP1: Setup phenotype-error-core Crate

**ID**: phenotype-error-core-WP1
**Type**: Setup/Infrastructure
**Effort**: 2-3 minutes
**Status**: PENDING
**Dependencies**: None

### Tasks
1. Create new library crate: `cargo new --lib crates/phenotype-error-core`
2. Update `Cargo.toml`:
   - Add workspace inheritance: `[workspace.package]`
   - Add dependencies: `serde`, `thiserror` (from workspace)
   - Add features: `serde = ["dep:serde"]`
3. Update root `Cargo.toml` workspace members to include new crate
4. Create `src/lib.rs` with ErrorKind enum (14+ variants)
5. Add docstring examples and module-level documentation
6. Run `cargo check` to verify compilation

### Acceptance Criteria
- [ ] Crate compiles without errors
- [ ] ErrorKind enum has all 14 required variants
- [ ] thiserror::Error trait is properly derived
- [ ] serde support is optional/configurable
- [ ] From implementations exist for io::Error, serde_json::Error, regex::Error

### Success Metrics
- Compilation: 0 errors, 0 warnings
- `cargo test -p phenotype-error-core` passes
- Crate can be added as dependency in other crates

---

## WP2: Migrate phenotype-contracts Inbound Error

**ID**: phenotype-error-core-WP2
**Type**: Migration
**Effort**: 3-4 minutes
**Status**: PENDING
**Dependencies**: WP1

### Tasks
1. Add `phenotype-error-core` dependency to `phenotype-contracts/Cargo.toml`
2. Open `crates/phenotype-contracts/phenotype-contracts/src/ports/inbound/mod.rs`
3. Replace inbound::Error enum with type alias to phenotype-error-core::ErrorKind
4. Update error construction: `Error::NotFound(msg)` → `ErrorKind::NotFound(msg)`
5. Update all uses in UseCase, CommandHandler, QueryHandler traits
6. Run `cargo test -p phenotype-contracts` to verify
7. Fix any type mismatches or compilation errors

### Acceptance Criteria
- [ ] All inbound error types compile
- [ ] Error enum variants match ErrorKind
- [ ] All contract tests pass
- [ ] Error messages are preserved
- [ ] No breaking changes to public API (if aliased)

### Success Metrics
- `cargo test -p phenotype-contracts` passes
- No compilation warnings
- Error handling behavior unchanged

---

## WP3: Migrate phenotype-contracts Outbound Error

**ID**: phenotype-error-core-WP3
**Type**: Migration
**Effort**: 3-4 minutes
**Status**: PENDING
**Dependencies**: WP1

### Tasks
1. Open `crates/phenotype-contracts/phenotype-contracts/src/ports/outbound/mod.rs`
2. Replace outbound::Error enum with type alias to phenotype-error-core::ErrorKind
3. Update error construction in Repository, CachePort, SecretPort, EventPublisher, EventSubscriber traits
4. Verify all error scenarios are covered by ErrorKind variants
5. Run `cargo test -p phenotype-contracts` to verify
6. Fix any type mismatches or compilation errors

### Acceptance Criteria
- [ ] All outbound error types compile
- [ ] Error variants match ErrorKind
- [ ] All contract tests pass
- [ ] Port interface semantics preserved
- [ ] No breaking changes to public API

### Success Metrics
- `cargo test -p phenotype-contracts` passes (inbound + outbound)
- All Repository, CachePort, SecretPort traits work as expected
- No error handling regressions

---

## WP4: Migrate phenotype-event-sourcing

**ID**: phenotype-error-core-WP4
**Type**: Migration
**Effort**: 4-5 minutes
**Status**: PENDING
**Dependencies**: WP1

### Tasks
1. Add `phenotype-error-core` dependency to both event-sourcing Cargo.tomls
2. Create EventSourceError wrapper enum in `src/error.rs`:
   ```rust
   pub enum EventSourceError {
       DuplicateSequence(String),
       SequenceGap { expected: i64, actual: i64 },
       InvalidHash(String),
       Other(ErrorKind),
   }
   ```
3. Update EventStoreError to use ErrorKind for common variants (NotFound, StorageError)
4. Update HashError to use ErrorKind for common variants
5. Add From<ErrorKind> and Into<ErrorKind> implementations
6. Update EventSourcingError to wrap domain errors
7. Run `cargo test -p phenotype-event-sourcing` to verify
8. Fix any compilation errors

### Acceptance Criteria
- [ ] EventSourceError wrapper preserves domain semantics
- [ ] DuplicateSequence, SequenceGap, InvalidHash variants preserved
- [ ] Common errors (NotFound, Storage) use ErrorKind
- [ ] All conversions work (ErrorKind ↔ EventSourceError)
- [ ] All event sourcing tests pass
- [ ] Error messages are preserved

### Success Metrics
- `cargo test -p phenotype-event-sourcing` passes
- Event sourcing error handling works end-to-end
- Domain-specific error information is not lost

---

## WP5: Migrate phenotype-policy-engine

**ID**: phenotype-error-core-WP5
**Type**: Migration
**Effort**: 3-4 minutes
**Status**: PENDING
**Dependencies**: WP1

### Tasks
1. Add `phenotype-error-core` dependency to policy-engine Cargo.toml
2. Create PolicyError wrapper enum in `src/error.rs`:
   ```rust
   pub enum PolicyError {
       RegexCompilation { pattern: String, source: regex::Error },
       InvalidConfiguration(String),
       Other(ErrorKind),
   }
   ```
3. Update PolicyEngineError to delegate common errors to ErrorKind:
   - EvaluationError → ErrorKind::Internal
   - LoadError → ErrorKind::Config
   - PolicyNotFound → ErrorKind::NotFound
   - SerializationError → ErrorKind::Serialization
4. Add From<ErrorKind> and Into<ErrorKind> implementations
5. Run `cargo test -p phenotype-policy-engine` to verify
6. Fix any compilation errors

### Acceptance Criteria
- [ ] PolicyError wrapper preserves domain semantics
- [ ] RegexCompilation and InvalidConfiguration variants preserved
- [ ] Common errors delegate to ErrorKind
- [ ] All conversions work (ErrorKind ↔ PolicyError)
- [ ] All policy engine tests pass
- [ ] Error messages are preserved

### Success Metrics
- `cargo test -p phenotype-policy-engine` passes
- Policy evaluation error handling works end-to-end
- Domain-specific error information (regex source) is not lost

---

## WP6: Verification & Integration Tests

**ID**: phenotype-error-core-WP6
**Type**: Testing & Verification
**Effort**: 4-5 minutes
**Status**: PENDING
**Dependencies**: WP2, WP3, WP4, WP5

### Tasks
1. Run full test suite: `cargo test --workspace`
2. Add integration tests for cross-crate error propagation:
   - contracts (inbound) → event-sourcing error conversion
   - contracts (outbound) → policy-engine error conversion
3. Add serialization tests for ErrorKind (serde, JSON)
4. Verify error messages are consistent across crates
5. Add documentation examples for error handling patterns
6. Run `cargo clippy` and fix any warnings
7. Verify no regressions in existing error handling

### Acceptance Criteria
- [ ] All tests pass (unit + integration): `cargo test --workspace`
- [ ] Serde serialization/deserialization works
- [ ] Cross-crate error propagation works
- [ ] No clippy warnings
- [ ] Error message consistency verified
- [ ] Documentation examples provided

### Success Metrics
- `cargo test --workspace` passes with 0 failures
- Serde round-trip: ErrorKind → JSON → ErrorKind
- Cross-crate error handling verified
- `cargo clippy --all-targets` returns 0 warnings

---

## WP7: PR Creation & Merge

**ID**: phenotype-error-core-WP7
**Type**: Integration
**Effort**: 2-3 minutes
**Status**: PENDING
**Dependencies**: WP6

### Tasks
1. Create feature branch: `feat/extract-phenotype-error-core`
2. Stage all changes: crate creation, migrations, tests
3. Create commit with clear, descriptive message:
   ```
   feat: extract phenotype-error-core shared error library

   Consolidate 5 error enums from 4 crates (442 LOC) into single
   phenotype-error-core library. Achieves 180 LOC reduction (49%).

   Changes:
   - Create phenotype-error-core with ErrorKind enum (14 variants)
   - Migrate phenotype-contracts inbound/outbound errors
   - Migrate phenotype-event-sourcing (preserve domain errors)
   - Migrate phenotype-policy-engine (preserve domain errors)
   - Add comprehensive error conversions (From impls)
   - Add integration tests for cross-crate error propagation

   Affected crates: phenotype-contracts, phenotype-event-sourcing,
   phenotype-policy-engine, phenotype-cache-adapter

   Fixes: Code duplication in error handling across 4 crates
   ```
4. Push to origin: `git push -u origin feat/extract-phenotype-error-core`
5. Create PR with detailed description
6. Verify CI passes (or merge with --admin if billing constrained)
7. Merge to main

### Acceptance Criteria
- [ ] Branch created and pushed successfully
- [ ] PR created with detailed description
- [ ] CI checks pass (or acknowledged as billing-constrained)
- [ ] Changes merged to main
- [ ] Commit history is clean and readable

### Success Metrics
- PR merged to main
- Commit appears in `git log`
- main branch reflects all changes
- No merge conflicts

---

## Overall Success Criteria

✓ All WPs completed
✓ Test suite passes: `cargo test --workspace`
✓ All 4 crates compile without warnings
✓ Error handling behavior unchanged (0 regressions)
✓ LOC reduction achieved: ≥180 LOC
✓ PR merged to main with clear commit history

## Parallel Execution Plan

**Parallelizable Groups**:
- WP2 & WP3 can run in parallel (both contract migrations)
- WP4 & WP5 can run after WP2/WP3 (event-sourcing and policy-engine migrations)
- WP6 must wait for WP2-WP5 (integration testing)
- WP7 must wait for WP6 (PR/merge)

**Recommended Execution Order**:
1. WP1 (setup)
2. WP2 + WP3 in parallel (contract migrations)
3. WP4 + WP5 in parallel (domain-specific migrations)
4. WP6 (verification)
5. WP7 (PR/merge)

**Estimated Total Time**: 10-15 minutes wall clock (agent-driven)
