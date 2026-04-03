# Code Review Agents — Coordination Instructions

This document defines how multiple code review agents coordinate within the Phase 0 multi-tool review infrastructure.

## Overview

The code review system uses multiple specialized agents working in concert:
- **CodeRabbit** (AI review) — High-level code quality, design patterns, documentation
- **Cargo Audit** (dependency security) — Vulnerability scanning in Cargo.lock/Cargo.toml
- **CodeQL** (static analysis) — Dataflow, type safety, security vulnerabilities
- **GitGuardian** (secrets scanning) — Hardcoded secrets, API keys, credentials
- **OSV Scanner** (vulnerability database) — Known vulnerabilities in dependencies
- **Cargo Deny** (policy enforcement) — Dependency license compliance, advisory blocklists

## Agent Responsibilities

### CodeRabbit (AI Review)
**Scope**: Design, performance, documentation, best practices

**Rules**:
- Review ALL code changes in Rust source files (`**/*.rs`)
- Review configuration changes (`.github/workflows/`, `*.toml`, `.coderabbit.yaml`)
- Generate summary comments with key findings
- Suggest improvements to error handling and API design
- Highlight unsafe code blocks and complex logic
- Check test coverage for new functions

**Authority**: Can comment and suggest changes; cannot block merge

**Escalation**: Flag to human review if:
- Unsafe code found
- Breaking API changes detected
- Security implications unclear
- File size exceeds 500 lines

### Cargo Audit (Security Scanner)
**Scope**: Dependency vulnerability detection

**Rules**:
- Scan all `Cargo.toml` and `Cargo.lock` modifications
- Check against RUSTSEC vulnerability database
- Report known vulnerabilities with severity levels
- Suggest updates or workarounds

**Authority**: Can BLOCK merge if critical/high vulnerabilities found

**Escalation**: Automatic escalation to security team if vulnerability score > 7.0

### CodeQL (Static Analysis)
**Scope**: Type safety, dataflow analysis, security patterns

**Rules**:
- Analyze all Rust source code for potential issues
- Check for unsafe memory patterns
- Validate error handling paths
- Detect potential integer overflows
- Flag uninitialized variables

**Authority**: Can BLOCK merge if critical issues found

**Escalation**: Automatic escalation if CodeQL score < 95%

### GitGuardian (Secrets Scanner)
**Scope**: Hardcoded credentials and sensitive data

**Rules**:
- Scan all files for secrets patterns
- Check for API keys, tokens, private keys, passwords
- Validate against common secret formats
- Flag `.env` files containing actual secrets

**Authority**: MUST BLOCK merge if secrets detected

**Escalation**: Automatic escalation to security team on secret detection

### OSV Scanner (Vulnerability Database)
**Scope**: Known vulnerabilities in dependencies

**Rules**:
- Query National Vulnerability Database (NVD) and OSV
- Check all transitive dependencies
- Report CVE information with remediation
- Suggest safe version ranges

**Authority**: Can BLOCK merge if critical vulnerabilities found

**Escalation**: Automatic escalation if CVSS score > 8.0

### Cargo Deny (Policy Enforcement)
**Scope**: License compliance and dependency policies

**Rules**:
- Enforce approved license list
- Check for banned dependencies
- Validate semantic versioning compliance
- Check source registry whitelisting

**Authority**: Can BLOCK merge if policy violations found

**Escalation**: Manual review by legal/platform team required

## Coordination Protocol

### Phase 1: Parallel Analysis
All agents run in parallel on PR submission:
1. CodeRabbit: AI code review (300s timeout)
2. Cargo Audit: Dependency audit (180s timeout)
3. CodeQL: Static analysis (300s timeout)
4. GitGuardian: Secret scanning (120s timeout)
5. OSV Scanner: Vulnerability check (240s timeout)
6. Cargo Deny: Policy check (120s timeout)

### Phase 2: Consensus Building
Results are aggregated and scored:
- CodeRabbit weight: 0.40 (design, quality)
- Cargo Audit weight: 0.25 (security)
- CodeQL weight: 0.20 (correctness)
- GitGuardian weight: 0.10 (secrets)
- OSV Scanner weight: 0.15 (dependencies)
- Cargo Deny weight: 0.05 (policy)

**Consensus threshold**: 70% approval across weighted agents

### Phase 3: Decision
- **All agents pass**: Auto-merge enabled (if configured)
- **Some agents flag issues**: Human review requested
- **Critical agents block**: Merge BLOCKED until resolved

### Phase 4: Resolution
- Author fixes issues
- Agents re-run on new commit
- Consensus re-evaluated
- Merge proceeds or escalates

## Failure Modes & Recovery

### Agent Timeout
If any agent exceeds timeout:
1. Flag as "timed out" in review
2. Continue with other agents
3. Require human decision on timeout agent
4. Escalate to platform team

### Agent Disagreement
If agents have conflicting findings:
1. Surface conflict in PR comment
2. Request human review
3. Document disagreement reason
4. Update agent rules if systematic conflict

### False Positives
If an agent produces false positives:
1. Add exception to agent configuration
2. Document exception with reason
3. Update training (for ML-based agents)
4. Monitor for systemic false positive patterns

## Human Escalation Triggers

Automatic escalation to `@code-review-team` when:
- Any agent detects secrets
- CodeQL criticality score < 95%
- Cargo Audit finds vulnerability with CVSS > 7.0
- Breaking API changes detected
- PR spans multiple domains (inconsistent scoring)
- File count exceeds 20 or lines exceed 1000
- Review consensus cannot be reached within 2 hours

## Agent Configuration Updates

When updating agent rules in this document:
1. Create PR on `infrastructure/agent-rules-update-<date>`
2. Get approval from platform team lead
3. Merge and tag as version update (e.g., `v0.2.0-agents`)
4. Announce changes in #code-reviews channel
5. Monitor agent behavior for 24 hours post-update

## Testing & Validation

### Agent Health Checks
Run daily:
```bash
./scripts/validate_agents.sh
```

Checks:
- Agent connectivity and API availability
- Timeout compliance
- False positive rate
- Coverage across file types

### Dry Run Reviews
Test agents on simulated PR:
```bash
./scripts/test_agents_dry_run.sh --pr <number>
```

## Appendix: Agent-Specific Configuration

### CodeRabbit Config
See `.coderabbit.yaml` for full configuration including:
- Language-specific rules
- File-based review policies
- Auto-suggestion settings
- Comment formatting

### Cargo Audit Config
See `deny.toml` (if exists) for:
- Vulnerability database sources
- Advisory allowlists
- Version requirements
- Yanked package handling

### CodeQL Config
See `.github/codeql-config.yml` for:
- Query selection
- Languages analyzed
- Severity thresholds
- Custom queries

### GitGuardian Config
Configured via GitHub Secrets:
- `GITGUARDIAN_API_KEY` — authentication
- Custom secret patterns
- Allowlist for false positives

### OSV Scanner Config
See `.osv-scanner-config.json` for:
- Vulnerability database sources
- License policy
- Ignored advisories
- Version constraints

## Contacts

- **Code Review Lead**: @code-review-team
- **Security Team**: @security-team
- **Platform Team**: @platform-team
- **Legal/Compliance**: @legal-team

## Version

- **Document Version**: 1.0
- **Last Updated**: 2026-03-30
- **Phase**: 0 (Initial Setup)
