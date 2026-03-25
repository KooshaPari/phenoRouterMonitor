# Architecture Decision Records - phenotype-infrakit

## ADR-001: Independent Crates with No Inter-Dependencies

**Status**: Accepted
**Context**: Infrastructure crates must be consumable individually.
**Decision**: Each crate is self-contained; workspace-level dependency versions for consistency.
**Consequences**: No transitive dependency bloat; consumers pick only what they need.

## ADR-002: SHA-256 Hash Chains for Event Integrity

**Status**: Accepted
**Context**: Event stores need tamper detection.
**Decision**: Each event links to the previous via SHA-256 hash, forming a verifiable chain.
**Consequences**: Any modification to historical events is detectable; slight storage overhead for hashes.

## ADR-003: TOML for Policy Configuration

**Status**: Accepted
**Context**: Policy rules need a human-readable, version-controllable format.
**Decision**: Use TOML for policy rule definitions with programmatic loading.
**Consequences**: Easy to review in PRs; structured enough for machine parsing.

## ADR-004: Forward-Only State Machine

**Status**: Accepted
**Context**: Workflow states should progress linearly to prevent invalid rollbacks.
**Decision**: State machine enforces forward-only transitions via ordinal comparison.
**Consequences**: Prevents accidental regression; skip-state config allows controlled jumps.
