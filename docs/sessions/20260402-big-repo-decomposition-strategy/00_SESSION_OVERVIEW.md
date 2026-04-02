# Session Overview

Session: `20260402-big-repo-decomposition-strategy`

## Goal

Define a concrete decomposition policy for large application repos in the shelf that:

- preserves strong isolation for agents
- enforces `<500 LOC` hard limits with `<350 LOC` target
- uses existing ADR, XDD, and hexagonal architecture guidance already present in the shelf
- decides when to decompose into internal modules, workspace packages, services, or separate repos

## Outcome

- Big app repos should keep collapsing downward into internal modules and workspace packages before spawning new git repos.
- New repos should be reserved for true product, runtime, release, security, or ownership boundaries.
- The default target shape is: `repo -> workspace/packages -> bounded modules -> small files`, not `repo -> more repos`.
- Existing family recommendations from the GitHub genericification audit remain valid and should be executed with file-size and architectural constraints enforced at the module level.

## Deliverable

- Strategy: [big-repo-decomposition-strategy.md](/Users/kooshapari/CodeProjects/Phenotype/repos/docs/sessions/20260402-big-repo-decomposition-strategy/artifacts/big-repo-decomposition-strategy.md)
