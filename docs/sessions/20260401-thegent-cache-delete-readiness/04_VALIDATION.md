# Validation

Commands used:

```bash
find . -maxdepth 4 -type d \( -name 'thegent-cache' -o -name 'pyfacet' \)
rg -n --glob '!**/.git/**' --glob '!**/node_modules/**' --glob '!**/target/**' "thegent-cache|thegent_cache|thegent-cache-rs|pyfacet" .
cargo metadata --no-deps --format-version 1 --manifest-path thegent/crates/Cargo.toml
cargo tree --manifest-path thegent/crates/Cargo.toml -i thegent-cache-rs
```

Validated facts:

- `thegent-cache` exists in active local trees.
- `pyfacet` does not exist as an active local identity.
- `thegent-cache-rs` remains a live workspace member.
- The crate appears self-contained, but not retired.
