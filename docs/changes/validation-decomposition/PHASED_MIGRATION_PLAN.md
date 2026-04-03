# Validation System Decomposition: Phased Migration Plan

**Project**: Refactor CLI validation from monolithic `validate.rs` (674 LOC) to modular, registry-based system  
**Timeline**: 3 phases over 2-3 weeks  
**Owner**: TBD (assign during sprint planning)  
**Status**: Planning Phase  

---

## Executive Summary

- **Goal**: Reduce validate.rs from 674 LOC to <100 LOC via registry-based modular system
- **Benefit**: 50% LOC reduction, 100% plugin extensibility, zero duplication
- **Risk**: Low (new system runs parallel, old code removed incrementally)
- **Effort**: ~12 hours wall-clock (3 agents, 4 hours each, or 1 agent sequential)

---

## Phase 1: Trait Foundation & Core Rules

**Duration**: 3-4 hours (1 agent)  
**Deliverables**: Traits, core rules, field validators  
**Atomic Commit**: `feat(validation): implement trait hierarchy and core rules`

### Work Items

#### WI-1.1: Trait Definitions (1 hour)
- **File**: `crates/phenotype-validation/src/traits/mod.rs`
- **Subtasks**:
  - [ ] Define `ValidationRule` trait with `validate()`, `name()`, `severity()`
  - [ ] Define `FieldValidator` struct with `add_rule()`, `validate()`
  - [ ] Define `CommandValidator` struct with `add_field()`, `validate()`
  - [ ] Define `ValidationError` enum with context stack
  - [ ] Add 15+ tests for trait behavior
- **Test**: `cargo test traits::` must pass

#### WI-1.2: Core Rules Implementation (1.5 hours)
- **Files**: 
  - `crates/phenotype-validation/src/traits/rule.rs`
- **Subtasks**:
  - [ ] Implement `RequiredRule` (non-empty check)
  - [ ] Implement `LengthRule` (min/max/range)
  - [ ] Implement `PatternRule` (regex-based)
  - [ ] Implement `NumericRangeRule` (min/max numeric)
  - [ ] Implement `CustomRule` (closure-based)
  - [ ] Add 30+ tests covering edge cases
- **Test**: `cargo test traits::rule::` must pass with 80%+ coverage

#### WI-1.3: Field Validator Implementation (1 hour)
- **File**: `crates/phenotype-validation/src/traits/field_validator.rs`
- **Subtasks**:
  - [ ] Implement `FieldValidator::new()` (empty constructor)
  - [ ] Implement `add_rule()` (builder pattern)
  - [ ] Implement `validate()` (return first error)
  - [ ] Implement `validate_all()` (collect all errors)
  - [ ] Implement `rule_count()` and `rule_names()` introspection
  - [ ] Add 10+ tests
- **Test**: `cargo test traits::field_validator::` must pass

### Success Criteria for Phase 1
- [ ] All 4 trait files created and compile
- [ ] 55+ tests passing
- [ ] 0 compiler warnings
- [ ] Documentation comments on all public items
- [ ] Atomic commit pushed to branch: `validation/phase-1-traits`

### Rollback Plan
Delete `crates/phenotype-validation/src/traits/` and revert Cargo.toml dependencies.

---

## Phase 2: Registry & Presets

**Duration**: 4-5 hours (1 agent)  
**Deliverables**: Global registry, preset validators  
**Atomic Commit**: `feat(validation): implement registry pattern and preset validators`

### Work Items

#### WI-2.1: ValidatorRegistry (1.5 hours)
- **File**: `crates/phenotype-validation/src/registry.rs`
- **Subtasks**:
  - [ ] Define `ValidatorFactory` type (fn pointer)
  - [ ] Implement global `VALIDATOR_REGISTRY` using `once_cell::Lazy`
  - [ ] Implement `ValidatorRegistry::register(name, factory)`
  - [ ] Implement `ValidatorRegistry::get(name) -> Option<FieldValidator>`
  - [ ] Implement `ValidatorRegistry::list() -> Vec<String>`
  - [ ] Implement `ValidatorRegistry::exists(name) -> bool`
  - [ ] Add 8+ tests
- **Test**: `cargo test registry::` must pass, registry is thread-safe

#### WI-2.2: Preset Validators (2 hours)
- **File**: `crates/phenotype-validation/src/presets.rs` (refactor)
- **Subtasks**:
  - [ ] Implement `email_validator()` preset
  - [ ] Implement `url_validator()` preset
  - [ ] Implement `username_validator()` preset
  - [ ] Implement `slug_validator()` preset
  - [ ] Implement `strong_password_validator()` preset
  - [ ] Implement `moderate_password_validator()` preset
  - [ ] Implement `basic_password_validator()` preset
  - [ ] Implement `us_phone_validator()` preset
  - [ ] Implement `uuid_validator()` preset
  - [ ] Implement `register_presets()` function (registers all 9 presets)
  - [ ] Add 20+ tests (2-3 per preset)
