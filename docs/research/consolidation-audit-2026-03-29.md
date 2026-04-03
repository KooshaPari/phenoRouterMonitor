# Phenotype Ecosystem Consolidation Audit
Date: 2026-03-29

## P1 — Eliminate Entire Repos (merge into canonical)

1. **helix-logging + helix-tracing** → subsume into `tracely` → rename to `phenotype-observability`
2. **thegent-cache (FacetRs)** → moka already in phenotype-shared; expose `phenotype-shared::cache` module; archive repo
3. **phenotype-cipher** → merge into `phenotype-config/pheno-crypto`
4. **hexagon-go** → merge template into `template-lang-go`; keep `go-hex` as pure lib

## P2 — Add Shared Module, Remove Per-Repo Impls

5. Error types → `phenotype-shared/crates/phenotype-errors`
6. CLI kit (Rust) → merge clikit + apikit → `phenotype-shared/crates/phenotype-cli`
7. Go logging → standardize cliproxyapi-plusplus + KodeVibe-Go on slog stdlib
8. Python HTTP → standardize zen + portage on httpx

## P3 — Wrap 3rd-Party Libs

9. phenotype-vessel → wrap bollard crate (~50 LOC vs ~500)
10. phenotype-patch → wrap `similar` crate
11. phenotype-sentinel → wrap `governor` + `circuit-breaker`
12. pheno-crypto/phenotype-cipher → wrap RustCrypto chacha20poly1305 + age

## P4 — Version Bumps (DONE via PRs)

- serde_yaml 0.9 → serde-yaml-ng 0.10 (phenotype-gauge, phenotype-xdd-lib, tokenledger)
- thiserror 1.0 → 2 (phenotype-cipher, phenotype-forge, phenotype-nexus, phenotype-xdd-lib, clikit, thegent-plugin-host)
- tokio unpin from 1.35 → ^1 (thegent-plugin-host)
- logrus → slog stdlib (cliproxyapi-plusplus, KodeVibe-Go)
- phenotype-cli-core + hexagon-go Go 1.21 → 1.26
- KWatch bubbletea v0.25 → v1.3.x

## Duplication Matrix

| Pattern | Own impls | Canonical | Action |
|---------|-----------|-----------|--------|
| Rust logging/tracing | helix-logging, helix-tracing, tracely, AgilePlus | tracely → phenotype-observability | P1 |
| Go logging | logrus (2 repos) | slog stdlib | P4 done |
| Rust config | phenotype-config, AgilePlus creds | phenotype-config/pheno-crypto | P2 |
| Rust cache | thegent-cache, phenotype-shared inline | phenotype-shared/phenotype-cache | P1 |
| Rust crypto | phenotype-cipher, pheno-crypto | pheno-crypto (consolidated) | P1 |
| Go hex arch | go-hex, hexagon-go, template-lang-go | go-hex lib + template-lang-go | P1 |
| Rust errors | every repo | phenotype-errors (new) | P2 |
| Rust CLI kit | clikit, apikit, AgilePlus bin | phenotype-shared/phenotype-cli | P2 |
| Python CLI | thegent, phench, portage | phenotype-cli-py (new) | P2 |

## 3rd-Party Replacement Candidates

| Hand-rolled | OSS replacement | LOC saved | Priority |
|-------------|----------------|-----------|---------|
| phenotype-vessel | bollard v0.18 | ~500 | HIGH |
| phenotype-patch | similar v2.7 | ~300 | MED |
| phenotype-sentinel | governor + circuit-breaker | ~400 | MED |
| pheno-crypto AES | chacha20poly1305 + age | ~200 | HIGH (security) |

## P1 Completion Status (2026-03-29)

| Item | Status | Notes |
|------|--------|-------|
| helix-logging archived | DONE | GitHub archived |
| helix-tracing archived | DONE | GitHub archived |
| thegent-cache archived | DONE | GitHub archived |
| helix-logging + helix-tracing absorbed into tracely | DONE | PR #3 opened: https://github.com/KooshaPari/tracely/pull/3 |
