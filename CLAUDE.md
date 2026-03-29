# CLAUDE.md — repos shelf root

## Identity

This is the **repos shelf** for `CodeProjects/Phenotype/organizational-shelf/repos`.
A shelf is a top-level organizational unit containing related but independent
project repositories. Think of it like a `/opt` or `~/code` directory, but
versioned and synced as a polyrepo (repo of repos).

**NOT AgilePlus.** AgilePlus is one of ~30 projects inside this shelf.
See `projects/INDEX.md` for the full catalog.

## Structure

```
repos/                          # ← YOU ARE HERE (shelf root)
├── .worktrees/                 # Canonical worktree staging area
├── .archive/                   # Archived/rejected items
├── apps/                       # Application projects (user-facing)
├── libs/                       # Shared libraries (internal packages)
├── tooling/                    # Developer tools, CLIs, scripts
├── infra/                      # Infrastructure, deployment, devops
├── platforms/                  # Platform-as-product projects
├── crates/                     # Rust workspace members
├── packages/                   # JS/TS monorepo packages
├── docs/                       # Cross-project documentation
│   ├── adr/                   #   Architecture decision records
│   └── guides/                #   How-to guides
├── scripts/                    # Cross-project utility scripts
├── governance/                 # Governance tooling (policy, scoring)
├── projects/                   # Project catalog & metadata
│   └── INDEX.md               #   Master project list
├── WORKSTORES.md               # Worktree management guide
└── REPOS_INDEX.md              # Detailed shelf index
```

## Agent Rules

**READ `AGENTS.md` FIRST.** It contains the authoritative agent interaction
rules for this shelf. Key points:

- When working on a project, cd into its directory first (e.g., `cd heliosCLI`)
- Never assume a project is at shelf root — always verify
- Test commands must run inside the target project directory, not shelf root
- File reads should specify the correct relative path from shelf root

## Project Index

See `projects/INDEX.md` for the full catalog of all projects in this shelf.

## Quick Reference

| What you need | Where to look |
|---------------|---------------|
| Project list | `projects/INDEX.md` |
| Governance rules | `AGENTS.md` |
| Architecture decisions | `docs/adr/` |
| Cross-project scripts | `scripts/` |
