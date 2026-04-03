# Research Notes

## Key evidence
- `template-program-ops/scripts/validate-foundation.sh` expects root repos named `template-lang-python`, `template-lang-go`, `template-lang-typescript`, `template-lang-rust`, `template-lang-mojo`, `template-lang-zig`, `template-lang-swift`, `template-lang-kotlin`, and `template-lang-elixir-hex`.
- Running that script from shelf root currently fails immediately after `template-commons` because `template-lang-python` does not exist at the repo root.
- The embedded language layers currently live under `phenoTemplates/templates/{go,kotlin,mojo,swift,zig}`, with `python` and `rust` present only as partial stubs and `typescript`/`elixir` missing despite being listed in `phenoTemplates/registry/index.json`.
- `hexagon/templates/{kotlin,mojo,swift}` are byte-for-byte aligned with `phenoTemplates/templates/{lang}/templates/{lang}`; `go` and `zig` diverge materially.
- `phenoTemplates/templates/zig/templates/zig` contains generated artifacts (`.zig-cache`, `zig-out`) that should not stay in a canonical template surface.
- Every entry in `phenotype-registry/registry.json` still records `"path": "repos/<project>"` (e.g., lines 17‑70) even though every repo lives at the shelf root (AgilePlus, Apisync, etc.). None of the `repos/` subdirectories exist, so every path pointer in the registry resolves to a missing directory and needs a refresh to the actual root paths.
- The placeholder directories under `template-lang-{go,kotlin,python,rust,typescript,zig,elixir-hex}` exist but contain zero files, so they are no longer valid scaffolds. They should either be populated (with the canonical manifest/scripts) or archived/removed so that `validate-foundation.sh` and other tooling stop relying on empty placeholders that mirror zero metadata.

## Discovery surfaces already present
- Root layer manifests: `template-commons/contracts/template.manifest.json`, `template-domain-service-api/contracts/template.manifest.json`, `template-domain-webapp/contracts/template.manifest.json`, `template-program-ops/contracts/template.manifest.json`.
- Root smoke scripts: `template-commons/scripts/scaffold-smoke.sh`, `template-domain-service-api/scripts/scaffold-smoke.sh`, `template-domain-webapp/scripts/scaffold-smoke.sh`, `template-program-ops/scripts/scaffold-smoke.sh`.
- Embedded layer manifests/smoke scripts under `phenoTemplates/templates/{go,kotlin,mojo,swift,zig}`.
