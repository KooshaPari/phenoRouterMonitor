# Changelog

All notable changes to this project will be documented in this file.
## [0.1.0] - 2026-03-29

### Bug Fixes

- Skip billable CI runs - use workflow_dispatch only (#30)
- Skip billable runs (#45)
- Align shared crates for clean check, tests, and doctests (#46)
- Fix TDD test failures in domain layer (#47)
- Final cleanup for tdd test fixes (#48)
- Fix TDD test failures in domain layer (#50)
- Remove unused imports causing cargo warnings (#59)
- Coerce EventStoreError to EventSourcingError in verify_chain (#64)
- Suppress dead_code warning on StoredEvent.event_type (#65)
- Suppress dead_code warning on StoredEvent::event_type (#66)

### Documentation

- Add comparison matrix (shared with phenotype-infrakit)
- Add comparison matrix
- Replace stub spec docs with real content from codebase analysis (#49)
- Add language-agnostic hexagonal architecture specification (#51)
- Add governance files (CODEOWNERS, CI workflow) (#52)
- Add governance files (CODEOWNERS, CI workflow) (#53)
- Add language-agnostic hexagonal architecture specification (#58)
- Add USER_JOURNEYS.md and expand PRD/FR with E9 workspace ergonomics (#60)
- Finalize kitty-specs migration cleanup (#62)
- Add docs-site scaffold and verification harness

### Features

- Create phenotype-policy-engine crate
- Add hexagonal architecture adapter crates (#39)
- Add policy engine crate structure (#56)
- Initialize event sourcing crate (#57)

### Miscellaneous

- Add tests CI workflow (#12)
- Add VitePress docsite scaffold
- Add spec documentation (PRD, ADR, FR, PLAN, trackers)
- Governance sync 2026-03-25 (#33)
- Fix cargo audit configuration
- Integrate @phenotype/docs shared VitePress theme (#55)
- Migrate kitty-specs to docs/specs (AgilePlus format) (#61)
- Commit working changes from work-audit session 2026-03-28


