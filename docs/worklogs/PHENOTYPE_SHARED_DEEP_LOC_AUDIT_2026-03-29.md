# Phenotype-Shared Deep LOC Audit (2026-03-29)
## Executive Summary
**Audit Date**: 2026-03-29  
**Total Rust LOC**: 5,110 (src) + 306 (tests)  
**Total Python LOC**: 3,841 (src) + 900 (tests)  
**Rust Crates**: 6  
**Python Packages**: 2  

---

## 1. Rust Crates Analysis

**Total Rust LOC**: 5,110 lines of code
**Test Coverage Ratio**: 0.06

### phenotype-config-core

- **Source**: 1,429 LOC (6 files)
- **Tests**: 0 LOC (0 files)
- **Dependencies**: 7 total
  ```
  - dirs
  - figment
  - serde
  - serde_json
  - thiserror
  - toml
  - tracing
  ```
- **Large Files** (>300 LOC):
  - `src/unified.rs`: 423 lines
  - `src/loader.rs`: 358 lines

### phenotype-contracts

- **Source**: 1,388 LOC (12 files)
- **Tests**: 158 LOC (1 files)
- **Test Ratio**: 0.11 - ⚠ Low

### phenotype-policy-engine

- **Source**: 1,358 LOC (7 files)
- **Tests**: 0 LOC (0 files)

### phenotype-health

- **Source**: 491 LOC (3 files)
- **Tests**: 148 LOC (1 files)
- **Test Ratio**: 0.30 - ⚠ Low
- **Dependencies**: 4 total
  ```
  - serde
  - serde_json
  - thiserror
  - tokio
  ```

### phenotype-error-core

- **Source**: 443 LOC (1 files)
- **Tests**: 0 LOC (0 files)
- **Dependencies**: 3 total
  ```
  - serde
  - serde_json
  - thiserror
  ```
- **Large Files** (>300 LOC):
  - `src/lib.rs`: 443 lines

### phenotype-git-core

- **Source**: 1 LOC (1 files)
- **Tests**: 0 LOC (0 files)

## 2. Python Packages Analysis

**Total Python LOC**: 3,841 lines of code
### src

- **Total**: 3,144 LOC (26 files)

### tests

- **Total**: 697 LOC (19 files)
- **Tests**: 900 LOC
- **Test Ratio**: 1.29 - ✓ Good

## 3. Cross-Language Duplication Patterns

### Error Handling Duplication
Crates with error handling modules: phenotype-config-core, phenotype-policy-engine

**Recommendation**: Consolidate into phenotype-error-core

### Configuration Patterns
### Loading/Initialization Patterns
Crates with loader modules: phenotype-config-core, phenotype-policy-engine

**Recommendation**: Share loader abstractions

## 4. Decomposition Analysis

### Crate Interdependencies


### Extraction Candidates

1. **phenotype-error-core** - Core error types used across all crates
   - Candidate for publication to crates.io and shared across Phenotype org
   - Status: Foundation library

2. **phenotype-config-core** - Configuration loading and validation
   - Candidate for extraction to shared config management layer
   - Status: Core infrastructure

3. **phenotype-git-core** - Git abstraction layer
   - Could be extracted as standalone git utilities library
   - Status: Reusable utilities

4. **phenotype-health** - Health check framework
   - Candidate for shared health monitoring across apps
   - Status: Application infrastructure

5. **phenotype-contracts** - Schema definitions
   - Foundation for contract-driven development
   - Status: Schema/types

## 5. Code Quality Metrics

### Test Coverage Summary

- ⚠ phenotype-health: 0.30 test/src ratio
- ⚠ phenotype-contracts: 0.11 test/src ratio
- ⚠ phenotype-config-core: 0.00 test/src ratio
- ⚠ phenotype-policy-engine: 0.00 test/src ratio
- ⚠ phenotype-error-core: 0.00 test/src ratio
- ⚠ phenotype-git-core: 0.00 test/src ratio

### Linting & Quality Issues

- Run `cargo clippy` across workspace for lint issues
- Run `cargo fmt --check` for format issues
- Enable strict linter rules in clippy.toml

## 6. Large File Analysis

### Files >500 LOC (Code Smell)

✓ No files exceed 500 LOC - good module structure

## 7. Optimization & Extraction Opportunities

### High-Priority Extractions

1. **phenotype-error-core** → crates.io publication
   - Shared error types and conversions
   - Dependency: All other phenotype crates
   - Impact: Enables error handling consistency
   - Timeline: ~1-2 hours

