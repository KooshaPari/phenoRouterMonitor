# Known Issues

- Foundation coverage is incomplete for `phenoTemplates/templates/python` and `phenoTemplates/templates/rust`: required manifest/reconcile/Taskfile/scaffold files are still missing in-tree checks, so `validate-foundation.sh` remains failing for those layers until completed.
- Root-level `template-lang-*` placeholder directories were removed; tooling that still references them will now fail fast and should be migrated to the new active paths.
- `phenotype-registry` still uses global `path` values prefixed with `repos/`. This is intentional for consumers rooted at `/Users/.../Phenotype`, but should be documented in registry consumers if assumptions differ.
