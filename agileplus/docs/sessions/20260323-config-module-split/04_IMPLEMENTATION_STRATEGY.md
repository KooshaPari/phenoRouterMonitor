# Implementation Strategy

- Keep `AppConfig` in `config/mod.rs` and place section configs in dedicated files.
- Implement load/validate logic in a `loader` module to keep config sections focused.
- Re-export all public types from `config/mod.rs` to avoid import churn.
