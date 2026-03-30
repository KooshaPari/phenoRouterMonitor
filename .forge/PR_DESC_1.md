## Summary

Adds foundation dependencies for reducing boilerplate code across the workspace.

## Changes

- `derive_more = "1.0"` - For Display, Default, From, etc. derives
- `strum = "0.26"` - For enum string conversions

## Motivation

This is the foundation for the LOC reduction initiative documented in `docs/worklogs/DUPLICATION.md` (Wave 97 findings).

## Testing

- [x] `cargo check` passes

## Notes

This is the first PR in a stacked PR series for LOC reduction.
