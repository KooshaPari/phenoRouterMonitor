# Contract Tests

## Overview

This directory contains Pact contract test fixtures for the gRPC boundary between
the Python MCP consumer (`AgilePlusMCP`) and the Rust core provider (`AgilePlusCore`).

## Structure

```
tests/contract/
└── pacts/
    └── AgilePlusMCP-AgilePlusCore.json   # Pact contract file
```

## Running

Consumer-side contract tests (Python):
```bash
cd python && uv run pytest tests/contract/ -v
```

Provider verification (Rust):
```bash
cargo test --test pact_provider_test
```

Or use the Makefile:
```bash
make test-contracts
```

## Pact Interactions

| Interaction | Provider State |
|-------------|---------------|
| GetFeature | feature test-feature exists in planned state |
| DispatchCommand (plan) | feature test-feature exists in planned state |
| GetAuditTrail | feature test-feature has 3 audit entries |
| VerifyAuditChain | feature test-feature has 3 audit entries |
| CheckGovernanceGate | feature test-feature has all evidence for implementing -> validated |

## Notes

- gRPC Pact support via pact-rust is used for provider verification.
- `buf breaking` in CI enforces protobuf schema compatibility as a complementary check.
- Streaming RPCs (`GetAuditTrail`, `StreamAgentEvents`) are tested manually in integration
  tests; Pact covers unary RPCs.
