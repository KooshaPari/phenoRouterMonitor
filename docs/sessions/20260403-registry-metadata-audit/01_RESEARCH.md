# Research Notes: Registry Metadata Consistency

## Scope

- Phenotype shelf root: `/Users/kooshapari/CodeProjects/Phenotype/repos`
- Registry source of truth in this audit:
  - `phenotype-registry/registry.json`
  - `phenotype-registry/projects/*.json`
- Template registry index checked:
  - `phenoTemplates/registry/index.json`

## Findings

### 1) phenotype-registry internal consistency: OK

`phenotype-registry/registry.json` and `phenotype-registry/projects/*.json` are consistent with each other:

- 101 repos in both
- Per-repo entries match exactly (same keys/values)

This means drift is not from out-of-sync aggregation; it's from detection/index logic vs current repo structure.

### 2) Language detection docs are stale vs consolidated structure

`phenotype-registry/docs/index.md` describes language detection as root-manifest based (e.g. `repo/Cargo.toml`, `repo/go.mod`, etc.).

However, consolidated template registries store manifests under `templates/<lang>/...`, not at repo root:

- `phenoTemplates` has **no** root-level `Cargo.toml`, `go.mod`, `pyproject.toml`, `package.json`, or `build.zig`.
- `hexagon` has **no** root-level `Cargo.toml`, `go.mod`, `pyproject.toml`, `package.json`, `build.zig`, `mix.exs`, or `build.gradle.kts`.

Evidence:

- `phenoTemplates/templates/rust/Cargo.toml`
- `phenoTemplates/templates/go/templates/go/go.mod`
- `phenoTemplates/templates/python/pyproject.toml`
- `phenoTemplates/templates/kotlin/templates/kotlin/build.gradle.kts`
- `phenoTemplates/templates/swift/templates/swift/Package.swift`
- `phenoTemplates/templates/zig/templates/zig/build.zig`
- `hexagon/templates/elixir/mix.exs`
- `hexagon/templates/go/go.mod`
- `hexagon/templates/kotlin/build.gradle.kts`
- `hexagon/templates/rust/Cargo.toml`
- `hexagon/templates/swift/Package.swift`
- `hexagon/templates/zig/build.zig`

Expected update:

- Update language detection documentation and implementation to explicitly support consolidated layouts:
  - detect `templates/<language>/...` and include those languages
  - support nested manifests for monorepos (not only root-level manifests)

### 3) Stale language metadata for consolidated template registries

These repos have multi-language template layouts under `templates/<language>/...`, but phenotype-registry metadata collapses them to a single language:

- `phenotype-registry/projects/hexagon.json` currently lists only `zig`
  - actual templates: `elixir`, `go`, `kotlin`, `mojo`, `rust`, `swift`, `zig` (`hexagon/templates/*`)
- `phenotype-registry/projects/phenoTemplates.json` currently lists only `rust`
  - actual templates: `go`, `kotlin`, `mojo`, `python`, `rust`, `swift`, `zig` (`phenoTemplates/templates/*`)
- `phenotype-registry/projects/thegent.json` lists `python` + `typescript`
  - actual templates: `cpp`, `go`, `java`, `python`, `ruby`, `rust`, `typescript`, `zig` (`thegent/templates/*`)

Expected update:

- Expand `languages` lists for these repos to match their consolidated `templates/<language>` directories.
- Consider whether `primary_language` should remain a single language (current schema) or move to a richer classification (e.g. `primary_language: "multi"`), but schema change is optional.

### 4) Stale language metadata from manifest-only heuristics

There are multiple repos where `primary_language` is set to a language but the repository contains **zero** tracked source files for that language extension (e.g. python primary but no `.py`).

Examples (current registry values vs tracked source files):

- `Cursora`: primary `python` but no `.py` (has `.ts`)
- `Datamold`: primary `python` but no `.py` (has `.ts`)
- `Flagward`: primary `python` but no `.py` (has `.ts`)
- `Flowra`: primary `python` but no `.py` (has `.ts`)
- `Guardis`: primary `python` but no `.py` (has `.ts` + a rust fuzz target)
- `Httpora`: primary `python` but no `.py` (has `.ts`)
- `Quillr`: primary `python` but no `.py` (has `.ts`)
- `Seedloom`: primary `python` but no `.py` (has `.ts`)
- `Zerokit`: primary `python` but no `.py` (has `.ts`)
- `phenoHub`: primary `rust` but no `.rs` for core code (repo is JS/TS-heavy; `phenoHub/tests/bdd/steps.rs` exists)
- `phenoTemplates`: primary `rust` but no `.rs` (templates contain manifests but no rust sources at repo root)

Expected update:

- Refine language detection to reduce false positives from the presence of manifests/config only:
  - Require at least one representative source file for a language when selecting `primary_language`.
  - When multiple ecosystems exist (e.g. `pyproject.toml` + `package.json`), prefer the language with actual source in `src/` (or other conventional roots), not just manifest presence.

### 5) phenoTemplates template index (`framework`/`package_manager`) is stale + file is invalid JSON

`phenoTemplates/registry/index.json` is invalid JSON due to trailing commas.

It also contains stale `framework` / `package_manager` metadata vs the consolidated templates:

- Python template is PEP 621 + `hatchling` (`phenoTemplates/templates/python/pyproject.toml`), not Poetry.
- Go template uses `chi` (`phenoTemplates/templates/go/templates/go/go.mod`), not Echo.
- Kotlin template uses Ktor plugin (`phenoTemplates/templates/kotlin/templates/kotlin/build.gradle.kts`), not Spring.
- Swift template is plain SPM (`phenoTemplates/templates/swift/templates/swift/Package.swift`), not Vapor.
- Zig template uses stdlib patterns (`phenoTemplates/templates/zig/templates/zig/src/main.zig`), not `httpz`.

Expected update:

- Fix `phenoTemplates/registry/index.json` to be strict JSON.
- Update template metadata to match actual template implementations (or rename fields if `framework` is meant to be a higher-level category like `hexagonal` vs `web`).

