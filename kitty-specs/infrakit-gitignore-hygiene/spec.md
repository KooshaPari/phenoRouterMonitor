# Phenotype infrakit: workspace gitignore hygiene

## Goal
Stop ignoring in-repo workspace crates, `libs/phenotype-config-core`, and per-crate `tests/` trees so `git add` works without `-f`.

## Scope
- `.gitignore`: anchor root `tests/` only; remove blanket `libs/`; drop ignores for workspace members.

## Acceptance
- `git check-ignore` exits non-zero for `crates/phenotype-policy-engine/tests/*.rs` and `libs/phenotype-config-core/Cargo.toml`.
- `cargo check --workspace` still succeeds.
