# phenotype-infrakit (ARCHIVED)

**ARCHIVED** — This repository has been drained. All content has been absorbed into:

- **[HexaKit](https://github.com/KooshaPari/HexaKit)** — canonical Rust kit (44+ crates)
- **[phenoShared](https://github.com/KooshaPari/phenoShared)** — shared primitives (22 crates)

The drain plan is documented at:
[plans/2026-06-18-infrakit-drain-2026-06-18.md](../../blob/main/plans/2026-06-18-infrakit-drain-2026-06-18.md)

## Drain Summary

| Phase | Description | Status |
|---|---|---|
| P1 | Clean workspace — removed 16 ghost/stub/orphan entries | Done |
| P2 | Drain `phenotype-security-aggregator` to HexaKit | Done (PR #268) |
| P3 | Remove 12 duplicate crates — canonical homes verified | Done |
| P4 | Strip CI and docs — all generic or historical | Done |
| P5 | Finalize repo state — archive banner, config cleanup | Done |
| P6 | Archive repository — `gh repo archive` | Pending |

## License

MIT
