# GOVERNANCE Worklogs

Policy, compliance, quality gates, evidence tracking, and governance documentation.

---

## 2026-03-29 - Governance Framework Implementation

**Project:** [cross-repo]
**Category:** governance
**Status:** in_progress
**Priority:** P0

### Summary

Implementing comprehensive governance framework for Phenotype projects.

### Governance Pillars

| Pillar | Focus | Owner |
|--------|-------|-------|
| **Architecture** | Patterns, decisions | Architect |
| **Quality** | Testing, standards | QA |
| **Security** | Vulnerabilities, compliance | Security |
| **Dependencies** | Updates, licensing | DevOps |
| **Documentation** | Docs as code | Tech Writing |

### Governance Bodies

| Body | Members | Cadence |
|------|---------|---------|
| Architecture Guild | 3-5 | Weekly |
| Security Committee | 2-3 | Bi-weekly |
| Release Council | 5-7 | Per release |
| Quality Gate | 2 | Per PR |

### Related
- `docs/governance/`

---

## 2026-03-29 - CODEOWNERS Implementation

**Project:** [cross-repo]
**Category:** governance
**Status:** completed
**Priority:** P0

### Summary

Set up CODEOWNERS file for all repositories.

### Current Structure

```yaml
# CODEOWNERS
# Architecture
/libs/hexagonal-rs/** @owner-architect
/libs/nexus/** @owner-architect

# AgilePlus
/crates/agileplus-domain/** @owner-agileplus
/crates/agileplus-api/** @owner-agileplus
/crates/agileplus-cli/** @owner-agileplus

# heliosCLI
/utils/pty/** @owner-helioscli
/tools/forge/** @owner-helioscli

# thegent
/platforms/thegent/** @owner-thegent
```

### Requirements

1. All critical paths have owners
2. Owners respond within 24h
3. Multiple owners for redundancy
4. CODEOWNERS reviewed quarterly

### Related
- `CODEOWNERS`

---

## 2026-03-29 - Security Policy Documentation

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Document security policies and procedures.

### Policy Sections

| Section | Content | Status |
|---------|---------|--------|
| Access Control | AuthN/AuthZ policies | Draft |
| Data Classification | Public/Internal/Confidential | Draft |
| Vulnerability Management | RUSTSEC, CVEs | Draft |
| Incident Response | Runbooks | Draft |
| Secret Management | Rotation, storage | Draft |

### Data Classification

| Level | Description | Examples |
|-------|-------------|----------|
| Public | Open source | README, specs |
| Internal | Team only | Architecture, plans |
| Confidential | Restricted | Keys, tokens |
| Restricted | Highly sensitive | Credentials, PII |

### Related
- `docs/security/policy.md`

---

## 2026-03-29 - License Compliance Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Implement license compliance checking.

### Approved Licenses

| License | Category | Usage |
|---------|----------|-------|
| MIT | Permissive | ✅ Allowed |
| Apache 2.0 | Permissive | ✅ Allowed |
| BSD | Permissive | ✅ Allowed |
| MPL 2.0 | Weak copyleft | ⚠️ Review |
| GPL 3.0 | Copyleft | ❌ Avoid |
| AGPL 3.0 | Strong copyleft | ❌ Prohibited |

### Implementation

```yaml
# .cargo/deny.toml
[licenses]
version = 2
allow = [
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "CC0-1.0",
]

[[licenses.exceptions]]
name = "Ring"
stxt = "LicenseRef-ring-BSD-2-Clause AND LicenseRef-ring-BSD-3-Clause"
whitelist = ["ring"]
```

### CI Integration

```yaml
- name: License check
  run: cargo deny check licenses
```

---

## 2026-03-29 - Vulnerability Disclosure Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Establish vulnerability disclosure process.

### Policy

```markdown
# Security Policy

## Reporting Vulnerabilities

1. **DO NOT** create public GitHub issues
2. Email: security@phenotype.dev
3. Include:
   - Description
   - Steps to reproduce
   - Potential impact
   - Suggested fix (optional)

## Response Timeline

| Severity | Initial Response | Fix Target |
|----------|-----------------|------------|
| Critical | 24 hours | 7 days |
| High | 48 hours | 30 days |
| Medium | 1 week | 90 days |
| Low | 1 month | Next release |

## Disclosure

- Credit reporters (with permission)
- Public disclosure after fix
- CVE assignment for significant issues
```

