# Governance & Audit

AgilePlus treats governance as infrastructure, not paperwork. Every action produces an immutable record. Every transition is enforced by the system.

## State Machine

Features move through a deterministic state machine:

```
Draft → Specified → Researched → Planned → Implementing → Validating → Shipped
```

Each transition has **preconditions**:

| Transition | Requires |
|------------|----------|
| Draft → Specified | Spec artifact exists with functional requirements |
| Specified → Researched | Research output attached (feasibility or codebase scan) |
| Researched → Planned | Work packages generated with dependency graph |
| Planned → Implementing | At least one WP assigned and branch created |
| Implementing → Validating | All WPs complete, governance checks queued |
| Validating → Shipped | All checks pass, branches merged |

## Audit Chain

Every state transition produces an **audit entry**:

```rust
AuditEntry {
    id: Uuid,
    feature_id: String,
    from_state: State,
    to_state: State,
    actor: String,        // "human:alice" or "agent:claude-code"
    timestamp: DateTime,
    artifact_hash: String, // SHA-256 of associated artifact
    prev_hash: String,     // SHA-256 of previous entry (chain)
    metadata: JsonValue,
}
```

The `prev_hash` field creates an **append-only hash chain** — similar to a blockchain but local. If any entry is tampered with, the chain breaks and validation fails.

## Why This Matters

### For humans

You get a complete record of every decision: who changed what, when, and why. Useful for compliance, post-mortems, and onboarding.

### For agents

Agents can't skip steps. They receive structured prompts derived from the spec and plan. Their output is validated before it affects the codebase. If an agent produces bad code, the validation stage catches it before ship.

### For teams

Multiple agents can work on different WPs in parallel. The governance system ensures they don't conflict — each WP has its own branch, and the dependency graph prevents premature merges.
