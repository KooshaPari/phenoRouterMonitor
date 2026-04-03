# Plane Fork — Testing Strategy

## Validation Plan

- WP1 (Fork): GitHub fork confirmed + branch created
- WP2 (Boundary adapter): Adapter compiles, API calls reach Plane endpoints
- WP3 (Quarantine): No duplicate PM code in AgilePlus after migration
- WP4 (Wire controls): Health, restart, settings panels work alongside Plane
- WP5 (Co-existence): Existing services + Plane start cleanly via process-compose
- WP6 (Archive): TracerTM/TheGent no longer appear as PM surfaces

## Test Surface

- `cargo check` on boundary adapter crate
- `cargo test` on adapter unit tests
- `process-compose up` brings Plane + AgilePlus services up cleanly
- Manual smoke test: create a Plane project, see it reflected in AgilePlus controls
