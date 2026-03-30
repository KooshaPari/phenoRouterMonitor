## Summary

Adds `From` conversions for `EventStoreError` to enable the `?` operator with standard error types.

## Changes

- `From<serde_json::Error>` for `EventStoreError` - enables `?` with JSON operations
- `From<std::io::Error>` for `EventStoreError` - enables `?` with I/O operations
- `From<T: Into<String>>` for `EventStoreError::InvalidInput` - convenient error creation

## Motivation

Reduces boilerplate by enabling the `?` operator instead of verbose `map_err` calls.

## Testing

- [x] `cargo check -p phenotype-event-sourcing` passes

## Notes

Part of stacked PR series. Depends on #94 (workspace dependencies).
