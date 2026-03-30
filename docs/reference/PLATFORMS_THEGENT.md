# `platforms/thegent` in the Phenotype `repos` checkout

## Purpose

`platforms/thegent/` is a **full checkout of the [thegent](https://github.com/KooshaPari/thegent)** repository (dotfiles manager, polyglot dev hub, templates, and governance tooling for the Phenotype org). It lives under `platforms/` so local work can use thegent CLI, templates, and docs **without** treating it as a Rust workspace member of the parent repo.

## What it is not

- **Not** a nested Cargo package inside the `repos` workspace (do not add it as a `path` dependency from `crates/`).
- **Not** a substitute for the canonical **thegent** remote; treat this tree as a **developer-local mirror** for bootstrap and cross-repo scripting.

## How it relates to other paths

| Location | Role |
|----------|------|
| `repos/worktrees/<name>/` | Preferred **git worktree** checkouts for feature branches (see also `worktrees/` hub below). |
| `repos/worktrees/` (hub) | Named sibling checkouts (e.g. `phenotype`, `phenotype-infrakit`, `devenv-abstraction`) used as a **multi-repo shelf**—**do not** `rmdir`; it is not an empty placeholder. |
| `platforms/thegent/` | Standalone **thegent** project used for templates, governance docs, and agent/dotfile workflows. |

## Maintenance

- **Update:** `cd platforms/thegent && git pull` (or manage as submodule/subtree if the org standardizes that).
- **Size:** Large (full Python + Rust tree). Keep out of default `cargo build --workspace` scope.

## References

- Upstream: `https://github.com/KooshaPari/thegent`
- Worklog audit: `docs/worklogs/WORK_LOG.md` (Wave 92–96 non-canonical folders, CLEAN-007)

_Last updated: 2026-03-30_
