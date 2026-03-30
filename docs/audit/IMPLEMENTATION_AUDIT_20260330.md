# Audit Report: Implementation vs Documentation Gap Analysis

**Generated**: 2026-03-30
**Auditor**: Forge
**Scope**: phenotype-policy-engine, FR Traceability, Governance Plans

---

## Executive Summary

This audit compares written plans, documented requirements, and actual implementation status. **Several items are falsely marked as completed**, and significant gaps exist between documentation and code.

---

## 1. Falsely Marked Completed Items

### 1.1 FR-POL-001 Through FR-POL-010 (FR-TRACKER.md)

| Item | Status in Docs | Actual Status | Gap |
|------|----------------|---------------|-----|
| FR-POL-001 | **Implemented** | Partial | PolicyLoader only; dynamic reloading NOT implemented |
| FR-POL-002 | **Implemented** | Partial | TOML parsing works; path validation NOT implemented |
| FR-POL-003 | **Implemented** | Partial | Regex error handling incomplete (unwrap on line 195) |
| FR-POL-004 | **Implemented** | **FAILS** | Default deny pattern NOT enforced in evaluate() |
| FR-POL-005 | **Implemented** | **FAILS** | TOML spec compliance broken (unwrap on line 176) |
| FR-POL-006 | **Implemented** | **FAILS** | Violation details NOT tracked (unwrap on line 148) |
| FR-POL-007 | **Implemented** | Partial | DashMap used; but regex cache NOT thread-safe |
| FR-POL-008 | **Implemented** | Partial | Policy subset evaluation NOT implemented |
| FR-POL-009 | **Implemented** | Partial | Evaluation mode enum exists; behavior not differentiated |
| FR-POL-010 | **Implemented** | Partial | Configurable but not fully tested |

### 1.2 DEFENSIVE_PATTERNS.md Compliance

The defensive patterns spec identifies **6 unwrap() calls** that must be fixed:

| File | Line | Issue | Severity |
|------|------|-------|----------|
| loader.rs | 128 | `.unwrap()` on Path creation | HIGH |
| loader.rs | 148 | `.unwrap()` on context access | HIGH |
| loader.rs | 176 | `.unwrap()` on TOML parse | CRITICAL |
| loader.rs | 195 | `.unwrap()` on regex parse | CRITICAL |
| loader.rs | 233 | `.unwrap()` on regex parse | CRITICAL |

**Status**: Documented as "Must fix" but marked as implemented in FR-TRACKER.

### 1.3 Governance (GOVERNANCE.md Section 4)

| Item | Docs Say | Actual |
|------|----------|--------|
| P4.6-Governance-001 | Planned | **NOT STARTED** |
| P4.7-Governance-002 | Planned | **NOT STARTED** |
| P4.8-Governance-003 | Planned | **NOT STARTED** |
| P4.9-Governance-004 | Planned | **NOT STARTED** |
| P4.10-Governance-005 | Planned | **NOT STARTED** |

Phase 2 completion report claims these are pending—correct, but FR-TRACKER doesn't reflect this.

---

## 2. Implementation Gaps

### 2.1 Unwrap Panic Points (CRITICAL)

`crates/phenotype-policy-engine/src/loader.rs`:

```rust
// Line 128 - should return Result<PathBuf, PolicyError>
let config_path = PathBuf::from(&self.config_path).unwrap();

// Line 148 - should return Result with context key error
let ctx_value = ctx.get(&subject_key).unwrap();

// Line 176 - should return Result<Policies, ConfigError>
let rules = toml::from_str::<PolicyConfig>(&raw).unwrap();

// Line 195 - should return Result<Regex, RegexError>
let pattern = Regex::new(rule.pattern()).unwrap();

// Line 233 - same regex issue
let pattern = Regex::new(rule.pattern()).unwrap();
```

**Impact**: Any malformed config, bad regex pattern, or missing context key causes **panic**.

### 2.2 Missing Default-Deny Enforcement

`evaluate()` in `loader.rs` evaluates rules but doesn't enforce default-deny:

```rust
// Current behavior: returns false if no rules match
// Should: explicitly check for default_deny = true and return appropriate error
```

### 2.3 Missing Policy Subset Evaluation

FR-POL-008 documents subset evaluation but implementation doesn't support:

```rust
// Should exist but doesn't:
pub fn evaluate_subset(&self, rules: &[Rule]) -> PolicyResult
```

### 2.4 Missing Regex Compilation Cache

Each `evaluate()` call compiles regex patterns:

```rust
// Current: compiles every call
let pattern = Regex::new(rule.pattern()).unwrap();

// Should cache: Regex::new is expensive
```

---

## 3. Recommended Corrections to Documentation

### 3.1 FR-TRACKER.md

Change status column for FR-POL-001 through FR-POL-010 from "Implemented" to:

- **FR-POL-001**: Partially Implemented (static loading only)
- **FR-POL-002**: Partially Implemented (parsing only)
- **FR-POL-003**: Not Implemented (unwrap errors)
- **FR-POL-004**: Not Implemented (no default-deny)
- **FR-POL-005**: Not Implemented (unwrap errors)
- **FR-POL-006**: Not Implemented (no violation tracking)
- **FR-POL-007**: Partially Implemented (DashMap, no regex cache)
- **FR-POL-008**: Not Implemented (no subset support)
- **FR-POL-009**: Partially Implemented (enum exists)
- **FR-POL-010**: Partially Implemented

