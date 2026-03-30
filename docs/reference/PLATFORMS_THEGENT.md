# `platforms/thegent` in the Phenotype `repos` checkout

## Purpose

`platforms/thegent/` is a **full checkout of [thegent](https://github.com/KooshaPari/thegent)** (dotfiles manager, polyglot dev hub, templates, governance). It is **not** a member of the parent Rust workspace.

## What it is not

- Not a `path` dependency from `crates/` in this repo.
- Not a replacement for the upstream remote—treat as a **local mirror** for bootstrap and tooling.

## Related paths

| Path | Role |
|------|------|
| `repos/worktrees/` | Live hub of named checkouts (`phenotype`, `phenotype-infrakit`, …)—**do not** treat as empty. |
| `platforms/thegent/` | Standalone thegent tree for CLI, templates, agent workflows. |

## Maintenance

- Update: `cd platforms/thegent && git pull`.
- Keep out of default `cargo build --workspace` scope (large tree).

## References

- [`docs/worklogs/WORK_LOG.md`](./../worklogs/WORK_LOG.md) — Wave 92–96, CLEAN-007

_Last updated: 2026-03-31_
