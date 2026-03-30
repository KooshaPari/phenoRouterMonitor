# Validation System Decomposition and Registry Pattern

**Status**: Design Phase
**Target**: Convert field-level validators into extensible, plugin-based system
**Scope**: phenotype-validation crate refactor
**Expected Impact**: 50% LOC reduction, 100% extensibility improvement

---

## Current State Analysis

### Existing Structure
The `phenotype-validation` crate currently has:
- **validators.rs** (165 LOC): Basic utility functions for email, URL, UUID validation
- **presets.rs** (130 LOC): Validator preset builders referencing undefined modules
- **lib.rs** (15 LOC): Error types and basic result wrapper

**Pain Points**:
1. Presets reference non-existent `rules` and `validator` modules
2. No trait-based abstraction for extensibility
3. Validation rules are hardcoded, not pluggable
4. No registry pattern for discovering validators
5. Error types are minimal (generic `Invalid(String)`)

---

## Proposed Architecture

### Layer 1: Core Traits and Abstractions

```
┌─────────────────────────────────────────────────────────────┐
│                    Trait Hierarchy                           │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ ValidationRule trait (base)                          │   │
│  │  - fn validate(&self, value: &str) -> Result<()>    │   │
│  │  - fn name(&self) -> &'static str                    │   │
│  │  - fn severity(&self) -> Severity                    │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ▲                                    │
│                          │                                    │
│        ┌─────────────────┼─────────────────┬────────────┐   │
│        │                 │                 │            │   │
│   PatternRule   LengthRule  ComparisonRule ...          │   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ FieldValidator trait                                 │   │
│  │  - fn validate(&self, value: &str) -> Result<()>    │   │
│  │  - fn add_rule(&mut self, rule: Box<dyn Rule>)      │   │
│  │  - fn rules(&self) -> &[Box<dyn Rule>]              │   │
│  └──────────────────────────────────────────────────────┘   │
│                          ▲                                    │
│                          │                                    │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ CommandValidator trait                               │   │
│  │  - fn validate(&self, cmd: &Command) -> Result<()>  │   │
│  │  - fn register_field_validator(...)                  │   │
│  │  - fn field_validators(&self) -> &HashMap<..>       │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
│  ┌──────────────────────────────────────────────────────┐   │
│  │ ValidatorRegistry (singleton)                        │   │
│  │  - fn register(name: &str, factory: ValidatorFn)    │   │
│  │  - fn get(name: &str) -> Option<&ValidatorFn>       │   │
│  │  - fn list() -> Vec<&str>                           │   │
│  └──────────────────────────────────────────────────────┘   │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Layer 2: Built-in Rules

```
Core Rules:
├── RequiredRule          (checks non-empty)
├── PatternRule           (regex-based matching)
├── LengthRule
│   ├── MinLengthRule
│   ├── MaxLengthRule
│   └── ExactLengthRule
├── ComparisonRule
│   ├── NumericMinRule
│   ├── NumericMaxRule
│   └── NumericRangeRule
├── FormatRule
│   ├── EmailRule
│   ├── UrlRule
│   ├── UuidRule
│   └── IpAddressRule
└── CustomRule            (user-defined closures)

Plugin Rules (extensible):
├── DatabaseUniqueRule    (async validation against DB)
├── ExternalApiRule       (async validation against API)
└── AsyncCustomRule       (user-defined async closures)
```

### Layer 3: Field Validators (Presets)

Predefined, composable validators for common patterns:

```
Validators:
├── EmailValidator        -> RequiredRule + EmailRule
├── UrlValidator          -> RequiredRule + UrlRule
├── UsernameValidator     -> RequiredRule + PatternRule + LengthRule
├── PasswordValidator
│   ├── strong()          -> Required + MinLength(12) + Pattern(special chars)
│   ├── moderate()        -> Required + MinLength(8) + Pattern
│   └── basic()           -> Required + MinLength(6)
├── PhoneValidator        -> RequiredRule + PatternRule
└── SlugValidator         -> RequiredRule + PatternRule + LengthRule
```

### Layer 4: Command Validators (Orchestrators)

Composite validators for CLI commands:

```
Example: CreatePlanCommand validation
├── Field: name
│   └── UsernameValidator (3-50 chars, alphanumeric)
├── Field: description
│   ├── RequiredRule
│   ├── MinLengthRule(10)
│   └── MaxLengthRule(500)
├── Field: dueDate
│   ├── RequiredRule
│   └── CustomRule(is_future_date)
├── Cross-field: dates
│   └── CustomRule(start_date < end_date)
└── Async: uniqueness
    └── DatabaseUniqueRule(table="plans", column="name")
