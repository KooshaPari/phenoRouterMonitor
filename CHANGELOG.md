# Changelog

All notable changes to this project will be documented in this file.

The project follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and
[Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Storage-backed backlog/queue semantics across the domain, CLI, API, MCP, and SQLite layers.
- Batch backlog import for HTTP and MCP surfaces.
- Canary workflow alignment for deploy automation.
- Coverage upload plumbing for Rust and Python CI jobs.

### Changed

- Shared backlog intent, status, priority, and sort semantics now live in the domain layer.
- Trimmed dead exports from local module filter (useModuleFilter.ts) - aligned with phenodocs centralization.

## [0.1.0] - 2026-03-23

### Added

- Initial AgilePlus workspace release.
- Core feature, work-package, audit, governance, and workflow tooling.