- **Test**: `cargo test presets::` must pass, all presets discoverable via registry

#### WI-2.3: lib.rs Integration (1 hour)
- **File**: `crates/phenotype-validation/src/lib.rs`
- **Subtasks**:
  - [ ] Update documentation with architecture overview
  - [ ] Re-export all public traits and registry
  - [ ] Add prelude module for common imports
  - [ ] Update Cargo.toml with new dependencies (regex, uuid, once_cell)
  - [ ] Verify crate builds: `cargo build`
  - [ ] Generate docs: `cargo doc --open`
- **Test**: `cargo build && cargo test` must pass

### Success Criteria for Phase 2
- [ ] Registry fully functional (register, get, list, exists)
- [ ] 9 preset validators registered and tested
- [ ] 28+ tests passing
- [ ] 0 compiler warnings
- [ ] Backward-compat shims for old `validators.rs` API (optional)
- [ ] Atomic commit pushed to branch: `validation/phase-2-registry`

### Rollback Plan
Delete `crates/phenotype-validation/src/registry.rs`, revert `presets.rs` to old version.

---

## Phase 3: Migration & Documentation

**Duration**: 2-3 hours (1 agent)  
**Deliverables**: Examples, migration guide, CLI integration example  
**Atomic Commit**: `docs(validation): add migration guide and examples`

### Work Items

#### WI-3.1: Documentation (1 hour)
- **Files**:
  - `docs/guides/VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md` (240 lines)
  - `docs/changes/validation-decomposition/MIGRATION_GUIDE.md` (350 lines)
  - `docs/guides/VALIDATION_EXAMPLES.md` (500 lines)
- **Subtasks**:
  - [ ] Create architecture diagram (Mermaid)
  - [ ] Write migration checklist
  - [ ] Document 10 code examples (before/after)
  - [ ] Create plugin extensibility example
  - [ ] Document registry discovery pattern
  - [ ] Add troubleshooting FAQ
- **Test**: Files are valid Markdown, Mermaid diagrams render

#### WI-3.2: Examples (1 hour)
- **Files**:
  - `crates/phenotype-validation/examples/basic_usage.rs` (80 LOC)
  - `crates/phenotype-validation/examples/custom_validator.rs` (120 LOC)
  - `crates/phenotype-validation/examples/command_validator.rs` (100 LOC)
- **Subtasks**:
  - [ ] Example 1: Using presets from registry
  - [ ] Example 2: Creating custom field validator
  - [ ] Example 3: Command-level validation
  - [ ] Example 4: Custom rules (business logic)
  - [ ] Example 5: Plugin registration
  - [ ] Compile all examples: `cargo build --examples`
- **Test**: All examples compile and run

#### WI-3.3: CLI Integration Example (1 hour)
- **File**: `docs/changes/validation-decomposition/CLI_INTEGRATION_EXAMPLE.md` (250 LOC)
- **Subtasks**:
  - [ ] Show how to migrate `CreatePlanCommand` validator
  - [ ] Show how to migrate `UpdatePlanCommand` validator
  - [ ] Show error handling in CLI output
  - [ ] Show registry initialization in app startup
  - [ ] Show testing strategy
  - [ ] Include before/after LOC comparison
- **Test**: Examples are valid Rust (compile-checkable via code blocks)

### Success Criteria for Phase 3
- [ ] All documentation files created
- [ ] All examples compile and run successfully
- [ ] 0 Markdown linting errors (via vale/markdownlint)
- [ ] 0 broken links in cross-references
- [ ] CLI integration example shows >50% LOC reduction
- [ ] Atomic commit pushed: `docs(validation): complete migration guide and examples`

### Rollback Plan
Delete documentation and examples; they do not affect crate functionality.

---

## Cross-Phase Work Items

### Testing & Verification (Throughout all phases)

#### QA-1: Comprehensive Test Suite
- **Effort**: ~2 hours (distributed across phases)
- **Subtasks**:
  - [ ] Unit tests: 80+ tests across all modules
  - [ ] Integration tests: Registry + CLI integration
  - [ ] Property-based tests: Rule composition (optional for Phase 1)
  - [ ] Benchmark: Validator creation overhead (optional for Phase 2)
  - [ ] Coverage: `cargo tarpaulin --out Html` shows 85%+ coverage