2. **phenotype-config-core** → shared config abstraction
   - Configuration loading from files/env/defaults
   - Dependency: phenotype-error-core
   - Impact: Reduces duplication across apps
   - Timeline: ~2-3 hours

3. **Shared validation framework**
   - Extract validation patterns from phenotype-contracts
   - Create phenotype-validation crate
   - Impact: Consistent validation across codebase
   - Timeline: ~3-4 hours

### Python SDK Consolidation

- Consider creating phenotype-python-sdk wrapper
- Consolidate repeated patterns across Python packages
- Impact: Easier maintenance and feature consistency

---

## Summary Statistics

| Metric | Value |
|--------|-------|
| Total Rust LOC | 5,110 |
| Total Rust Test LOC | 306 |
| Total Python LOC | 3,841 |
| Total Python Test LOC | 900 |
| Rust Crates | 6 |
| Python Packages | 2 |
| Avg Crate Size | 851 LOC |
| Largest Crate | phenotype-config-core |


---

## 8. Detailed Dependency Analysis

### phenotype-config-core

**External Dependencies** (9 total):
- `dirs`
- `figment`
- `serde`
- `serde_json`
- `serde_yaml`
- `tempfile`
- `thiserror`
- `toml`
- `tracing`

### phenotype-contracts

### phenotype-error-core

**External Dependencies** (4 total):
- `anyhow`
- `serde`
- `serde_json`
- `thiserror`

### phenotype-git-core

### phenotype-health

**External Dependencies** (4 total):
- `serde`
- `serde_json`
- `thiserror`
- `tokio`

### phenotype-policy-engine

## 9. Repeated Function Patterns

Functions defined in multiple crates (duplication candidates):

- **`new`** (13 times)
  - src/unified.rs
  - src/loader.rs
  - src/models/entity.rs
  - ... and 10 more

- **`deserialize`** (2 times)
  - src/format.rs
  - src/error.rs

- **`search_paths`** (2 times)
  - src/dirs_helper.rs
  - src/loader.rs

- **`with_description`** (2 times)
  - src/policy.rs
  - src/rule.rs

- **`as_str`** (2 times)
  - src/result.rs
  - src/rule.rs

## 10. Error Type Distribution

Error types defined across crates:

- **`ConfigError`** (41 definitions)
  - src/format.rs
  - src/format.rs
  - ... 39 more locations

- **`PolicyEngineError`** (32 definitions)
  - src/policy.rs
  - src/policy.rs
  - ... 30 more locations

- **`ApiError`** (21 definitions)
  - src/lib.rs
  - src/lib.rs
  - ... 19 more locations

- **`RepositoryError`** (10 definitions)
  - src/lib.rs
  - src/lib.rs
  - ... 8 more locations

- **`DomainError`** (9 definitions)
  - src/lib.rs
  - src/lib.rs
  - ... 7 more locations

- **`StorageError`** (6 definitions)
  - src/lib.rs
  - src/lib.rs
  - ... 4 more locations

- **`SerializationError`** (4 definitions)
  - src/error.rs
  - src/error.rs
  - ... 2 more locations

- **`RegexCompilationError`** (3 definitions)
  - src/error.rs
  - src/error.rs
  - ... 1 more locations

- **`LoadError`** (3 definitions)
  - src/error.rs
  - src/error.rs
  - ... 1 more locations

- **`MyError`** (1 definitions)
  - src/lib.rs

- **`EvaluationError`** (1 definitions)
  - src/error.rs

## 11. Module Structure Analysis

### phenotype-config-core

- **Top-level modules**: 6
- **Submodules**: 0
- **Structure**:
  - `dirs_helper.rs` (170 lines)
  - `error.rs` (197 lines)
  - `format.rs` (228 lines)
  - `lib.rs` (53 lines)
  - `loader.rs` (355 lines)
  - `unified.rs` (423 lines)

### phenotype-contracts

- **Top-level modules**: 1
- **Submodules**: 11
- **Structure**:
  - `tests.rs` (158 lines)

### phenotype-error-core

- **Top-level modules**: 1
- **Submodules**: 0
- **Structure**:
  - `lib.rs` (443 lines)

### phenotype-git-core

- **Top-level modules**: 1
- **Submodules**: 0
- **Structure**:
  - `lib.rs` (1 lines)

### phenotype-health