---

## 2026-03-29 - Dependency Update Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define dependency update policies and automation.

### Update Cadence

| Update Type | Frequency | Automation |
|-------------|-----------|------------|
| Security patches | Within 48h | ✅ Required |
| Minor versions | Weekly | ✅ Automated |
| Major versions | Monthly | ⚠️ Manual review |
| Breaking changes | Quarterly | ⚠️ Special review |

### Process

1. **Automated**: Renovate creates PR
2. **CI**: All tests pass
3. **Review**: Owner approves
4. **Merge**: Auto-merge if green
5. **Release**: Include in next release

### Rollback

1. Revert merge commit
2. Pin to known-good version
3. File issue for investigation

---

## 2026-03-29 - Code Review Standards

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define code review standards and expectations.

### Review Requirements

| PR Type | Reviewers | Approval |
|---------|-----------|----------|
| Hotfix | 1 | Required |
| Feature | 2 | Required |
| Architectural | 3 | Required + Guild |
| Security | Security team | Required |

### Review Checklist

```markdown
## Code Review Checklist

### Correctness
- [ ] Code does what it claims
- [ ] Edge cases handled
- [ ] No obvious bugs
- [ ] Tests included

### Design
- [ ] Follows architecture
- [ ] Appropriate abstractions
- [ ] Not over-engineered
- [ ] Consistent patterns

### Security
- [ ] No sensitive data in code
- [ ] Input validation
- [ ] Proper auth/authz
- [ ] Dependencies vetted

### Performance
- [ ] No obvious bottlenecks
- [ ] Appropriate complexity
- [ ] Resource cleanup
```

### SLAs

| PR Size | Review Time | Merge Time |
|---------|-------------|------------|
| XS (<10 lines) | 4 hours | 1 day |
| S (<100 lines) | 1 day | 2 days |
| M (<500 lines) | 2 days | 5 days |
| L (>500 lines) | 5 days | 10 days |

---

## 2026-03-29 - Quality Gates Definition

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define quality gates for code promotion.

### Gate Stages

```
Draft → Ready → Review → Quality Gate → Merge
```

### Quality Gate Criteria

| Gate | Criteria | Blocking |
|------|----------|----------|
| **Tests** | All tests pass | ✅ |
| **Coverage** | >80% coverage | ⚠️ Warning |
| **Linting** | No lint errors | ✅ |
| **Security** | No vulnerabilities | ✅ |
| **Performance** | No regressions | ⚠️ If defined |
| **Docs** | Updated if needed | ⚠️ Warning |

### Implementation

```yaml
# .github/workflows/quality-gate.yml
- name: Quality Gate
  run: |
    if ! cargo test --all; then
      echo "Tests failed"
      exit 1
    fi
    
    if ! cargo clippy -- -D warnings; then
      echo "Lint errors"
      exit 1
    fi
    
    if ! cargo audit; then
      echo "Security issues"
      exit 1
    fi
```

---

## 2026-03-29 - Release Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define release process and versioning.

### Versioning

```
MAJOR.MINOR.PATCH
  │    │    │
  │    │    └── Bug fixes
  │    └─────── New features (backward compatible)
  └──────────── Breaking changes
```

### Release Types

| Type | Frequency | Changelog |
|------|-----------|-----------|
| Patch | Weekly | Auto |
| Minor | Monthly | Manual |
| Major | Quarterly | Manual |

### Process

```bash
# 1. Prepare release
git checkout main
git pull
cargo release minor --dry-run

# 2. Update changelog
git-changelog -o CHANGELOG.md

# 3. Create release PR
gh pr create --base main

# 4. After merge
git tag v1.2.0
git push --tags

# 5. Create GitHub release
gh release create v1.2.0 --generate-notes
```

### Hotfix Process

```bash
# 1. Branch from tag
git checkout -b hotfix/v1.2.1 v1.2.0

# 2. Fix and commit
# ... fix code ...
git commit -m "fix: hotfix description"

# 3. Merge to main and tag
git checkout main
git merge hotfix/v1.2.1 --no-ff
git tag v1.2.1

# 4. Merge back to develop
git merge main hotfix/v1.2.1
```

