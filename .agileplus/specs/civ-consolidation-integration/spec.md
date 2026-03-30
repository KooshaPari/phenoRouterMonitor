# CIV Consolidation & Integration

## Summary

Integrate CIV (Simulation Engine) with Phase 2 consolidated libraries: phenotype-error-core, phenotype-config-core, phenotype-git-core. Replace domain-specific implementations with shared libraries, saving 1,350-2,450 LOC (35-45%) while maintaining simulation core domain logic.

## Goals

- Integrate CIV with Phase 2 consolidated libraries
- Replace error handling with phenotype-error-core
- Replace configuration system with phenotype-config-core
- Integrate event sourcing, state machine, policy engine
- Add telemetry and testing frameworks
- Achieve 35-45% LOC reduction

## Success Criteria

- ✓ CIV compiles with zero warnings
- ✓ All tests pass (100% coverage on domain logic)
- ✓ 1,350-2,450 LOC eliminated
- ✓ Determinism preserved for simulation runs
- ✓ No performance regression
- ✓ Event sourcing audit trails working
- ✓ Policy evaluation deterministic

## Scope

**LOC Savings Target**: 1,350-2,450 LOC (35-45%)
- Configuration management: 120-200 LOC
- Error handling: 100-150 LOC
- Event sourcing: 200-300 LOC
- State machine: 150-200 LOC
- Policy engine: 200-300 LOC
- Telemetry: 150-200 LOC
- Testing utilities: 80-100 LOC
- Git operations: estimated 50-100 LOC
- Domain logic (keep): 600-1,000 LOC

**Files Affected**: Estimated 20-30 files

## Work Packages (6 Phases)

### Phase 1: Error Consolidation (1 day)

#### WP01: Integrate phenotype-error-core (2-3 min)
- Map CIV error types → phenotype-error-core variants
- Replace local error enum with re-export + wrappers
- Update all error handling code
- Run test suite

**Expected Savings**: 100-150 LOC

### Phase 2: Configuration Management (1 day)

#### WP02: Integrate phenotype-config-core (3-4 min)
- Extract CIV config struct types
- Replace manual TOML/YAML parsing → ConfigLoader API
- Add scenario validation
- Update integration tests

**Expected Savings**: 120-200 LOC

### Phase 3: Event Sourcing (0.5 days)

#### WP03: Integrate phenotype-event-sourcing (2-3 min)
- Model CIV events (SimulationStarted, AgentMoved, PolicyChanged, etc.)
- Use phenotype-event-sourcing for audit trails
- Implement event replay for simulation checkpoints
- Add hash-chain integrity verification

**Expected Savings**: 200-300 LOC

### Phase 4: State Machine & Policy (1 day)

#### WP04: Integrate phenotype-state-machine (2-3 min)
- Model simulation states (Initialized, Running, Paused, Completed, Failed)
- Replace custom state tracking → phenotype-state-machine
- Add state transition guards
- Update workflow logic

**Expected Savings**: 150-200 LOC

#### WP05: Integrate phenotype-policy-engine (2-3 min)
- Replace custom policy evaluator → phenotype-policy-engine
- Migrate policies to TOML format
- Update policy validation
- Verify determinism

**Expected Savings**: 200-300 LOC

### Phase 5: Telemetry & Testing (0.5 days)

#### WP06: Integrate phenotype-telemetry (2-3 min)
- Replace logging boilerplate → phenotype-telemetry
- Add async-safe duration tracking
- Integrate with metrics collection
- Profile and optimize

**Expected Savings**: 150-200 LOC

#### WP07: Integrate phenotype-test-infra (2-3 min)
- Replace test utilities → phenotype-test-infra
- Migrate test fixtures to TestBuilder trait
- Use InMemoryStore for snapshot tests
- Run full test suite

**Expected Savings**: 80-100 LOC

### Phase 6: Verification & Optimization (0.5 days)

#### WP08: Integration Testing & Performance (3-4 min)
- Run full test suite (simulation domain tests)
- Verify event sourcing accuracy
- Check performance (determinism, throughput)
- Profile and optimize hot paths
- Document integration patterns

**Expected Savings**: 50-100 LOC (misc cleanup)

## Risk Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Determinism loss | CRITICAL | Feature branch, comprehensive testing, determinism verification tests |
| Event format changes | MEDIUM | Versioning in event schema, migration guide |
| Performance regression | MEDIUM | Benchmarks before/after, profile hot paths |
| Library API changes | LOW | Phase 2 libraries stable, version pinning |
| Integration complexity | MEDIUM | Phased approach, one library per day, parallel with other work |

## Timeline

- **Total Duration**: 4 days (after Phase 2 libraries available)
- **Phase 1**: 1 day (error consolidation)
- **Phase 2**: 1 day (config management)
- **Phase 3**: 0.5 days (event sourcing)
- **Phase 4**: 1 day (state machine + policy)
- **Phase 5**: 0.5 days (telemetry + testing)
- **Phase 6**: 0.5 days (verification)

**Prerequisite**: Phase 2 libraries (error-core, config-core, event-sourcing, state-machine, policy-engine, telemetry, test-infra) must be complete

## Dependent Libraries

| Library | Version | Status | Required For |
|---------|---------|--------|--------------|
| phenotype-error-core | 0.1.0 | Phase 2 | WP01 |
| phenotype-config-core | 0.1.0 | Phase 2 | WP02 |
| phenotype-event-sourcing | 0.1.0+ | Phase 1 | WP03 |
| phenotype-state-machine | 0.1.0+ | Phase 1 | WP04 |
| phenotype-policy-engine | 0.1.0+ | Phase 1 | WP04 |
| phenotype-telemetry | 0.1.0+ | Phase 1 | WP06 |
| phenotype-test-infra | 0.1.0+ | Phase 1 | WP07 |
| phenotype-git-core | 0.1.0 | Phase 2 | Optional (if CIV uses git) |

## Related

- `docs/worklogs/CIV_PROJECT_PHENOTYPE_ECOSYSTEM_LOC_AUDIT_2026-03-29.md` — Full audit
- `docs/worklogs/LOC_AUDIT_DEEP_FINDINGS_2026-03-29.md` — Consolidated findings
- Phench Phase 1-2 (parallel execution, 5 weeks)
- Bifrost Phase 1-7 (parallel execution, 12 weeks)
- Phase 2 Library Specifications (prerequisite)

## Notes

- CIV source currently not on filesystem; spec assumes standard Rust simulation architecture
- 35-45% LOC reduction is achievable without sacrificing domain logic
- Event sourcing crucial for simulation replay/verification
- State machine ensures deterministic workflow transitions
- Policy engine enables declarative rule evaluation (audit trail)
- Simulation core (600-1,000 LOC) remains domain-specific, not consolidated