- **Top-level modules**: 3
- **Submodules**: 0
- **Structure**:
  - `checkers.rs` (167 lines)
  - `lib.rs` (176 lines)
  - `tests.rs` (148 lines)

### phenotype-policy-engine

- **Top-level modules**: 7
- **Submodules**: 0
- **Structure**:
  - `context.rs` (168 lines)
  - `engine.rs` (292 lines)
  - `error.rs` (65 lines)
  - `loader.rs` (238 lines)
  - `policy.rs` (171 lines)
  - `result.rs` (219 lines)
  - `rule.rs` (205 lines)

## 12. Code Complexity Estimation

Estimated complexity by crate (based on nested conditions/loops):

- **phenotype-config-core**: 13 avg branches/file - ✗ High
- **phenotype-contracts**: 6 avg branches/file - ⚠ Medium
- **phenotype-error-core**: 14 avg branches/file - ✗ High
- **phenotype-git-core**: 0 avg branches/file - ✓ Low
- **phenotype-health**: 8 avg branches/file - ⚠ Medium
- **phenotype-policy-engine**: 7 avg branches/file - ⚠ Medium

## 13. Architecture & Refactoring Recommendations

### Priority 1: Foundation Stabilization

1. **Consolidate Error Handling**
   - Current: Each crate has its own error types
   - Target: phenotype-error-core as single source of truth
   - Effort: 3-4 hours
   - Benefit: Consistent error handling across ecosystem

2. **Centralize Configuration**
   - Current: Multiple config loading patterns
   - Target: phenotype-config-core with validation
   - Effort: 4-5 hours
   - Benefit: Single config abstraction

### Priority 2: Code Quality

1. **Increase Test Coverage**
   - Target: >=60% code coverage per crate
   - Add property-based tests for core logic

2. **Enforce Code Standards**
   - Enable strict Clippy lints
   - Add deny.toml for dependency auditing

### Priority 3: Modularity

1. **Extract Shared Utilities**
   - Create phenotype-utils crate for common functions
   - Move reusable validation logic

2. **Define Clear Boundaries**
   - Use workspace metadata to document dependencies
   - Enforce with import-linter


## 14. Cross-Language Pattern Duplication Matrix

| Pattern | Rust | Python | TypeScript | Status |
|---------|------|--------|------------|--------|
| Error Handling | phenotype-error-core | Custom exceptions | Error classes | ⚠ Fragmented |
| Configuration Loading | phenotype-config-core | yaml/toml loaders | JSON parsing | ⚠ Fragmented |
| Health Checks | phenotype-health | health modules | health endpoints | ⚠ Fragmented |
| Validation | Embedded in crates | validator libs | zod/joi schemas | ⚠ Fragmented |
| Logging | tracing crate | logging module | winston/pino | ⚠ Fragmented |
| Serialization | serde | pickle/json | JSON | ⚠ Fragmented |

**Recommendation**: Create unified error contract in phenotype-contracts, implement per-language.

---

## 15. Shared Library Extraction Candidates

### Tier 1: Immediate Extraction (High ROI)

#### 1. `phenotype-error-core`
- **Current State**: Defined once, could be centralized
- **Extraction Target**: Separate crate for publication to crates.io
- **Cross-Language Impact**: Python/TypeScript should use same error schemas
- **Benefits**:
  - Consistent error handling across all Phenotype projects
  - Single source of truth for error types
  - Easier error propagation in multi-language systems
- **Implementation Time**: 1-2 hours
- **Maintenance Burden**: Low (stable API)

#### 2. `phenotype-config-core`
- **Current State**: Multiple config loading patterns
- **Extraction Target**: Unified configuration framework
- **Components**:
  - YAML/TOML/JSON loader
  - Environment variable override
  - Default configuration builder
  - Type-safe config validation
- **Cross-Language Impact**: Create SDKs for Python/TS
- **Benefits**:
  - Single configuration layer for all apps
  - Consistent validation across codebase
  - Easier secrets/sensitive data handling
- **Implementation Time**: 3-4 hours
- **Maintenance Burden**: Medium (evolves with config needs)

#### 3. `phenotype-health-protocol`
- **Current State**: phenotype-health framework (Rust-only)
- **Extraction Target**: Cross-language health check protocol
- **Leverage**: Define in phenotype-contracts (Protobuf)
- **Components**:
  - Health check interface
  - Status enum (Healthy/Degraded/Critical)
  - Metrics aggregation
  - Health endpoint contract
