# Bifrost Extensibility Framework

## Summary

Create unified `phenotype-extensibility` framework to consolidate adapter/plugin/extension patterns fragmented across Thegent, AgilePlus, Pheno-CLI, and AgentAPI. Eliminates 5 major duplication patterns (registry, lifecycle, policy, config, testing) and reduces extension code by 73% (7,310 LOC).

## Goals

- Extract shared adapter abstractions (BaseAdapter, AdapterRegistry, LifecycleHooks)
- Create unified testing framework (ContractTestBase, MockRegistry)
- Publish phenotype-extensibility to GitHub Packages
- Establish extension architecture as canonical pattern
- Enable 75% boilerplate reduction for new adapters

## Success Criteria

- ✓ phenotype-extensibility crate published and tested
- ✓ All 4 projects migrated to use framework
- ✓ 7,310 LOC eliminated across ecosystem
- ✓ 75% boilerplate reduction for new adapters
- ✓ Unified documentation (ARCHITECTURE.md, ADDING_ADAPTERS.md)
- ✓ Zero duplication of registry, lifecycle, policy patterns

## Scope

**Total LOC Savings**: 7,310 LOC (73% reduction)
- Adapter Registry & Lifecycle: 640 LOC
- Error Handling Framework: 280 LOC
- Configuration System: 1,000 LOC
- Test Adapter Framework: 1,400 LOC
- Bifrost Routing: 280 LOC
- External Package Wrapping: 1,450 LOC
- Plus: 380 LOC from miscellaneous consolidation

**Framework Size**: ~500 LOC (new code)
**Projects Affected**: 4 (Thegent, AgilePlus, Pheno-CLI, AgentAPI)

## Work Packages (7 Phases)

### Phase 1: Framework Foundation (2 weeks)

#### WP01: Core Traits & Registry (4-5 min)
- Create BaseAdapter trait
- Create AdapterRegistry<T> generic
- Create LifecycleHooks trait
- Implement PolicyValidator trait
- Unit tests for all traits

#### WP02: Error & Policy (3-4 min)
- AdapterError types
- Policy enforcement
- Validation helpers
- Integration tests

#### WP03: Test Framework (3-4 min)
- TestHarness<T>
- MockAdapter trait
- Contract matchers
- Assertion helpers

#### WP04: Multi-Language Bindings (4-5 min)
- Python (PyO3) bindings
- Go (WASM/FFI) bindings
- TypeScript (Node FFI) bindings

#### WP05: Documentation (3-4 min)
- ARCHITECTURE.md
- ADDING_ADAPTERS.md
- TESTING_GUIDE.md
- 4 working examples

### Phase 2: Adapter Consolidation (2 weeks)

#### WP06: AgilePlus Migration (3-4 min)
- Per-crate handlers → use AdapterRegistry
- Remove 400 LOC boilerplate
- Update tests

#### WP07: Pheno-CLI Migration (2-3 min)
- Factory pattern → use BaseAdapter + Registry
- Remove 120 LOC
- Add tests

#### WP08: Thegent Registry (2-3 min)
- Delegate to phenotype-extensibility
- Remove 240 LOC (320 → 80)

#### WP09: Thegent Lifecycle (2-3 min)
- Use LifecycleHooks trait
- Remove 480 LOC (632 → 150)
- Integration tests

### Phase 3-5: Library Consolidation (Parallel with Phase 2)

#### WP10: Error Consolidation (1 week, parallel)
- phenotype-error-core
- Migrate all projects
- 280 LOC saved

#### WP11: Config Consolidation (1 week, parallel)
- phenotype-config-core
- Migrate all projects
- 1,000 LOC saved

#### WP12: Test Framework (2 weeks, parallel)
- phenotype-test-adapters
- Unified test base classes
- 1,400 LOC saved

### Phase 6: OSS Integration (2 weeks)

#### WP13: Container Management (2-3 min)
- Wrap bollard (v0.18)
- Replace phenotype-vessel
- 500 LOC saved

#### WP14: Additional Wrappers (3-4 min)
- governor (circuit breaker)
- similar (diff/patch)
- chacha20poly1305 (crypto)
- 950 LOC saved total

### Phase 7: Routing Consolidation (1 week)

#### WP15: Bifrost Routing (2-3 min)
- Extract from AgentAPI
- Create phenotype-routing
- Remove thegent duplication
- 280 LOC saved

## Risk Mitigation

| Risk | Severity | Mitigation |
|------|----------|-----------|
| Migration complexity | HIGH | Phased approach, feature branches, parallel PRs |
| Backward compatibility | MEDIUM | Compatibility layer, 3-release deprecation |
| Performance overhead | MEDIUM | Trait dispatch benchmarks, optimize if needed |
| Adoption resistance | MEDIUM | Examples + documentation, gradual rollout |

## Timeline

- **Total Duration**: 12 weeks
- **Phase 1**: 2 weeks (foundation)
- **Phase 2**: 2 weeks (consolidation, parallel with Phase 3-5)
- **Phase 3-5**: 5 weeks (error/config/testing, parallel)
- **Phase 6**: 2 weeks (OSS integration)
- **Phase 7**: 1 week (routing)

## Related

- `docs/worklogs/BIFROST_EXTENSIONS_COMPREHENSIVE_LOC_AUDIT_2026-03-29.md` — Full audit
- `docs/worklogs/LOC_AUDIT_DEEP_FINDINGS_2026-03-29.md` — Consolidated findings
- Phench Phase 1-2 (parallel execution, 5 weeks)
- CIV Consolidation (depends on Phase 2-5 libraries)

## Notes

- Bifrost is currently fragmented across 4 projects with 3x duplication of core patterns
- Framework extraction enables consistent extension architecture across ecosystem
- 7,310 LOC reduction potential is highest-impact consolidation opportunity
- All 4 projects benefit from unified testing + error handling frameworks
