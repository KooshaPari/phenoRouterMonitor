# Validation

## Commands run
- `bash template-program-ops/scripts/validate-foundation.sh`
- `bash template-program-ops/scripts/validate-domains.sh`
- `cd template-program-ops && task quality`
- `git status --short`

## Outcomes
- `validate-foundation.sh`: now targets active template paths but fails at known-incomplete foundations (`phenoTemplates/templates/python` and `phenoTemplates/templates/rust`) which still miss one or more required contract/manifests.
- `validate-domains.sh`: passes with current domain repos.
- `task quality`: executes `manifest:validate`, `reconcile:validate`, `validate-foundation`, `validate-domains`, and `scaffold-smoke`.
- Empty placeholder directories at shelf root were removed.