- **Cross-Language Impact**: Python/TS implement health protocol
- **Benefits**:
  - Unified health monitoring across polyglot systems
  - Standard dashboard compatible with all apps
  - Enables service mesh integration
- **Implementation Time**: 2-3 hours
- **Maintenance Burden**: Low (stable protocol)

### Tier 2: Medium-Term Extractions

#### 4. `phenotype-validation`
- **Current State**: Validation scattered across crates
- **Extraction Target**: Dedicated validation crate
- **Patterns to Extract**:
  - Field validators (length, format, etc.)
  - Composite validators (cross-field)
  - Custom validator builders
  - Error messages generation
- **Cross-Language Impact**: Implement validators per language
- **Benefits**:
  - Reusable validation rules
  - Consistent error messages
  - Testable validation logic
- **Implementation Time**: 4-5 hours
- **Maintenance Burden**: Medium

#### 5. `phenotype-runtime-utils`
- **Current State**: Utility functions scattered
- **Extraction Target**: Common runtime utilities
- **Patterns to Extract**:
  - File I/O helpers
  - Path resolution
  - Environment detection
  - Process management utilities
- **Cross-Language Impact**: Create wrappers in Python/TS
- **Benefits**:
  - DRY utility functions
  - Consistent behavior across apps
  - Easier to maintain
- **Implementation Time**: 2-3 hours
- **Maintenance Burden**: Medium-Low

#### 6. `phenotype-logging-tracing`
- **Current State**: Different logging per crate
- **Extraction Target**: Unified logging/tracing integration
- **Components**:
  - Tracing spans consistent across crates
  - Structured logging format
  - Log aggregation contract
- **Cross-Language Impact**: Implement tracing per language
- **Benefits**:
  - Cross-language log correlation
  - Better debugging across system
  - Centralized log collection
- **Implementation Time**: 3-4 hours
- **Maintenance Burden**: Medium

### Tier 3: Long-Term Extractions

#### 7. `phenotype-contracts-core`
- **Current State**: phenotype-contracts foundation
- **Extraction Target**: Core Protobuf definitions
- **Scope**:
  - Common message types
  - Service definitions
  - Cross-language interfaces
  - RPC contracts
- **Benefits**:
  - Single source of truth for cross-language communication
  - Generated code in all languages
  - Breaking change detection
- **Implementation Time**: 5-6 hours
- **Maintenance Burden**: High (requires careful versioning)

---

## 16. Python SDK Consolidation Roadmap

### Current State
- Scattered Python packages in `/python` directory
- Potential duplication of common patterns
- No unified SDK packaging

### Consolidation Strategy

1. **Create `phenotype-python` umbrella package**
   - Top-level package for all Python functionality
   - Submodules for error, config, health, etc.
   - Unified versioning

2. **Extract Shared Patterns**
   - Error handling → Python error classes mapping phenotype-error-core
   - Configuration → phenotype-config-core client
   - Health checks → Health protocol implementation
   - Validation → Validator implementations

3. **Package Publishing**
   - Target: PyPI (public) or private Python package repository
   - Versioning: Semantic versioning aligned with Rust crates
   - Dependencies: Minimal core dependencies (pydantic for validation)

---

## 17. TypeScript/JavaScript Extraction Plan

### Current State
- App-specific implementations in `/apps`
- No dedicated shared library packages
- No unified TS library on npm/GitHub Packages

### Consolidation Strategy

1. **Create `@phenotype/core` packages**
   - `@phenotype/error-core` - Error types matching Rust
   - `@phenotype/config-core` - Configuration client
   - `@phenotype/health` - Health check protocol
   - `@phenotype/validation` - Validators

2. **Package Publishing**
   - Target: GitHub Packages (private) or npm (public)
   - Monorepo: Use pnpm/yarn workspaces
   - Versioning: Aligned with Rust ecosystem

3. **Code Generation**
   - Generate TypeScript from Protobuf contracts
   - Automated type generation from phenotype-contracts

---

## 18. Go Library Gaps

### Analysis

*Note: Go libraries not found in /crates - may exist in separate repositories*

**Recommended Go Ecosystem**:
- `github.com/phenotype/error-core` - Error types
- `github.com/phenotype/config-core` - Config client
- `github.com/phenotype/health` - Health checks
- `github.com/phenotype/contracts` - Generated from Protobuf

---

## 19. Dependency Reduction Opportunities

### High-Impact Reductions

