---
title: Test Results
description: Live test suite status from latest CI run
---

# Test Results

Test results from the latest CI run will appear here. The test suite includes:

- **Unit Tests**: Core SDK, domain plugins, and bridge layer
- **Integration Tests**: End-to-end gameplay validation
- **Schema Tests**: Pack validation against JSON schemas
- **Asset Tests**: Pipeline imports, optimizations, and bundles
- **Fuzzing Tests**: Property-based testing and corpus analysis

## Current Status

Results are generated automatically on each commit to the main branch via GitHub Actions. Check the [GitHub Actions workflow](https://github.com/KooshaPari/Dino/actions) for the latest runs.

## Test Coverage

DINOForge maintains a comprehensive test suite across all layers:

| Layer | Test Type | Coverage |
|-------|-----------|----------|
| SDK | Unit + Integration | Core registries, schemas, loaders |
| Warfare Domain | Unit + Integration | Archetypes, doctrines, balance |
| Economy Domain | Unit | Rates, trade models |
| Runtime Bridge | Integration | ECS mapping, stat modifiers, asset swaps |
| Asset Pipeline | Integration | Import, optimize, generate, validate |
| CLI Tools | Integration | PackCompiler, DumpTools, DinoforgeMcp |

For detailed results, see the [latest CI run](https://github.com/KooshaPari/Dino/actions/workflows/test.yml).
