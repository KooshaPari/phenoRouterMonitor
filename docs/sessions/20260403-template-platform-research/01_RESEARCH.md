# Research

## What Exists In This Shelf Checkout

Present (git repos):
- `template-program-ops/` - control-plane scripts + canonical registry artifact generation.
- `template-commons/` - shared governance/contract scaffolding (reconcile rules, smoke checks).
- `template-domain/` - domain template monorepo with:
  - `template-domain/packages/api/`
  - `template-domain/packages/webapp/`
- `Hexagon/` and `hexagon/` - duplicate clones pointing at the same remote+commit; both contain `templates/<lang>/`.

Present (non-git stub):
- `phenotype-templates/` - currently only `.agileplus/*`; no `registry/index.json` or templates content.

Not present at shelf root:
- `templates/` (scaffolding repo) - absent locally.
- `phenoTemplates/` (starter registry repo) - absent locally.

## Canonical Entrypoints (Control Plane)

`template-program-ops/Taskfile.yml`:
- `task templates:registry` -> `template-program-ops/scripts/build-template-platform-registry.sh`
- `task templates:inventory` -> registry build + `template-program-ops/scripts/list-template-platform.sh`
- `task check` -> registry build + foundation/domain validation + local smoke checks

## Canonical Artifact(s)

Generated JSON registry (canonical for machine use):
- `template-program-ops/catalog/template-platform-registry.json`

Source inputs for registry generation:
- `template-program-ops/scripts/template-platform-manifest.sh` (required/optional policy overlay)
- Starter registry (optional, if present):
  - `phenoTemplates/registry/index.json`, or
  - `phenotype-templates/registry/index.json`
- Hexagonal templates discovery:
  - `find <shelf>/Hexagon/templates/*` (all first-level dirs are registered)

## Current Registry Output (Observed)

`template-program-ops/catalog/template-platform-registry.json` currently contains:
- Required: `template-commons`, `template-domain/packages/api`, `template-domain/packages/webapp`, `template-program-ops`
- Optional: all `Hexagon/templates/{elixir,go,kotlin,mojo,rust,swift,zig}`
- No starter entries because starter registry is not present.

## Key Risks For A "Unified Registry"

1. Mixed contract levels:
   - The platform validates contract-capable surfaces (`contracts/*`, `Taskfile.yml`, `scripts/scaffold-smoke.sh`).
   - `Hexagon/templates/*` is discovered as optional but is not consistently contract-capable (many have no contracts).
2. Naming/path drift:
   - Registry dedupes by `.path`. This assumes "one template root per path".
   - If future sources want multiple templates per directory, the schema must evolve.
3. Duplicate clones / casing:
   - Both `Hexagon/` and `hexagon/` exist and point to the same remote+commit today.
   - The builder scans only `Hexagon/templates/*`, so edits in `hexagon/` will be invisible to the registry.
4. Missing starter source:
   - Without a starter registry, the platform currently can only advertise domain + hexagonal surfaces.
5. Upstream deprecation drift:
   - GitHub metadata indicates several repos are deprecated/moved (for example starters/templates moved elsewhere).
   - If this shelf is the operational truth, the control plane should declare which source-of-truth to follow.

