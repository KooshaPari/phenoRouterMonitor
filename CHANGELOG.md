# Changelog

All notable changes to this project will be documented in this file.

The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed
- Quality: Resolved all 48+ Clippy warnings across `agileplus-api`, `agileplus-sqlite`, `agileplus-domain`, `agileplus-events`, `agileplus-git`, `agileplus-plane`, `agileplus-triage`, and `agileplus-subcmds`.
- Quality: Simplified `manual_async_fn` usage in tests.
- Quality: Fixed `DoubleEndedIterator` usage (`filter().next_back() -> rfind()`).
- Quality: Fixed `await_holding_lock` in `agileplus-sqlite` tests.
- CLI: Resolved `E0061` argument count errors in `agileplus-cli` build.

### Changed
- Domain: Implemented `Default` for `KeychainCredentialStore`.

## [0.1.1] - 2026-03-25

### Fixed

- CI: Removed duplicated permissions key in `buf` job.
- Dashboard: Resolved Alpine scope loss on kanban board by removing `hx-trigger load`.
- Security: Bumped `time` crate to >=0.3.49 to resolve dependabot alerts.
- Quality: Resolved clippy warnings in dashboard and git snapshot.

## [0.1.0] - 2026-03-23

### Added

- Initial AgilePlus workspace release.
- Core feature, work-package, audit, governance, and workflow tooling.
