# Versioning and release policy

AgilePlus uses:

- [Semantic Versioning](https://semver.org/) for release numbers.
- [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) for user-visible release history.
- Codecov coverage reporting in CI for quality regression detection.

See also `docs/process/governance.md` for the canonical intake, review, and release flow.

## Policy

- Breaking changes require a major version bump.
- Backward-compatible user-facing features require a minor version bump.
- Bug fixes and internal hardening require a patch version bump.
- `CHANGELOG.md` is the canonical release history in the repository root.
- The main/canary lane model applies: `main` is the release source, `canary` tracks
  `main` for early validation and preview feedback.

## Release flow

1. Merge the change to `main` (canary stays in sync via the sync-canary workflow).
2. Update `CHANGELOG.md` with user-visible items.
3. Use the `release.yml` workflow to gate, generate the changelog, and publish.
4. Let the changelog workflow refresh release notes from git history.
5. The publish workflow validates that the release version matches the repo manifest before upload.

## Coverage gate

- Rust and Python CI jobs upload coverage artifacts to Codecov.
- The project uses Codecov status checks to watch for regression in both project and patch scope.
