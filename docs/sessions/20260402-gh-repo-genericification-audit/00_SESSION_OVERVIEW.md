# Session Overview

Session: `20260402-gh-repo-genericification-audit`

## Goal

Evaluate GitHub repo families for generic-core vs domain-variant consolidation using the rule:

- generic core by default
- domain-prefixed variant only when complexity, opinionated behavior, or lifecycle separation is real
- separate repo only when release cadence, ownership, runtime, or security boundary is materially distinct

## Outcome

- `AgilePlus` should remain the umbrella workspace repo, but `agileplus-plugin-*` should collapse into it as workspace packages.
- `thegent` should remain the umbrella product repo, but most `thegent-*` satellites should collapse into one extensions or runtime workspace.
- `heliosCLI`, `heliosApp`, `phenodocs`, and `helMo` remain separate, but `heliosHarness`, `heliosBench`, and `portage` are strong collapse candidates into the `heliosCLI` workspace if they are still active.
- `phenotype-shared` is the best neutral substrate for collapsing thin `phenotype-*` infra repos such as `phenotype-cipher`, `phenotype-nexus`, and `phenotype-xdd-lib`.
- `agentapi-plusplus`, `AgentMCP`, and `agent-wave` remain separate; placeholder-thin repos such as `Httpora` and `Zerokit` should collapse or archive.
- `Hexacore` is a strong generic core; `Cmdra` likely belongs inside it. `HexaGo`, `HexaPy`, and `HexaType` remain legitimate language-specific variants unless packaging is intentionally unified.

## Deliverable

- Consolidation matrix: [github-repo-genericification-matrix.md](/Users/kooshapari/CodeProjects/Phenotype/repos/docs/sessions/20260402-gh-repo-genericification-audit/artifacts/github-repo-genericification-matrix.md)
