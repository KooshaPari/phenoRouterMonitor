# Research

## Local Shelf Validation

### Active Code Presence

- `thegent-cache` exists locally at:
  - [thegent/crates/thegent-cache](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/thegent-cache)
  - [platforms/thegent/crates/thegent-cache](/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/thegent-cache)
- The active `thegent` workspace includes `thegent-cache` as a member in
  [thegent/crates/Cargo.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/crates/Cargo.toml).
- The platform mirror currently comments it out as temporarily disabled for conflicts in
  [platforms/thegent/crates/Cargo.toml](/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/crates/Cargo.toml).

### Naming State

- Rust crate package name is still `thegent-cache-rs`.
- Rust library name is still `thegent_cache`.
- Python package name is still `thegent_cache_rs`.
- No `pyfacet` references were found anywhere in the local shelf.

### Dependency Reality

- `cargo metadata --manifest-path thegent/crates/Cargo.toml` includes `thegent-cache-rs` as a
  live workspace member.
- `cargo tree -i thegent-cache-rs` shows no reverse dependencies from sibling crates, which means
  the crate appears self-contained rather than deeply wired into other crates.
- That does **not** make it delete-ready, because workspace membership, package identity, and docs
  still treat it as active.

### Documentation Drift

Live docs and audits still reference `thegent-cache` as an active component, including:

- [thegent/README.md](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/README.md)
- [thegent/docs/WORKLOG.md](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/docs/WORKLOG.md)
- [thegent/docs/reference/WORK_STREAM.md](/Users/kooshapari/CodeProjects/Phenotype/repos/thegent/docs/reference/WORK_STREAM.md)
- [platforms/thegent/README.md](/Users/kooshapari/CodeProjects/Phenotype/repos/platforms/thegent/README.md)
- [consolidation-audit-2026-03-29.md](/Users/kooshapari/CodeProjects/Phenotype/repos/docs/research/consolidation-audit-2026-03-29.md)

## Best-Practice Interpretation

Archiving the standalone GitHub repo was reasonable as a consolidation move. Deletion is not yet
appropriate because:

1. the active code identity has not been migrated away from `thegent-cache`
2. the rename to `pyfacet` has not been propagated locally
3. documentation still advertises the old identity as live
4. the platform mirror still carries the crate, even if currently disabled

## Delete Blockers

- Active workspace membership in `thegent`
- Old package names still present in manifests
- No local `pyfacet` successor identity
- Widespread documentation references
- No recorded final migration note saying the archived GitHub repo is safe to remove
