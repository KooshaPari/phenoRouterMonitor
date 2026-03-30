# Phase 5: Config Core Implementation

## Status: PLANNED

## Goals
1. Migrate to figment for config management
2. Create phenotype-config-core crate
3. Replace manual TOML/YAML parsing

## Key Findings
- phenotype-config-core exists but underutilized
- Manual TOML parsing in 3+ crates
- figment provides TOML/YAML/env overlay

## Implementation Plan
1. Evaluate figment vs config-rs for phenotype-config-core
2. Migrate existing config patterns to phenotype-config-core
3. Document config hierarchy in ADR