```

---

## Decomposition Strategy (3 Phases)

### Phase 1: Trait Foundation (Atomic Commit)
**Files**: 
- Create: `src/traits/mod.rs`, `src/traits/rule.rs`, `src/traits/validator.rs`
- Create: `src/rules/mod.rs`, `src/rules/core.rs`

**Deliverables**:
- `ValidationRule` trait with default implementations
- `FieldValidator` trait with builder methods
- `CommandValidator` trait for command-level orchestration
- `ValidationError` enum with rich context
- Core required/pattern rules

**Tests**: Unit tests for each trait

### Phase 2: Registry and Presets (Atomic Commit)
**Files**:
- Create: `src/registry/mod.rs`
- Refactor: `src/presets.rs` to use new trait system
- Create: `src/builders.rs` for preset fluent builders

**Deliverables**:
- `ValidatorRegistry` singleton with registration/lookup
- Plugin-based registration hooks
- Email/URL/Username/Password preset validators
- Fluent builder API for custom validators
- Registry integration tests

**Tests**: Integration tests for registry plugins

### Phase 3: Migration and Documentation (Atomic Commit)
**Files**:
- Create: `examples/custom_validator.rs`, `examples/command_validator.rs`
- Update: `lib.rs` to re-export new structures
- Create: `docs/VALIDATION_GUIDE.md` (user-facing)

**Deliverables**:
- Example: Custom regex-based field validator
- Example: Command-level validation orchestrator
- Example: Async database uniqueness validator
- Migration guide for existing code
- Architecture documentation

---

## Design Decisions

### 1. Rule Composition Over Inheritance
**Decision**: Use trait-based composition with `ValidationRule` trait
**Rationale**:
- Allows mixing built-in and custom rules without subclassing
- Each rule is independently testable
- Rules can be nested (e.g., `Or`, `And` combinator rules)

### 2. Registry as Global Singleton
**Decision**: Lazy-static registry with registration hooks
**Rationale**:
- Enables plugin discovery at runtime
- Allows 3rd-party crates to register custom validators
- Can be mocked/swapped in tests

### 3. Async Validation Support
**Decision**: Separate `AsyncValidationRule` trait
**Rationale**:
- Sync rules remain fast and Zero-Copy
- Async rules opt-in for features like DB checks
- Enables future executor abstraction (tokio, async-std, etc.)

### 4. Error Context Hierarchy
**Decision**: Rich `ValidationError` with context stack
**Rationale**:
- Track validation failure at field + rule level
- Enable CLI to display user-friendly messages with paths
- Support i18n through error keys

---

## Plugin Extensibility Example

### User-Defined Plugin

```rust
// File: my_validators.rs
use phenotype_validation::{ValidationRule, ValidationError, Severity};

pub struct CustomDomainRule {
    allowed_domains: Vec<String>,
}

impl ValidationRule for CustomDomainRule {
    fn validate(&self, value: &str) -> Result<(), ValidationError> {
        let domain = value.split('@').nth(1)
            .ok_or(ValidationError::new("missing_domain"))?;
        
        if self.allowed_domains.contains(&domain.to_string()) {
            Ok(())
        } else {
            Err(ValidationError::new("domain_not_allowed")
                .with_context("allowed", &self.allowed_domains))
        }
    }
    
    fn name(&self) -> &'static str { "custom_domain" }
    fn severity(&self) -> Severity { Severity::Error }
}

// Register at startup
#[ctor::ctor]
fn register_domain_validator() {
    ValidatorRegistry::register(
        "corporate_email",
        || Box::new(FieldValidator::new()
            .add_rule(EmailRule::new())
            .add_rule(CustomDomainRule {
                allowed_domains: vec!["corp.com".to_string()]
            })
        )
    );
}
```

### Usage

```rust
// Automatic discovery via registry
let validator = ValidatorRegistry::get("corporate_email")
    .expect("validator registered");