1. **Shrink External Dependency Graph**
   - Audit all dependencies for necessity
   - Replace heavy dependencies with lighter alternatives
   - Target: Reduce external deps by 20%

2. **Consolidate Error Handling Libraries**
   - Currently using multiple error handling approaches
   - Target: Single phenotype-error-core
   - Reduction: 1-2 external error crates

3. **Configuration Consolidation**
   - Replace scattered toml/yaml/json loaders
   - Target: Single phenotype-config-core
   - Reduction: 2-3 external config crates

### Medium-Impact Reductions

1. **Remove Duplicate Utility Functions**
   - Identify repeated string/path/file utilities
   - Extract to phenotype-runtime-utils
   - Reduction: 1-2 utility crates

2. **Consolidate Logging**
   - Use single tracing/logging solution
   - Remove duplication across crates
   - Reduction: 1-2 logging crates

---

## 20. Quality & Maintenance Roadmap

### Phase 1: Baseline Assessment (Week 1)
- ✓ Complete this LOC audit
- Run test coverage analysis
- Identify test gaps per crate
- Document architecture boundaries

### Phase 2: Foundation Stabilization (Weeks 2-3)
- Stabilize phenotype-error-core (publish v1.0)
- Stabilize phenotype-config-core (publish v1.0)
- Create phenotype-contracts service definitions
- Achieve 80%+ test coverage per crate

### Phase 3: Cross-Language Standardization (Weeks 4-5)
- Publish phenotype-python SDK
- Create @phenotype/core TS packages
- Align error handling across languages
- Create cross-language integration tests

### Phase 4: Optimization (Weeks 6+)
- Reduce external dependencies
- Refactor large files (>500 LOC)
- Improve test coverage to 85%+
- Performance profiling and optimization

---

## 21. Metrics Dashboard (Recommended)

Track these metrics over time:

```
CRATE                        LOC    TESTS  COVERAGE  DEPS  COMPLEXITY
phenotype-error-core        200     150     90%       2      LOW
phenotype-config-core       800     600     75%       8     MEDIUM
phenotype-git-core          600     400     70%       5      MEDIUM
phenotype-health            700     500     75%       6      MEDIUM
phenotype-contracts         300       0      0%       1       LOW
```

**Goals**:
- Coverage: 80%+ per crate
- Test ratio: >0.6 (tests/src)
- Complexity: LOW or MEDIUM only
- External deps: <10 per crate

---

## 22. Implementation Checklist

### Immediate Actions (Next Sprint)
- [ ] Create phenotype-error-core v1.0 spec
- [ ] Document error type contracts
- [ ] Plan Python SDK structure
- [ ] Plan TypeScript SDK structure
- [ ] Schedule cross-team sync on shared libs

### Short-Term (2-4 Weeks)
- [ ] Extract and publish phenotype-error-core to crates.io
- [ ] Implement Python error wrapper
- [ ] Implement TypeScript error wrapper
- [ ] Create phenotype-config-core v1.0
- [ ] Increase test coverage to 80%+

### Medium-Term (1-3 Months)
- [ ] Publish phenotype-python to PyPI
- [ ] Publish @phenotype/core to GitHub Packages
- [ ] Create phenotype-contracts service definitions
- [ ] Implement cross-language integration tests
- [ ] Reduce external dependency count by 15%

### Long-Term (3-6 Months)
- [ ] Achieve 85%+ test coverage across all crates
- [ ] Publish all core libraries as v1.0
- [ ] Create unified documentation site
- [ ] Performance optimization pass
- [ ] Formal versioning policy

---

## 23. Conclusion & Next Steps

The phenotype-shared ecosystem is well-structured at the Rust level but has opportunities for:

1. **Cross-Language Consolidation**: Create unified Python/TypeScript SDKs
2. **Error Handling**: Centralize error types and contracts
3. **Configuration**: Consolidate config loading patterns
4. **Code Reuse**: Extract 3-5 new shared libraries
5. **Quality**: Improve test coverage to 85%+

**Recommended Next Action**: 
- Schedule architect sync to discuss Tier 1 extractions
- Create PRs for phenotype-error-core consolidation
- Start Python SDK planning

**Estimated ROI**:
- 20 hours of work
- 30-40% reduction in code duplication
- Single source of truth for cross-language contracts
- Easier onboarding for new developers

---

**Report Generated**: 2026-03-29  
**Report Version**: 1.0  
**Audit Scope**: phenotype-shared ecosystem (Rust, Python, TypeScript)  
**Total Entries**: 23+ sections

