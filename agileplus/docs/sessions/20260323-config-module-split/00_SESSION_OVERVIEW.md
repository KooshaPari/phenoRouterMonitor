# Session Overview

Goal: split `agileplus-domain` config into a module tree with identical behavior.

Success criteria:
- Public API unchanged (`agileplus_domain::config::*` remains valid).
- Tests cover defaults, TOML roundtrip, and env overrides.
- Targeted test run completes (or blocker captured).

Decisions:
- Keep `AppConfig` in `config/mod.rs` and move section types into dedicated modules.
