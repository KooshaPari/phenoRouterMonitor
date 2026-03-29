# phenotype-infrakit Cargo.lock repair

## Problem
`cargo test --locked` fails because the workspace is missing `Cargo.lock`.

## Goal
Add the missing workspace lockfile and verify the workspace builds/tests with locked dependency resolution.

## Acceptance Criteria
- `Cargo.lock` exists at the workspace root.
- `cargo test --locked` succeeds for the workspace.
- No unrelated source changes are introduced.