validator.validate("user@corp.com", "email")?;
```

---

## Testing Strategy

### Unit Tests
- Each rule tested in isolation
- Mock validators for testing command validators
- Snapshot tests for error messages

### Integration Tests
- Registry discovery and plugin loading
- Command validation with multiple fields
- Async validation with in-memory DB mock

### Example Tests

```rust
#[test]
fn test_required_rule_fails_on_empty() { }

#[test]
fn test_pattern_rule_with_custom_regex() { }

#[test]
fn test_field_validator_combines_rules() { }

#[test]
fn test_command_validator_validates_all_fields() { }

#[test]
fn test_registry_discovers_custom_validators() { }

#[tokio::test]
async fn test_async_validation_rule() { }

#[test]
fn test_validation_error_context_stack() { }
```

---

## LOC Impact Analysis

### Current State
```
validators.rs         165 LOC
presets.rs           130 LOC (incomplete)
lib.rs                15 LOC
─────────────────────────
Total:               310 LOC (incomplete implementation)
```

### Proposed State
```
traits/mod.rs         40 LOC
traits/rule.rs       120 LOC (core + default impls)
traits/validator.rs   90 LOC (field + command validators)
rules/mod.rs          30 LOC
rules/core.rs        180 LOC (required, pattern, length, comparison)
rules/format.rs      150 LOC (email, url, uuid, ip)
rules/async.rs        80 LOC (async trait + examples)
presets.rs            80 LOC (rebuilt, now working)
registry.rs          120 LOC (singleton + plugin hooks)
builders.rs           90 LOC (fluent API)
lib.rs                50 LOC (re-exports + error types)
─────────────────────────
Total:               960 LOC (complete, extensible implementation)

Old validators.rs   → Deprecated (refactored into rules/format.rs)
Old presets.rs      → Rebuilt on new system
```

### Reduction Ratio (CLI Context)
If phenotype-validation is adopted by CLI command validators:
- **Before**: 674 LOC monolithic validate.rs + 310 LOC validation crate = 984 LOC
- **After**: Registry-based system allows:
  - 50 LOC per command validator (vs. 100+ for monolithic)
  - 80% less duplication across commands
  - Estimated **550 LOC reduction** in CLI layer

---

## Migration Path for Existing Code

### Step 1: Run Existing Tests with New System
```bash
# Old API still works via backward-compat shims
cargo test --features legacy-api
```

### Step 2: Adopt New Traits Incrementally
```rust
// Old way (still supported)
let email = is_valid_email(&value);

// New way (recommended)
let validator = ValidatorRegistry::get("email")?;
validator.validate(&value, "email")?;
```

### Step 3: Register Custom Validators
```rust
// In CLI crate initialization
use phenotype_validation::registry::ValidatorRegistry;

fn init_validators() {
    ValidatorRegistry::register("create_plan_command", || {
        Box::new(PlanCommandValidator::new())
    });
}
```

---

## Success Criteria

- [ ] All existing validators rewritten as rules
- [ ] Registry discovers 10+ predefined validators
- [ ] Custom validator example works end-to-end
- [ ] No regression in existing validation tests
- [ ] CLI adoption shows 50%+ LOC reduction
- [ ] Plugin API allows 3rd-party validators
- [ ] Error messages include context (field path, rule name, suggestion)
- [ ] 80%+ test coverage maintained or improved

---

## Open Questions

1. **Async Error Handling**: Should async rules fail the command or log warnings?
   - Proposal: Configurable severity per rule
   
2. **Localization**: Should validation messages support i18n?
   - Proposal: Error keys + separate message catalog per locale
   
3. **Performance**: Should we cache validator instances?
   - Proposal: Registry caches lazily created validators
   
4. **CLI Integration**: How should validators report multi-field errors?
   - Proposal: Collect all errors, display with field paths and indices

---

## Related Documentation

- **Architecture Decision Record**: `docs/adr/VALIDATION_REGISTRY_PATTERN.md` (to be created)
- **User Guide**: `docs/guides/VALIDATION_GUIDE.md` (to be created)
- **Migration Guide**: `docs/changes/validation-refactor/MIGRATION_GUIDE.md` (Phase 3)

---

## Timeline

- **Phase 1 (Traits)**: 3-4 hours, 1 atomic commit
- **Phase 2 (Registry)**: 4-5 hours, 1 atomic commit
- **Phase 3 (Migration)**: 2-3 hours, 1 atomic commit
- **Total**: ~10-12 hours wall-clock, 3 agents parallel or 1 agent sequential
