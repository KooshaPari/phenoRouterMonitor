# Dependency Exception Process

Process for requesting and approving exceptions to Phenotype dependency standards.

## When Exceptions Are Allowed

Standards exist to provide consistency, but exceptions are permitted when:

### Valid Exception Reasons

| Reason | Description | Approval Level |
|--------|-------------|----------------|
| **Performance Critical** | Standard dependency doesn't meet performance requirements (e.g., <1ms latency SLA) | 2 maintainers |
| **Legacy Integration** | Required for compatibility with external system or existing codebase | 1 maintainer |
| **Missing Functionality** | Standard dependency lacks required feature with no workaround | 2 maintainers |
| **Security Concern** | Standard dependency has unresolved CVE or audit concern | 1 maintainer |
| **Ecosystem Lock-in** | Project is already committed to alternative with significant migration cost | 2 maintainers + lead |
| **Research/Experimental** | Prototype or spike requiring different tooling | 1 maintainer |
| **Platform Constraint** | Target platform doesn't support standard (e.g., WASM, embedded) | 1 maintainer |

### Invalid Exception Reasons

- Personal preference
- Familiarity with different tool
- "It works fine"
- Vague performance claims without benchmarks
- Avoiding learning curve

---

## Exception Request Template

Create a file at `docs/exceptions/DEP-<project>-<dependency>.md`:

```markdown
# Exception Request: [Project] - [Non-Standard Dependency]

**Date**: YYYY-MM-DD
**Requester**: @username
**Project**: project-name
**Language**: Rust|Python|TypeScript|Go
**Function Category**: Web Framework|Database|CLI|etc.

## Standard Dependency

`standard-crate` vX.Y.Z

## Requested Exception

`non-standard-crate` vX.Y.Z

## Reason Category

[ ] Performance Critical
[x] Legacy Integration
[ ] Missing Functionality
[ ] Security Concern
[ ] Ecosystem Lock-in
[ ] Research/Experimental
[ ] Platform Constraint

## Detailed Justification

Explain why the standard dependency is not suitable:

1. Technical constraint 1
2. Technical constraint 2
3. Attempted workarounds and why they failed

## Impact Assessment

### On This Project
- Minimal - isolated to one module
- Moderate - affects architecture
- Significant - core to project design

### On Ecosystem
- None - internal tool only
- Low - small team project
- Medium - shared library
- High - foundational crate

## Migration Path

If exception is temporary, describe eventual migration:

- Blockers to resolve: [dependencies, features, etc.]
- Timeline: [target date or milestone]
- Success criteria: [when can we migrate?]

## Alternatives Considered

| Alternative | Why Not Selected |
|-------------|------------------|
| `option-1` | Reason |
| `option-2` | Reason |

## Additional Notes

Any other context for reviewers.
```

---

## Review Process

### Step 1: Submit Request

1. Create exception request document
2. Open PR against `phenotype-standards` repo
3. Tag 2 relevant maintainers for review
4. Link to relevant code/architecture docs

### Step 2: Initial Review (3 business days)

Reviewers evaluate:
- [ ] Justification is valid per exception categories
- [ ] Alternatives were adequately explored
- [ ] Impact assessment is accurate
- [ ] Documentation is complete

### Step 3: Decision

| Outcome | Action |
|---------|--------|
| **Approved** | Merge PR, add exception to registry |
| **Approved with Conditions** | Merge PR with required changes documented |
| **Denied** | Close PR with explanation, requestor can revise and resubmit |
| **Needs More Info** | Requestor has 7 days to provide additional details |

### Step 4: Exception Registry

Approved exceptions are recorded in `REGISTRY.md`:

```markdown
| Project | Function | Standard | Approved Alternative | Reason | Expiry | Approver |
|---------|----------|----------|---------------------|--------|--------|----------|
| legacy-parser | CLI | clap | structopt | Legacy lock-in | 2025-12-31 | @lead |
```

---

## Exception Expiration

Exceptions are not permanent:

| Exception Type | Default Expiry | Renewal Process |
|----------------|----------------|-----------------|
| Performance Critical | 6 months | Re-benchmark with new versions |
| Legacy Integration | 12 months | Re-assess integration requirements |
| Missing Functionality | 6 months | Check if standard now has feature |
| Security Concern | 3 months | Re-audit dependencies |
| Ecosystem Lock-in | 18 months | Re-assess migration cost |
| Research/Experimental | 3 months | Convert to standard or migrate |
| Platform Constraint | As needed | Re-assess platform requirements |

---

## Exception Registry

### Current Exceptions

| Project | Function | Standard | Approved Alternative | Reason | Expiry | Approver |
|---------|----------|----------|---------------------|--------|--------|----------|
| *None* | - | - | - | - | - | - |

### Expired Exceptions

| Project | Function | Standard | Was Using | Expired | Action Taken |
|---------|----------|----------|-----------|---------|--------------|
| *None* | - | - | - | - | - |

---

## Fast-Track Exceptions

For urgent exceptions (security, production incident):

1. Create exception request with `[URGENT]` prefix
2. Notify #engineering Slack channel
3. Any maintainer can approve with 1 additional reviewer
4. Document decision retroactively in 48 hours

---

## Exception Appeals

If exception is denied:

1. Address reviewer concerns
2. Revise request with additional justification
3. Re-submit as new PR
4. Escalate to tech lead if still denied after 2 attempts

---

## Questions?

- Slack: #engineering-standards
- Docs: `docs/standards/README.md`
