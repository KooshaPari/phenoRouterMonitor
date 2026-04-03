# Validation

## Commands run
- `for d in template-lang-elixir-hex template-lang-go template-lang-kotlin template-lang-python template-lang-rust template-lang-typescript template-lang-zig template-lang-mojo template-lang-swift; do ... done`
  - Result: the first seven directories exist with 0 entries; `template-lang-mojo/` and `template-lang-swift/` are missing.
- `nl -ba template-program-ops/scripts/validate-foundation.sh`
  - Result: lines 7-15 hard-code the root repo list, including the missing `template-lang-mojo` and `template-lang-swift` paths.
