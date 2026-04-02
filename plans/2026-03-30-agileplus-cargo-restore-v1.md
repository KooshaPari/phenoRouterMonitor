# AgilePlus Cargo.toml - Restore Valid Members

## Changes to Make

Uncomment valid crates, remove non-existent references:

```
VALID CRATES (uncomment):
- crates/agileplus-domain
- crates/agileplus-cli
- crates/agileplus-api
- crates/agileplus-grpc
- crates/agileplus-sqlite
- crates/agileplus-git
- crates/agileplus-plane
- crates/agileplus-telemetry
- crates/agileplus-triage
- crates/agileplus-events
- crates/agileplus-cache
- crates/agileplus-subcmds
- crates/agileplus-graph
- crates/agileplus-nats
- crates/agileplus-sync
- crates/agileplus-dashboard
- crates/agileplus-github
- crates/agileplus-p2p
- crates/agileplus-integration-tests
- crates/agileplus-contract-tests
- crates/agileplus-benchmarks

REMOVE (don't exist):
- libs/hexagonal-rs
- libs/hexkit
- libs/cipher
- libs/gauge
- libs/logger
- libs/metrics
- libs/tracing
- libs/cli-framework
- libs/config-core
- libs/xdd-lib-rs
- tools/forge
- rust/
- tests/bdd/

KEEP (exist):
- libs/nexus
- libs/plugin-registry
- libs/plugin-sample
- libs/plugin-cli
- libs/plugin-git
- libs/plugin-grpc
- libs/plugin-integration
- libs/intent-registry
- libs/health-monitor
```