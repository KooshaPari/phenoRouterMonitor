# phenoSDK Wave A — contracts (SDD)

## Goal
Extract stable ports and public DTOs from `worktrees/phenoSDK/main/src/pheno/ports` (and related schemas) into versioned contract artifacts consumable by Phenotype polyglot clients.

## Acceptance
- Inventory of port modules and coupling to adapters.
- Published contract format chosen per surface (OpenAPI, protobuf, or shared JSON Schema).
- One downstream consumer PoC (e.g. TS or Go generated client) against Phenotype template libs.

## Out of scope
Moving adapter implementations or CLI in this wave.
