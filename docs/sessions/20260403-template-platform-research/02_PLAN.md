# Recommended Next Steps

## 1) Make the Control Plane Self-Consistent

- Update `template-program-ops/SPEC.md` "Expected Checkout Layout" to match reality:
  - Use `template-domain/packages/{api,webapp}` (not `template-domain-service-api/` and `template-domain-webapp/`).
  - Remove/mark `phenoTemplates/` as optional and deprecated unless you intend to clone it here.

## 2) Eliminate Hexagon Path Ambiguity

Pick ONE canonical directory name and enforce it:
- Preferred: keep `Hexagon/` (matches current builder scan path + manifest paths).
- Then remove or quarantine the other clone (`hexagon/`) to prevent silent drift.

If you want robustness instead: teach the builder to scan whichever of `Hexagon/templates` or `hexagon/templates`
exists, but that changes the canonical registry paths and should be an intentional contract decision.

## 3) Decide What "Unified Registry" Means

The current registry already provides one surface for:
- governance+contracts templates (commons)
- domain templates (api/webapp)
- hexagonal templates (language dirs)

If you also want "starter kits":
- define/clone a starter registry repo and write `registry/index.json` in the schema the builder expects.

## 4) Raise Optional Hexagon Templates To Contract-Capable (If Desired)

If you want `Hexagon/templates/*` to participate in the same contract+smoke ecosystem:
- add `contracts/template.manifest.json`
- add `contracts/reconcile.rules.yaml`
- add `scripts/scaffold-smoke.sh`
- ensure `Taskfile.yml` has at least `check`, `quality`, `release:prep`

Then `template-program-ops/scripts/validate-foundation.sh` can optionally validate more of them (or you can
add a separate `validate-hexagonal.sh` lane to avoid hard coupling).

