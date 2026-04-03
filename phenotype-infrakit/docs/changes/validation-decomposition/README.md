# Validation System Decomposition Project

**Status**: Design + Specification Complete, Ready for Implementation
**Target**: Reduce CLI `validate.rs` from 674 LOC → <100 LOC via registry-based system
**Expected Impact**: 50% LOC reduction, 100% extensibility improvement, 0% duplication

---

## Quick Navigation

### For Executives & Architects
Start here: **[VALIDATION_DECOMPOSITION_SUMMARY.md](./VALIDATION_DECOMPOSITION_SUMMARY.md)**
- Executive overview with metrics and impact analysis
- Before/after code comparisons
- Timeline, risks, and success criteria
- 10 min read

### For Implementers
1. **[PHASED_MIGRATION_PLAN.md](./PHASED_MIGRATION_PLAN.md)** (30 min read)
   - 3 atomic commits with work items (WI-1.1 through WI-3.3)
   - Effort estimates: 3-4h (Phase 1), 4-5h (Phase 2), 2-3h (Phase 3)
   - Success criteria, rollback plans, parallel/sequential paths

2. **Source Code** (in `/crates/phenotype-validation/src/`)
   - `traits/mod.rs` — Trait exports
   - `traits/error.rs` — Error types with context (180 LOC)
   - `traits/rule.rs` — Core rules & implementations (400 LOC)
   - `traits/field_validator.rs` — Field-level validator (190 LOC)
   - `traits/command_validator.rs` — Command-level orchestrator (280 LOC)
   - `registry.rs` — Plugin registry pattern (120 LOC)
   - `presets.rs` — 9 preset validators (80 LOC)
   - `lib.rs` — Updated public API (50 LOC)

### For CLI Engineers (Post-Phase 3)
Start here: **[MIGRATION_GUIDE.md](./MIGRATION_GUIDE.md)**
- Step-by-step CLI migration instructions
- Before/after patterns for each command
- Common validation patterns (optional, dependent, conditional)
- Troubleshooting & rollback

### For Learning & Examples
See: **[../../../docs/guides/VALIDATION_EXAMPLES.md](../../../docs/guides/VALIDATION_EXAMPLES.md)**
- 10 working code examples (basic to advanced)
- Plugin registration & custom rules
- Testing strategies
- Async validation planning

### For Architecture Context
See: **[../../../docs/guides/VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md](../../../docs/guides/VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md)**
- Complete architecture with diagrams
- Trait hierarchy breakdown
- Layer-by-layer design
- Extensibility patterns

---

## Key Artifacts

| File | Purpose | Audience | Length |
|------|---------|----------|--------|
| **VALIDATION_DECOMPOSITION_SUMMARY.md** | Executive overview | Leaders, architects | 4 pages |
| **PHASED_MIGRATION_PLAN.md** | Detailed implementation plan | Developers | 12 pages |
| **MIGRATION_GUIDE.md** | CLI adoption guide | CLI engineers | 10 pages |
| **../VALIDATION_EXAMPLES.md** | Code examples & patterns | All engineers | 15 pages |
| **../VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md** | Architecture design | Architects, developers | 12 pages |
| **Source code** | Working implementation | Developers | 920 LOC |

---

## 3-Phase Implementation

### Phase 1: Trait Foundation (3-4 hours)
**Deliverable**: Core traits + 5 rules + 55+ tests

```bash
git branch validation/phase-1-traits
# Create:
#   crates/phenotype-validation/src/traits/mod.rs (40 LOC)
#   crates/phenotype-validation/src/traits/error.rs (180 LOC)
#   crates/phenotype-validation/src/traits/rule.rs (400 LOC)
#   crates/phenotype-validation/src/traits/field_validator.rs (190 LOC)
#   crates/phenotype-validation/src/traits/command_validator.rs (280 LOC)
cargo test --lib traits::
git commit -m "feat(validation): implement trait hierarchy and core rules"
```

### Phase 2: Registry & Presets (4-5 hours)
**Depends on**: Phase 1
**Deliverable**: Registry + 9 validators + 28+ tests

```bash
git branch validation/phase-2-registry
# Create:
#   crates/phenotype-validation/src/registry.rs (120 LOC)
# Update:
#   crates/phenotype-validation/src/presets.rs (80 LOC)
#   crates/phenotype-validation/src/lib.rs (50 LOC)
cargo test --lib
git commit -m "feat(validation): implement registry pattern and preset validators"
```

### Phase 3: Documentation & Examples (2-3 hours)
**Depends on**: Phase 2
**Deliverable**: Docs + 5 examples

