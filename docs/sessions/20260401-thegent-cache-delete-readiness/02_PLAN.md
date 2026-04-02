# Plan

## Immediate Disposition

1. Keep `KooshaPari/thegent-cache` archived.
2. Do not delete the repo yet.
3. Treat `thegent/crates/thegent-cache` as the active canonical implementation until migration is completed.

## Retirement Plan

1. Decide the final canonical identity:
   - keep `thegent-cache`
   - or finish the rename to `pyfacet`
2. Update active manifests and package metadata in `thegent` and `platforms/thegent`.
3. Remove or migrate workspace membership only after the replacement location is real.
4. Rewrite live docs to point at the final canonical identity.
5. Confirm no external consumers still rely on the archived repo URL or old package names.
6. Only then consider deleting the archived GitHub repo.

## Success Condition

`thegent-cache` becomes delete-ready only when it is no longer an active local crate identity and
the replacement location is documented, adopted, and verified.
