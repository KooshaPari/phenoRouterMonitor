# Governance flow

This document is the canonical entry point for the repo's issue-to-worktree-to-release path.

## Intake

- Use the GitHub issue templates under `.github/ISSUE_TEMPLATE/`.
- Bug reports must include version, branch, commit, and reproduction context.
- Feature requests must state scope, impact, and release considerations.
- Work should be organized through branch and worktree lanes before merge/release.
- The repo uses a main/canary lane model: `main` is release-bound, `canary` is the
  continuously synced preview lane.

## Review

- Pull requests use `.github/pull_request_template.md`.
- `CODEOWNERS` defines the default review owners and special-path ownership.
- Pre-commit and security guard hooks run through the canonical `.github/hooks/` path.

## Release

- `docs/process/versioning.md` defines SemVer and Keep a Changelog policy.
- `.github/workflows/release.yml` orchestrates gating, changelog generation, and publish.
- `.github/workflows/ci.yml` uploads Rust and Python coverage to Codecov for regression checks.
- Releases are cut from `main`; `canary` tracks `main` for early validation and preview.

## Rules of engagement

- Keep changes aligned with the canonical workflow and template files.
- Update `CHANGELOG.md` for user-visible changes.
- Do not introduce alternate release or intake paths without updating this document.
