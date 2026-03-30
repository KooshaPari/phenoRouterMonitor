# LOC Reduction Opportunities - phenotype-infrakit

## Summary

This document captures LOC reduction opportunities identified during the Wave 97 audit.

## LOC Reduction Patterns Found

### 1. Manual `impl Default` (4 instances)

| Crate | File | LOC | Action |
|-------|------|-----|--------|
| phenotype-event-sourcing | snapshot.rs:14 | 7 | Convert to `#[derive(Default)]` |
| phenotype-test-infra | lib.rs:183 | 5 | Convert to `#[derive(Default)]` |
| phenotype-retry | builder.rs:133 | 4 | Keep (uses constants) |
| agileplus-api-types | lib.rs:18 | 5 | Convert to `#[derive(Default)]` |

### 2. Verbose Error Handling

| Pattern | Count | LOC Savings |
|---------|-------|------------|
| `.map_err(\|e\| ...)` | 833 | ~1,666 |
| `.unwrap()` | 2,984 | Variable |
| `.expect()` | 5,358 | Variable |

### 3. Clone Opportunities

| Pattern | Count | Potential Savings |
|---------|-------|-------------------|
| `.clone()` calls | 6,422 | Use `Arc`, `Rc`, `Cow` |

### 4. Display Derives for Enums

| Crate | Enums | LOC Savings |
|-------|-------|------------|
| phenotype-policy-engine | 4 enums | ~40 LOC |
| phenotype-event-sourcing | 3 enums | ~30 LOC |

## Implementation Notes

### Using `derive_more`

With `derive_more = "1.0"` already in workspace dependencies:

```rust
// Before
#[derive(Clone, Debug)]
pub struct Config {
    pub threshold: u32,
}

impl Default for Config {
    fn default() -> Self {
        Self { threshold: 100 }
    }
}

// After (requires nightly for field defaults)
#[derive(Clone, Debug, Default)]
pub struct Config {
    #[default = 100]
    pub threshold: u32,
}
```

### Using `strum` for Enums

With `strum = { version = "0.26", features = ["derive"] }` already in workspace:

```rust
// Before
#[derive(Clone, Debug)]
pub enum Status {
    Active,
    Inactive,
    Pending,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Active => write!(f, "active"),
            Status::Inactive => write!(f, "inactive"),
            Status::Pending => write!(f, "pending"),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "active" => Ok(Status::Active),
            "inactive" => Ok(Status::Inactive),
            "pending" => Ok(Status::Pending),
            _ => Err(format!("unknown status: {}", s)),
        }
    }
}

// After
use strum::{Display, EnumString};

#[derive(Clone, Debug, Display, EnumString)]
pub enum Status {
    #[strum(serialize = "active")]
    Active,
    #[strum(serialize = "inactive")]
    Inactive,
    #[strum(serialize = "pending")]
    Pending,
}
```

## Total LOC Savings Potential

| Category | Identified | Achievable | Priority |
|----------|------------|------------|----------|
| Manual Default impls | ~25 LOC | ~20 LOC | 🟡 MEDIUM |
| Enum Display/FromStr | ~200 LOC | ~150 LOC | 🟡 MEDIUM |
| Error handling | ~8,000 LOC | ~3,000 LOC | 🟠 HIGH |
| Clone optimization | N/A | ~500 LOC | 🟡 MEDIUM |
| **TOTAL** | **~8,200** | **~3,670** | |

## Prerequisites

Before implementing these changes:

1. Fix pre-existing compilation errors in phenotype-event-sourcing
2. Add `derive_more` and `strum` to individual crate dependencies (or use workspace)

## Related Work

- PR #94: Added `derive_more` and `strum` to workspace dependencies
- docs/worklogs/DUPLICATION.md: Full LOC audit findings
- docs/worklogs/RESEARCH.md: 2026 Rust ecosystem analysis
