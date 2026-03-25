# Changelog

All notable changes to this project will be documented in this file.

The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Fixed

- CI: Removed duplicated permissions key in `buf` job.
- Dashboard: Resolved Alpine scope loss on kanban board by removing `hx-trigger load`.
- Security: Bumped `time` crate to >=0.3.49 to resolve dependabot alerts.
- Quality: Resolved clippy warnings in dashboard and git snapshot.

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