- **Test**: `cargo test --all` passes with 0 failures

#### QA-2: Linting & Code Quality
- **Effort**: ~1 hour (distributed)
- **Subtasks**:
  - [ ] Clippy: `cargo clippy --all -- -D warnings` passes
  - [ ] Format: `cargo fmt --check` passes
  - [ ] Documentation: `cargo doc --no-deps --document-private-items` no warnings
  - [ ] Dependencies: `cargo audit` reports 0 vulnerabilities
- **Test**: All quality checks pass in CI

#### QA-3: Backward Compatibility
- **Effort**: ~1 hour (distributed)
- **Subtasks**:
  - [ ] Old `is_valid_email()` function still available
  - [ ] Old `is_valid_url()` function still available
  - [ ] Old `is_valid_uuid()` function still available
  - [ ] Tests exist for legacy API
- **Test**: Legacy tests pass

### Documentation Review (End of Phase 3)
- [ ] Architecture Decision Record (ADR) written
- [ ] User guide updated in main README
- [ ] API changes documented in CHANGELOG.md
- [ ] Migration guide reviewed by team lead

---

## Dependencies & Blockers

### Required Dependencies
```toml
regex.workspace = true       # PatternRule implementation
uuid.workspace = true        # UUID validation
once_cell.workspace = true   # Lazy registry singleton
```

### No Blockers Identified
- All traits compile without external dependencies
- Registry uses standard library + once_cell
- Tests can use mocks/fixtures

---

## Risk & Mitigation

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| API instability in Phase 1 | Medium | High | Freeze Phase 1 API after WI-1.3 completion |
| Registry contention (multi-threaded) | Low | Medium | Use `Lazy<Mutex<>>` (already planned) |
| CLI integration takes longer than Phase 3 | Medium | Medium | Phase 3 only documents; CLI migration is separate task |
| Backward-compat issues with old code | Low | Low | Keep old validators.rs module as bridge |

---

## Success Metrics (All 3 Phases)

By end of Phase 3:

- [ ] **Code Quality**: 0 compiler warnings, 0 clippy issues, 85%+ test coverage
- [ ] **LOC Reduction**: CLI `validate.rs` reduced from 674 LOC → <100 LOC (80%+ reduction)
- [ ] **Extensibility**: 3+ custom validators registered and working
- [ ] **Documentation**: All 3 docs complete, all 5 examples working
- [ ] **Testing**: 80+ unit tests + 10+ integration tests, all passing
- [ ] **Performance**: Validator creation overhead < 1ms (single rule)
- [ ] **Backward Compatibility**: Old API still works (legacy shim)

---

## Execution Notes

### Parallel Execution (Recommended)
- **Team**: 2-3 agents
- **Agent A**: WI-1.1 + WI-1.2 (3.5 hours) → Phase 1 complete
- **Agent B**: WI-2.1 + WI-2.2 (3.5 hours) → Phase 2 complete (depends on Phase 1)
- **Agent C**: WI-3.1 + WI-3.2 + WI-3.3 (3 hours) → Phase 3 complete (depends on Phase 2)
- **Wall-clock**: ~4 hours (critical path: Phase 1 → Phase 2 → Phase 3)

### Sequential Execution (Alternative)
- **Team**: 1 agent
- **Order**: WI-1.1 → WI-1.2 → WI-1.3 → WI-2.1 → WI-2.2 → WI-2.3 → WI-3.1 → WI-3.2 → WI-3.3
- **Wall-clock**: ~12 hours (1 agent, sequential)

---

## Post-Completion Tasks (Not in Phase 3)

These are follow-up tasks for after validation system is ready:

1. **CLI Integration** (separate task): Migrate all CLI commands to use registry
2. **Async Rules** (Phase 2 enhancement): Implement async validation rules
3. **i18n Support** (Phase 3 enhancement): Localize error messages
4. **Performance Optimization**: Benchmark and cache hot validators
5. **Schema Integration**: Connect validation to JSON Schema / OpenAPI

---

## Sign-Off & Approval

| Role | Name | Approval | Date |
|------|------|----------|------|
| Implementer | TBD | [ ] | __ / __ / ____ |
| Reviewer | TBD | [ ] | __ / __ / ____ |
| Owner | TBD | [ ] | __ / __ / ____ |

---

## References

- **Architecture**: `docs/guides/VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md`
- **Migration**: `docs/changes/validation-decomposition/MIGRATION_GUIDE.md`
- **Examples**: `docs/guides/VALIDATION_EXAMPLES.md`
- **Crate**: `/crates/phenotype-validation/`
- **Related ADR**: TBD (to be created in Phase 1 completion)

