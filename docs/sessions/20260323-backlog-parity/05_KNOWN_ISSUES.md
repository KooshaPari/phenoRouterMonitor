# Known Issues

- Runtime validation is now wired through `agileplus serve`, which starts both the HTTP API and
  gRPC server from the shipped CLI binary.
- The Docker Compose test runner now invokes
  `cargo test -p agileplus-integration-tests --features integration -- --include-ignored`, which
  runs the wired `full_workflow` integration target.
- MCP backlog batch import/pop/get/status still depend on the gRPC integrations path, so the first
  end-to-end follow-up should exercise the new compose flow rather than treating the harness as
  blocked.
- External GitHub branch protection / ruleset configuration is still outside the repository.
- The main remaining risk is runtime behavior under the compose stack, not missing workflow
  semantics or harness entrypoints.
