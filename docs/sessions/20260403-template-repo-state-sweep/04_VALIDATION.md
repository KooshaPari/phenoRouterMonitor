# Validation

Commands run:

```bash
find . -maxdepth 1 -type d \( -name 'template-*' -o -name 'template-lang-*' \) | sort
find . -maxdepth 2 -type d \( -iname '*template-rust*' -o -iname '*template-lang-swift*' -o -iname '*template-lang-mojo*' \) | sort
find . -maxdepth 1 -type d -name 'template-*' -empty | sort
for d in phenoTemplates/templates/swift phenoTemplates/templates/mojo phenoTemplates/templates/rust; do ...; done
for d in phenoInfrakit/template-lang-swift phenoInfrakit/template-lang-mojo phenoInfrakit/template-lang-rust; do ...; done
rg -n 'phenoTemplates/templates/(swift|mojo|rust)' template-program-ops
```

Results:

- Root no longer contains `template-lang-swift/` or `template-lang-mojo/`.
- Root still contains `template-lang-rust/`, but it is empty.
- `phenoTemplates/templates/swift` and `phenoTemplates/templates/mojo` are populated and contract-capable.
- `phenoTemplates/templates/rust` is present but incomplete.
- `phenoInfrakit/template-lang-{swift,mojo,rust}` are empty placeholders.