---

## 2026-03-29 - Branching Strategy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define Git branching strategy.

### Branch Model

```
main (production)
  │
  └── develop (integration)
        │
        ├── feature/xxx
        ├── bugfix/xxx
        └── hotfix/xxx
```

### Branch Naming

```bash
# Feature branches
feature/add-xxx
feature/fix-xxx

# Bugfix branches
bugfix/issue-123

# Hotfix branches
hotfix/security-xxx

# Release branches
release/v1.2.0
```

### PR Requirements

| Branch | Target | Reviewers |
|--------|--------|----------|
| feature/* | develop | 2 |
| bugfix/* | develop | 1 |
| hotfix/* | main | 2 |
| release/* | main | 3 |

### Merge Strategy

- Squash commits for feature branches
- Merge commit for release branches
- Fast-forward for hotfixes

---

## 2026-03-29 - Commit Message Standards

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define commit message conventions.

### Format

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types

| Type | Description |
|------|-------------|
| feat | New feature |
| fix | Bug fix |
| docs | Documentation |
| style | Formatting |
| refactor | Code restructure |
| test | Adding tests |
| chore | Maintenance |

### Examples

```bash
feat(spec): add requirement validation

Implement validation for requirement text format including:
- Minimum length checks
- Required field validation
- Format regex matching

Closes #123
```

### CI Integration

```bash
# Commit message linting
commitlint --from HEAD~1
```

---

## 2026-03-29 - Documentation Standards

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define documentation requirements and standards.

### Required Documentation

| Type | Location | Owner |
|------|----------|-------|
| README | repo root | Author |
| CLAUDE.md | repo root | Architect |
| API docs | /docs/api | Developer |
| Architecture | /docs/arch | Architect |
| Runbooks | /docs/runbooks | DevOps |

### Documentation Checklist

```markdown
## PR Documentation Checklist

- [ ] README updated (if behavior changed)
- [ ] API docs updated (if public API changed)
- [ ] Examples added/updated
- [ ] Breaking changes documented
- [ ] Migration guide (if needed)
```

### Standards

1. **Format**: Markdown preferred
2. **Style**: Clear, concise
3. **Links**: Relative to repo
4. **Images**: Optimized, in repo
5. **Examples**: Working code

---

## 2026-03-29 - Testing Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define testing requirements and standards.

### Coverage Requirements

| Crate Type | Minimum Coverage | Target |
|------------|-----------------|--------|
| Core libraries | 90% | 95% |
| Application code | 80% | 90% |
| CLI tools | 70% | 85% |
| Integration tests | N/A | 100% |

### Test Categories

| Category | Purpose | Location |
|----------|---------|----------|
| Unit | Single function | `*_test.rs` |
| Integration | Multiple components | `tests/` |
| Property | Random inputs | `proptest` |
| Snapshot | Output validation | `insta` |
| Performance | Benchmarking | `benches/` |

### Test Naming

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn unit_of_work_expected_behavior() {
        // Arrange
        let input = 42;
        
        // Act
        let result = process(input);
        
        // Assert
        assert_eq!(result, expected);
    }
}
```

---

## 2026-03-29 - CI/CD Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define CI/CD pipeline requirements.

### Pipeline Stages

```
Push → Lint → Test → Build → Security → Deploy
```

### Required Checks

| Stage | Checks | Required |
|-------|--------|----------|
| Lint | clippy, fmt, commitlint | ✅ |
| Test | unit, integration | ✅ |
| Build | compile, cross-compile | ✅ |
| Security | audit, deny | ✅ |
| Coverage | codecov, tarpaulin | ⚠️ |
| Deploy | staging, production | Manual |

### SLAs

| Stage | Timeout |
|-------|---------|
| Lint | 5 min |
| Test | 15 min |
| Build | 10 min |
| Security | 5 min |

---

## 2026-03-29 - Incident Response Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define incident response procedures.

### Severity Levels

| Level | Impact | Response Time | Examples |
|-------|--------|---------------|----------|
| SEV1 | Production down | 15 min | Data loss, outage |
| SEV2 | Degraded | 1 hour | Partial outage |
| SEV3 | Minor | 4 hours | Non-critical bug |
| SEV4 | Cosmetic | 1 week | UI issues |

### Response Process

```markdown
## Incident Response

### 1. Detection
- Alert fires
- On-call responds
- Severity assessed

### 2. Communication
- Create incident channel
- Notify stakeholders
- Update status page

### 3. Mitigation
- Identify root cause
- Implement fix
- Verify resolution

### 4. Post-mortem
- Document timeline
- Identify improvements
- Track action items
```

### Runbooks

| Incident | Runbook |
|----------|---------|
| Database down | /docs/runbooks/db-outage.md |
| High latency | /docs/runbooks/latency.md |
| Security breach | /docs/runbooks/security.md |

---

## 2026-03-29 - Change Management Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define change management process.

### Change Categories

| Category | Risk | Approval | Example |
|----------|------|----------|---------|
| Standard | Low | Auto | Dependency update |
| Normal | Medium | Team lead | Feature PR |
| Major | High | Architecture guild | New service |
| Emergency | Critical | On-call | Hotfix |

### Change Process

```markdown
## Standard Change
1. Developer creates PR
2. CI runs tests
3. Auto-merge on green

## Major Change
1. Proposal created (ADR)
2. Architecture guild review
3. Proof of concept
4. Implementation review
5. Gradual rollout
```

---

## 2026-03-29 - Audit Trail Requirements

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define audit trail requirements.

### Tracked Events

| Category | Events | Retention |
|----------|--------|------------|
| Authentication | Login, logout, failed | 1 year |
| Authorization | Permission changes | 1 year |
| Data | Create, update, delete | 3 years |
| System | Config changes | 1 year |
| Security | Access attempts | 1 year |

### Event Format

```json
{
  "timestamp": "2026-03-29T12:00:00Z",
  "actor": "user@example.com",
  "action": "entity.update",
  "resource": "spec/123",
  "changes": {
    "field": "status",
    "old": "draft",
    "new": "approved"
  },
  "metadata": {
    "ip": "192.168.1.1",
    "user_agent": "Mozilla/5.0..."
  }
}
```

---

## 2026-03-29 - Data Retention Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define data retention requirements.

### Retention Periods

| Data Type | Retention | Disposal |
|-----------|-----------|----------|
| User data | Account lifetime | Delete on request |
| Application logs | 90 days | Auto-delete |
| Audit logs | 1 year | Archive then delete |
| Backups | 30 days | Rotate |
| Temp data | 7 days | Auto-delete |

### Compliance

| Regulation | Requirement |
|------------|-------------|
| GDPR | Right to deletion |
| CCPA | Data disclosure |
| SOC 2 | Retention controls |

---

## 2026-03-29 - Backup and Recovery Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define backup and recovery requirements.

### Backup Schedule

| Type | Frequency | Retention | Verification |
|------|-----------|-----------|--------------|
| Full | Daily | 30 days | Weekly |
| Incremental | Hourly | 7 days | Daily |
| Transaction log | Continuous | 24 hours | Hourly |

### Recovery Requirements

| Metric | Target |
|--------|--------|
| RTO (Recovery Time Objective) | 4 hours |
| RPO (Recovery Point Objective) | 1 hour |
| Backup success rate | >99.9% |
| Recovery test frequency | Monthly |

### Tested Scenarios

1. Database recovery
2. File recovery
3. Full system recovery
4. Cross-region failover

---

## 2026-03-29 - Access Control Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define access control requirements.

### Principle of Least Privilege

```markdown
## Access Levels

| Level | Description | Example |
|-------|-------------|---------|
| Read | View only | Viewers |
| Write | Can modify | Developers |
| Admin | Full access | Admins |
| Owner | Can delete | Team leads |

## Request Process

1. Request via #access-requests
2. Manager approval
3. 24h provisioning
4. Quarterly review
```

### Privileged Access

| Access | Control | Review |
|--------|---------|--------|
| Production SSH | Bastion only | Weekly |
| Secret access | Just-in-time | Per access |
| Admin console | MFA required | Monthly |
| Database | Read-only by default | Quarterly |

---

## 2026-03-29 - Secret Management Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define secret management requirements.

### Prohibited Practices

```markdown
## NEVER DO

- [ ] Commit secrets to Git
- [ ] Share secrets in Slack
- [ ] Hardcode in source
- [ ] Use in environment variables (except Docker)
- [ ] Log secrets
```

### Approved Storage

| Secret Type | Storage | Access |
|-------------|---------|--------|
| API keys | Vault/Doppler | Via SDK |
| Database passwords | Vault | Via secret engine |
| SSH keys | Vault | Via SSH CA |
| Certificates | Vault | Via PKI engine |

### Rotation Policy

| Secret Type | Rotation | Automated |
|-------------|----------|-----------|
| API keys | 90 days | ✅ |
| Database | 180 days | ✅ |
| SSH keys | Annual | ✅ |
| Certificates | 90 days | ✅ |

---

## 2026-03-29 - Encryption Standards

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define encryption requirements.

### Encryption at Rest

| Data Type | Algorithm | Key Size |
|-----------|-----------|----------|
| Database | AES-256 | 256-bit |
| Files | AES-256 | 256-bit |
| Backups | AES-256 | 256-bit |
| Passwords | bcrypt/argon2 | Cost 12+ |

### Encryption in Transit

| Protocol | Minimum | Required |
|----------|---------|----------|
| TLS | 1.2 | ✅ |
| TLS | 1.3 | ✅ |
| SSH | 2 | ✅ |

### Key Management

```markdown
## Key Hierarchy

Root Key (HSM)
  └── Service Keys (Vault)
        ├── Data Encryption Keys
        └── Signing Keys
```

---

## 2026-03-29 - Compliance Reporting

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define compliance reporting requirements.

### SOC 2 Controls

| Control | Evidence | Frequency |
|---------|----------|-----------|
| CC6.1 | Access logs | Monthly |
| CC7.1 | Vulnerability scans | Quarterly |
| CC7.2 | Incident reports | Per incident |
| CC8.1 | Change logs | Weekly |

### Evidence Collection

```bash
# Automated evidence collection
./scripts/compliance-collect.sh

# Generates:
# - evidence/access-logs.zip
# - evidence/vuln-scans.zip
# - evidence/change-audit.zip
```

### Reports

| Report | Audience | Frequency |
|--------|----------|-----------|
| Security scan | Security team | Weekly |
| Compliance status | Management | Monthly |
| Risk assessment | Leadership | Quarterly |
| Penetration test | Board | Annual |

---

## 2026-03-29 - Training and Certification

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define training requirements.

### Required Training

| Training | Audience | Frequency | Completion |
|----------|----------|-----------|------------|
| Security awareness | All | Annual | 100% |
| Data handling | All | Annual | 100% |
| Architecture | Engineers | Initial | 100% |
| Code review | Reviewers | Initial | 100% |

### Certifications

| Role | Recommended | Required |
|------|-------------|----------|
| Developer | AWS SAP | - |
| Security | CSSLP | - |
| Architect | TOGAF | - |
| DevOps | CKA | - |

---

## 2026-03-29 - Third-Party Risk Management

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define third-party risk assessment.

### Assessment Criteria

| Risk Factor | Weight | Score |
|-------------|--------|-------|
| Data access | High | 1-5 |
| Integration depth | Medium | 1-5 |
| Criticality | High | 1-5 |
| Security posture | High | 1-5 |

### Vendor Categories

| Category | Assessment | Monitoring |
|----------|------------|------------|
| Critical | Full review | Continuous |
| Standard | Questionnaire | Annual |
| Low | Self-assessment | Ad-hoc |

### Due Diligence

```markdown
## Vendor Assessment

1. Security questionnaire
2. SOC 2 report review
3. Penetration test results
4. Data handling practices
5. Sub-processor disclosure
```

---

## 2026-03-29 - Risk Assessment Process

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define risk assessment methodology.

### Risk Matrix

| | Low Impact | Medium Impact | High Impact |
|---|------------|---------------|-------------|
| **High likelihood** | Medium | High | Critical |
| **Medium likelihood** | Low | Medium | High |
| **Low likelihood** | Low | Low | Medium |

### Risk Categories

| Category | Examples | Owner |
|----------|---------|-------|
| Technical | Data loss, downtime | Engineering |
| Security | Breach, vulnerability | Security |
| Compliance | Regulatory, legal | Legal |
| Operational | Process failure | Operations |

### Treatment Options

1. **Avoid**: Eliminate risk source
2. **Mitigate**: Reduce likelihood/impact
3. **Transfer**: Insurance, contracts
4. **Accept**: Document and monitor

---

## 2026-03-29 - Business Continuity Planning

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define business continuity requirements.

### BCP Metrics

| Metric | Target |
|--------|--------|
| Maximum tolerable downtime | 4 hours |
| Recovery time objective | 1 hour |
| Recovery point objective | 15 minutes |

### Critical Functions

| Function | RTO | RPO |
|----------|-----|-----|
| Authentication | 15 min | 0 |
| Core API | 1 hour | 1 hour |
| Dashboard | 4 hours | 4 hours |
| Reporting | 24 hours | 24 hours |

### Plans

```markdown
## BCP Components

1. Business impact analysis
2. Recovery strategies
3. Incident response
4. Communication plan
5. Testing schedule
```

---

## 2026-03-29 - Configuration Management

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define configuration management requirements.

### Configuration Items

| Type | Versioned | Auditable |
|------|-----------|-----------|
| Code | ✅ | ✅ |
| Infrastructure | ✅ | ✅ |
| Secrets | ✅ | ✅ |
| Feature flags | ✅ | ✅ |
| Environment | ❌ | ✅ |

### Baseline Configuration

```yaml
# baseline.yaml
system:
  os_version: "22.04"
  kernel_version: "5.15"
  
packages:
  - name: openssl
    version: "3.0.2"
  - name: docker
    version: "24.0"

config:
  - path: /etc/nginx/nginx.conf
    hash: sha256:abc123
```

### Drift Detection

```bash
# Detect configuration drift
./scripts/check-baseline.sh

# Reports:
# - Missing packages
# - Version mismatches
# - Config changes
```

---

## 2026-03-29 - Compliance Automation

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Implement compliance automation.

### Tools

| Framework | Purpose |
|-----------|---------|
| OPA/Gatekeeper | Policy enforcement |
| InSpec | Compliance testing |
| Falco | Runtime security |
| Vault | Secret auditing |

### Policy as Code

```rego
# allow-only-approved-images.rego
package kubernetes.policies

deny[msg] {
  input.request.kind.kind == "Pod"
  not startswith(input.request.object.spec.containers[_].image, "approved.registry.com/")
  msg = "Container image not from approved registry"
}
```

### Continuous Compliance

```yaml
# GitHub Actions
- name: Compliance check
  run: |
    inspec exec compliance-profile --reporter json > results.json
    
- name: Upload results
  uses: actions/upload-artifact@v4
```

---

## 2026-03-29 - Architecture Decision Records

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Maintain architecture decision records.

### ADR Format

```markdown
# ADR-001: Spec-Driven Development Engine

## Status
Accepted

## Context
We need a way to capture specifications that can drive code generation.

## Decision
Implement a spec-driven development engine with:
- Markdown-based spec format
- AST parser for spec syntax
- Code generator from spec AST
- Validation against spec

## Consequences
### Positive
- Single source of truth for specs
- Automatic validation
- Generated code guaranteed to match spec

### Negative
- Learning curve for spec format
- Tooling investment required
```

### ADR Process

1. Create draft ADR
2. Architecture guild review
3. Accept/reject/supersede
4. Implement

### Related
- `docs/adr/`

---

## 2026-03-29 - Intellectual Property Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define intellectual property policies.

### Ownership

```markdown
## IP Ownership

### Employee Work
- All work created during employment is company property
- IP assignment agreement required

### Open Source
- Contribution guidelines apply
- No proprietary code in public repos

### Third-Party Code
- License compliance required
- Attribution maintained
```

### Contribution Policy

1. Contributor signs CLA
2. Commits signed (DCO)
3. License verified
4. IP clearance checked

---

## 2026-03-29 - Performance Monitoring

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define performance monitoring requirements.

### SLIs and SLOs

| Service | SLI | SLO | Error Budget |
|---------|-----|-----|-------------|
| API | Latency p99 < 500ms | 99.9% | 0.1% |
| Database | Latency p99 < 100ms | 99.95% | 0.05% |
| Agent | Success rate > 95% | 99% | 1% |

### Alerting

| Alert | Condition | Severity | Response |
|-------|-----------|----------|----------|
| High latency | p99 > 1s | Warning | Investigate |
| Error spike | 5xx > 1% | Warning | Investigate |
| Down | Success < 50% | Critical | Page |

### Dashboards

1. Service health
2. Error rates
3. Latency distributions
4. Resource utilization

---

## 2026-03-29 - Privacy Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P0

### Summary

Define privacy requirements.

### Data Collection

```markdown
## Data We Collect

| Data | Purpose | Retention |
|------|---------|-----------|
| Name | Account | Lifetime |
| Email | Communication | Lifetime |
| Usage data | Improvement | 2 years |
| Logs | Security | 90 days |

## User Rights

- Access to personal data
- Correction of data
- Deletion (right to be forgotten)
- Portability
- Objection to processing
```

### Privacy by Design

1. Minimize data collection
2. Anonymize where possible
3. Encrypt sensitive data
4. Automate deletion

---

## 2026-03-29 - Container Security

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define container security requirements.

### Image Requirements

```dockerfile
# Base image policy
FROM registry.approved.com/base/ubuntu:22.04

# No running as root
USER 1000

# Read-only filesystem
RUN chmod 555 /

# No privileged mode
# security-opt: no-new-privileges
```

### Scanning

| Stage | Tool | Gate |
|-------|------|------|
| Build | Trivy | Block critical |
| Registry | Trivy | Block critical |
| Runtime | Falco | Alert |

### Best Practices

1. Minimal base images
2. No secrets in images
3. Signed images
4. Regular updates

---

## 2026-03-29 - API Governance

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define API governance standards.

### API Design

| Standard | Version | Format |
|----------|---------|--------|
| REST | v2 | JSON |
| gRPC | v1 | Protobuf |
| GraphQL | - | Schema |

### Versioning Policy

```markdown
## Version Lifecycle

| Status | Support | Deprecation |
|--------|---------|-------------|
| Current | Full | - |
| Maintained | Security | 6 months |
| Deprecated | None | 3 months |
| Retired | None | Removed |

## Breaking Changes

- New major version required
- Migration guide mandatory
- Dual support during transition
```

### Documentation

1. OpenAPI spec required
2. Examples for all endpoints
3. Error codes documented
4. Changelog maintained

---

## 2026-03-29 - Open Source Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define open source engagement policy.

### Allowed Contributions

| Type | Approval | Process |
|------|----------|---------|
| Bug fixes | Manager | Normal PR |
| Documentation | None | Direct commit |
| Core features | Executive | Pre-approval |
| Significant | Legal | IP review |

### Contribution Requirements

1. Sign CLA
2. DCO sign-off
3. License check
4. Security review (if applicable)

### External Dependencies

1. License review
2. Security scan
3. Maintenance status
4. Replacement plan

---

## 2026-03-29 - Vendor Assessment Process

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define vendor assessment process.

### Assessment Stages

```markdown
## Vendor Lifecycle

1. Identification
   - Business need identified
   - Market research

2. Initial Screening
   - Basic due diligence
   - Pre-qualification

3. Detailed Assessment
   - Security questionnaire
   - Technical evaluation
   - Financial review

4. Approval
   - Risk committee
   - Contract negotiation

5. Ongoing Monitoring
   - Annual review
   - Continuous assessment
```

### Assessment Criteria

| Factor | Weight | Pass Score |
|--------|--------|------------|
| Security | 40% | 70% |
| Reliability | 30% | 70% |
| Support | 20% | 60% |
| Cost | 10% | N/A |

---

## 2026-03-29 - Metrics and KPIs

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define governance metrics and KPIs.

### Engineering Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Deployment frequency | Daily | ? |
| Lead time | < 1 hour | ? |
| MTTR | < 1 hour | ? |
| Change failure rate | < 5% | ? |
| Test coverage | > 80% | ? |

### Security Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Open vulnerabilities | < 10 | ? |
| Critical vulns | 0 | ? |
| Mean time to patch | < 30 days | ? |
| Security training | 100% | ? |

### Compliance Metrics

| Metric | Target | Current |
|--------|--------|---------|
| Policy coverage | 100% | ? |
| Training completion | 100% | ? |
| Audit findings | 0 major | ? |

---

## 2026-03-29 - Exception Handling

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define exception handling process.

### Exception Types

```markdown
## Policy Exceptions

| Type | Duration | Approval |
|------|----------|----------|
| Temporary | < 90 days | Manager |
| Permanent | Indefinite | Director |
| Emergency | < 48 hours | On-call |

## Exception Request

1. Document the requirement
2. Explain why policy cannot be met
3. Propose compensating controls
4. Specify review date
5. Get approval
```

### Compensating Controls

| Exception | Controls |
|-----------|----------|
| No MFA | Additional logging, IP restrictions |
| Extended retention | Enhanced security, monitoring |
| Manual process | Documentation, audit trail |

---

## 2026-03-29 - Policy Review Process

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Define policy review process.

### Review Schedule

| Policy Type | Review Frequency | Owner |
|-------------|------------------|-------|
| Security | Quarterly | CISO |
| Data | Annual | DPO |
| Operations | Annual | CTO |
| Engineering | Annual | VP Eng |

### Review Process

```markdown
## Policy Review

1. Current state assessment
2. Gap analysis
3. Industry comparison
4. Stakeholder input
5. Update draft
6. Approval
7. Communication
8. Training
```

### Version Control

```bash
# Policy versioning
policies/
├── security/
│   ├── v1.0-2024-01.md
│   ├── v1.1-2024-04.md
│   └── v2.0-2025-01.md (current)
```

---

## 2026-03-29 - SLA Definitions

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define SLA requirements.

### Service SLAs

| Service | Availability | Latency | Support |
|---------|--------------|---------|---------|
| Core API | 99.9% | p99 < 500ms | Business |
| Agent | 99% | p99 < 30s | Standard |
| Dashboard | 99.5% | p99 < 2s | Best effort |

### SLA Credits

| Breach | Credit |
|--------|--------|
| 99.9% → 99% | 10% credit |
| 99% → 95% | 25% credit |
| < 95% | 50% credit |

### Measurement

1. External monitoring
2. Synthetic transactions
3. Real user monitoring
4. Third-party validation

---

## 2026-03-29 - Technology Radar

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P2

### Summary

Maintain technology radar for tech decisions.

### Radar Categories

| Ring | Definition |
|------|------------|
| Adopt | Recommended for use |
| Trial | Worth pursuing |
| Assess | Worth exploring |
| Hold | Not recommended |

### Tech Assessment

```markdown
## Technology Assessment Template

### Name
[Technology]

### Assessment Date
[Date]

### Ring
[Adopt/Trial/Assess/Hold]

### Rationale
[Why this ring]

### Alternatives
[Other options]

### Risks
[What could go wrong]

### Decision
[Recommendation]
```

### Review Cadence

Quarterly technology radar review.

---

## 2026-03-29 - Ethics and AI Policy

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define AI ethics policy.

### AI Principles

| Principle | Implementation |
|-----------|----------------|
| Fairness | Bias testing, diverse data |
| Transparency | Explainable outputs |
| Accountability | Human oversight |
| Privacy | Data minimization |
| Security | Adversarial testing |

### AI Usage Guidelines

```markdown
## Permitted AI Uses

- Code completion (approved tools)
- Documentation generation
- Test generation
- Code review assistance

## Prohibited AI Uses

- Decision-making without human review
- Processing PII without consent
- Generating harmful content
```

### Monitoring

1. Output quality checks
2. Bias testing
3. Security reviews
4. User feedback

---

## 2026-03-29 - Incident Documentation Standards

**Project:** [cross-repo]
**Category:** governance
**Status:** pending
**Priority:** P1

### Summary

Define incident documentation requirements.

### Post-Mortem Template

```markdown
# Post-Mortem: [Incident Name]

## Summary
Brief description

## Timeline
| Time | Event |
|------|-------|
| 10:00 | Alert fired |
| 10:05 | On-call acknowledged |
| 10:15 | Root cause identified |
| 10:30 | Mitigation applied |
| 11:00 | Service restored |

## Root Cause
Detailed explanation

## Impact
- Duration: X hours
- Users affected: Y
- Revenue impact: Z

## Lessons Learned
### What went well
- ...

### What could improve
- ...

## Action Items
| Item | Owner | Due |
|------|-------|-----|
| Implement fix | @person | Date |
```

### Blameless Culture

Focus on system improvements, not individual blame.
