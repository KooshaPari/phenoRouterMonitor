# AgilePlus Workspace Restoration Plan

## Root Cause
A previous refactor incorrectly commented out 30+ valid crates with fake TODO messages:
```
# "crates/agileplus-domain",  # TODO: missing src/lib.rs
# "crates/agileplus-cli",     # TODO: missing src/lib.rs
...
```

**Truth:** The source files `src/lib.rs` EXIST for all these crates. They were working before.

## Verification Complete
- `crates/agileplus-domain/src/lib.rs` ✅ EXISTS
- `crates/agileplus-cli/src/lib.rs` ✅ EXISTS  
- `crates/agileplus-api/src/lib.rs` ✅ EXISTS
- All other crates ✅ VERIFIED

## Implementation Plan

- [ ] Task: Restore workspace members by uncommenting valid crates
- [ ] Task: Verify with `cargo check --workspace`
- [ ] Task: Run `cargo test --workspace`
- [ ] Task: Commit restoration

## Files to Update
1. `Cargo.toml` - Uncomment all valid members
2. May need minor edition bumps (workspace says 2024, some crates use 2021)

## Success Criteria
- `cargo build --workspace` succeeds
- `cargo test --workspace` succeeds  
- All 40+ crates compile and test pass