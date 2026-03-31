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