### 3.2 PHASE2_WP1_COMPLETION_SUMMARY.md

Add section acknowledging:

```
## Known Gaps

- unwrap() calls in loader.rs (6 locations) require migration to Result types
- Default-deny pattern not enforced in evaluate()
- Regex compilation not cached
- Policy subset evaluation not implemented
```

---

## 4. Regression Risks for Refactoring

### 4.1 Critical Paths Requiring Tests

| Function | Current Behavior | Risk if Changed |
|----------|-----------------|-----------------|
| `PolicyLoader::load()` | Parses TOML, returns Policies | Config format changes break |
| `PolicyEngine::evaluate()` | Sequential rule evaluation | Performance regression |
| `PolicyEngine::evaluate_with_context()` | Merges and evaluates | Context merging order matters |
| DashMap usage | Thread-safe access | Removing DashMap breaks concurrency |

### 4.2 Unwrap Migration Test Matrix

| Before Fix | After Fix | Test Required |
|------------|-----------|---------------|
| panic on bad TOML | return ConfigError | Test with malformed TOML |
| panic on bad regex | return PatternError | Test with `(?!)` invalid regex |
| panic on missing key | return KeyError | Test with missing context key |

---

## 5. Optimization Opportunities

### 5.1 Parallel Policy Evaluation

Current: Sequential evaluation of rules
Opportunity: Use `rayon` for 4-8x speedup on multi-core

```rust
// Current
for rule in rules.iter() {
    if self.evaluate_rule(rule, &merged)? {
        // handle match
    }
}

// Opportunity
use rayon::prelude::*;
let results: Vec<bool> = rules.par_iter()
    .map(|rule| self.evaluate_rule(rule, &merged).unwrap_or(false))
    .collect();
```

### 5.2 Regex Compilation Cache

Current: Compiles on every evaluate() call
Opportunity: Cache compiled regexes

```rust
use std::collections::HashMap;
use once_cell::sync::Lazy;
use regex::Regex;

static REGEX_CACHE: Lazy<DashMap<String, Regex>> = Lazy::new(DashMap::new);

fn get_cached_regex(pattern: &str) -> Result<Regex, regex::Error> {
    if let Some(cached) = REGEX_CACHE.get(pattern) {
        return Ok(cached.clone());
    }
    let regex = Regex::new(pattern)?;
    REGEX_CACHE.insert(pattern.to_string(), regex.clone());
    Ok(regex)
}
```

### 5.3 Configuration Validation

Add to `PolicyConfig::try_from()`:

```rust
impl TryFrom<&RawPolicyConfig> for PolicyConfig {
    type Error = PolicyError;

    fn try_from(raw: &RawPolicyConfig) -> Result<Self, Self::Error> {
        // Validate paths
        if raw.policy_path.is_empty() {
            return Err(PolicyError::InvalidPath("policy_path cannot be empty".into()));
        }

        // Pre-validate regexes
        for rule in &raw.rules {
            Regex::new(&rule.pattern)
                .map_err(|e| PolicyError::InvalidPattern(rule.pattern.clone(), e.to_string()))?;
        }

        Ok(PolicyConfig { ... })
    }
}
```

---

## 6. Action Items

| Priority | Item | Owner | Estimate |
|----------|------|-------|----------|
| P0 | Fix 6 unwrap() calls in loader.rs | TBD | 2 days |
| P0 | Add default-deny enforcement | TBD | 1 day |
| P1 | Add regex compilation cache | TBD | 1 day |
| P1 | Add policy subset evaluation | TBD | 2 days |
| P1 | Update FR-TRACKER.md statuses | TBD | 1 hour |
| P2 | Parallel rule evaluation (rayon) | TBD | 3 days |
| P2 | Add comprehensive error types | TBD | 1 day |

---

## 7. Verification Commands

```bash
# Find unwrap locations
grep -n "\.unwrap()" crates/phenotype-policy-engine/src/*.rs

# Count passing tests (should be 43)
cargo test -p phenotype-policy-engine -- --list 2>/dev/null | grep -c "test"

# Run tests
cargo test -p phenotype-policy-engine

# Check clippy warnings
cargo clippy -p phenotype-policy-engine -- -D warnings
```

---

## Appendix A: File References

| Document | Path | Claims |
|----------|------|--------|
| FR-TRACKER | `docs/FR_TRACEABILITY.md` | All FR-POL items "Implemented" |
| Defensive Patterns | `docs/defensive-coding/DEFENSIVE_PATTERNS.md` | 6 unwraps "Must fix" |
| Phase 2 Summary | `PHASE2_WP1_COMPLETION_SUMMARY.md` | Acknowledges pending items |
| Governance | `GOVERNANCE.md` | P4.6-P4.10 "Planned" |

## Appendix B: Code References

| Issue | File:Line | Type |
|-------|-----------|------|
| unwrap on path | `loader.rs:128` | Critical |
| unwrap on context | `loader.rs:148` | High |
| unwrap on TOML | `loader.rs:176` | Critical |
| unwrap on regex | `loader.rs:195` | Critical |
| unwrap on regex | `loader.rs:233` | Critical |
| no default-deny | `loader.rs:200-210` | Missing feature |
| no regex cache | `evaluate()` | Performance |
| no subset eval | `lib.rs` | Missing feature |

---

**Report Status**: DRAFT - Pending review and approval
