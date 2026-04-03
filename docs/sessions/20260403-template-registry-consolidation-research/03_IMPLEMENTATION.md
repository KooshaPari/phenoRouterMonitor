# Implementation

## Scope
- Consolidate template surface metadata and validation controls around actual active template sources.
- Remove stale/empty placeholder template-language directories at shelf root.

## Changes applied
1. `phenoTemplates/registry/index.json`
   - Removed nonexistent `typescript` and `elixir` template entries.
   - Updated `go.framework` from `gin` to `echo`.
   - Updated `rust.framework` from `axum` to `cargo`.

2. `phenoTemplates/README.md`
   - Removed `typescript` and `elixir` table rows.
   - Aligned remaining framework names with registry metadata.

3. `template-program-ops/scripts/validate-foundation.sh`
   - Replaced obsolete `template-lang-*` root list with current active foundations:
     - `template-commons`
     - `phenoTemplates/templates/{go,kotlin,mojo,python,rust,swift,zig}`
     - `Hexagon/templates/elixir`

4. `template-program-ops/Taskfile.yml`
   - Added `./scripts/validate-foundation.sh` and `./scripts/validate-domains.sh` into the `quality` pipeline.

5. `template-program-ops/README.md`
   - Updated mission list to match active foundation path set.
   - Documented that `task check` includes domain and foundation validators.

6. `phenotype-registry/projects/phenoTemplates.json` and `phenotype-registry/projects/hexagon.json`
   - Expanded language lists and updated `primary_language` from single-language assumptions.

7. `phenotype-registry/registry.json`
   - Synced mirrored `hexagon` and `phenoTemplates` entries with expanded language sets.

8. Removed empty placeholder repositories:
   - `template-lang-go`
   - `template-lang-kotlin`
   - `template-lang-python`
   - `template-lang-rust`
   - `template-lang-typescript`
   - `template-lang-zig`
   - `template-lang-elixir-hex`
