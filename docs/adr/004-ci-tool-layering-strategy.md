# ADR 004: CI Tool Layering Strategy

## Status

Proposed

## Context

External CI tools (Snyk, FOSSA, CodeRabbit, SonarCloud) have rate limits and quotas that cause PRs to fail unexpectedly. This blocks development even when core code quality is excellent.

## Decision

We implement a **Tiered CI Tool Strategy** with three layers:

### Tier 1: Blocking (Required for Merge)

Core quality tools that MUST pass:

```yaml
- cargo check      # Compilation
- cargo clippy     # Linting
- cargo fmt       # Formatting  
- cargo test       # Tests
- cargo build      # Build
```

These run FIRST and BLOCK on failure.

### Tier 2: Required Before Merge

Security/compliance tools that SHOULD pass:

```yaml
- cargo deny       # License compliance
- cargo audit      # Security advisories
- cargo machete    # Dependency checking
```

These run SECOND. PRs can proceed but should resolve these before merge.

### Tier 3: Advisory (Continue on Error)

External/metered tools with continue-on-error:

```yaml
- snyk test       # Security scanning
- fossa           # License detection
- cyclonedx       # SBOM generation
- snyk code       # Static analysis
```

These run with `continue-on-error: true`. Failures are logged but don't block.

## Consequences

### Positive

- PRs merge faster when core quality is good
- External tool quotas don't block development
- Tool usage is prioritized by importance
- Clear expectation for what blocks vs. what advises

### Negative

- Some security checks are advisory only
- Need to monitor Tier 3 failures separately
- May miss quota-related issues until post-merge

## Implementation

See `.github/workflows/snyk-scan.yml`:

```yaml
- name: Snyk Security Test
  uses: snyk/actions/node@master
  continue-on-error: true  # Tier 3 - advisory
  with:
    args: --severity-threshold=high
```

## References

- [Snyk Rate Limits](https://docs.snyk.io/rate-limiting)
- [GitHub Actions Caching](https://docs.github.com/en/actions/using-workflows/caching-dependencies)