```bash
git branch validation/phase-3-docs
# Already created:
#   docs/changes/validation-decomposition/MIGRATION_GUIDE.md
#   docs/changes/validation-decomposition/PHASED_MIGRATION_PLAN.md
#   docs/guides/VALIDATION_EXAMPLES.md
# Just need:
#   crates/phenotype-validation/examples/*.rs (5 files)
cargo build --examples
git commit -m "docs(validation): add migration guide and examples"
```

---

## Success Metrics

By end of Phase 3:

- [ ] All 3 atomic commits merged
- [ ] 80+ unit tests passing
- [ ] 0 compiler warnings
- [ ] 85%+ test coverage
- [ ] All docs reviewed & approved
- [ ] All examples compile

By end of CLI integration (separate task):

- [ ] validate.rs reduced 674 → <100 LOC (80%+ reduction)
- [ ] 3+ custom validators registered
- [ ] All CLI commands using registry-based validation
- [ ] Error messages include field paths & suggestions

---

## Architecture Overview

```
Validation System Layers
├── Layer 1: Traits (abstractions)
│   ├── ValidationRule (core)
│   ├── FieldValidator (multi-rule)
│   ├── CommandValidator (orchestrator)
│   └── ValidationError (rich context)
├── Layer 2: Rules (implementations)
│   ├── RequiredRule
│   ├── LengthRule
│   ├── PatternRule
│   ├── NumericRangeRule
│   └── CustomRule
├── Layer 3: Registry (discovery)
│   └── ValidatorRegistry (plugin pattern)
├── Layer 4: Presets (convenience)
│   ├── email_validator
│   ├── url_validator
│   ├── username_validator
│   └── ... (9 total)
└── Layer 5: CLI Integration (user-facing)
    └── Per-command validators (registry-based)
```

---

## Key Design Decisions

| Decision | Rationale | Benefit | Tradeoff |
|----------|-----------|---------|----------|
| **Rule Composition** | Traits over inheritance | Flexible, testable | More configuration |
| **Registry as Singleton** | Plugin discovery | Extensible | Thread-safety needed |
| **Separate Async** | Keep sync fast | Zero-Copy baseline | Async is opt-in |
| **Rich Errors** | User-friendly | Clear messages | More complex type |

---

## Extensibility Example

```rust
// 1. Define custom rule
#[derive(Clone)]
pub struct WorkflowStatusRule { allowed: Vec<String> }

impl ValidationRule for WorkflowStatusRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        if self.allowed.contains(&value.to_string()) {
            Ok(())
        } else {
            Err(ValidationError::new("invalid_status")
                .with_suggestion(format!("Must be: {}", self.allowed.join(", "))))
        }
    }
    fn name(&self) -> &'static str { "workflow_status" }
}

// 2. Register at startup (one-time)
ValidatorRegistry::register("workflow_status", || {
    FieldValidator::new()
        .with_rule(RequiredRule::new())
        .with_rule(WorkflowStatusRule::new(vec![...]))
});

// 3. Use anywhere (plugin automatically discovered)
let validator = ValidatorRegistry::get("workflow_status")?;
validator.validate("active")?;
```

---

## Timeline

| Phase | Duration | Effort | Owner | Status |
|-------|----------|--------|-------|--------|
| Phase 1 | 3-4h | 1 agent | TBD | Ready |
| Phase 2 | 4-5h | 1 agent | TBD | Ready (depends Phase 1) |
| Phase 3 | 2-3h | 1 agent | TBD | Ready (depends Phase 2) |
| **Total (parallel)** | **~4h** | **3 agents** | TBD | **Ready** |
| **Total (sequential)** | **~12h** | **1 agent** | TBD | **Ready** |

---

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|-----------|
| API instability | Low | Medium | Freeze API after Phase 1 |
| Registry contention | Low | Low | Lazy<Mutex> thread-safety |
| CLI integration overrun | Medium | Medium | Phase 3 is docs only |
| Backward compat | Low | Low | Legacy bridge maintained |

---

## References

- **Architecture**: `../VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md`
- **Examples**: `../../../docs/guides/VALIDATION_EXAMPLES.md`
- **CLI Guide**: `./MIGRATION_GUIDE.md`
- **Plan**: `./PHASED_MIGRATION_PLAN.md`
- **Summary**: `./VALIDATION_DECOMPOSITION_SUMMARY.md`

---

## Questions?

- **Architecture**: See VALIDATION_DECOMPOSITION_AND_REGISTRY_PATTERN.md
- **Implementation**: See PHASED_MIGRATION_PLAN.md
- **Usage**: See VALIDATION_EXAMPLES.md
- **CLI Migration**: See MIGRATION_GUIDE.md
- **Status**: See VALIDATION_DECOMPOSITION_SUMMARY.md

---

**Status**: Ready for implementation
**Next Step**: Assign Phase 1 to developer/agent

