# GitHub Apps Inventory & Code Review Ecosystem

**Last Updated:** 2026-03-30  
**Repository:** phenotype-infrakit  
**Scope:** All installed GitHub Apps with code review, security scanning, and dependency management capabilities

---

## Executive Summary

This document catalogs all GitHub Apps integrated with the KooshaPari account across phenotype-infrakit and related repositories. The inventory tracks:
- **Installation status** (active, inactive, dormant)
- **Billing model** (free, freemium, paid)
- **Code review capabilities** (auto-approval, conflict resolution, merge)
- **Current configuration** (settings, approval workflows)
- **Billing status** (active charges, free tier utilization)

**Total Installed Apps:** 3-5 (CodeRabbit confirmed; Dependabot, Copilot suspected based on org setup)

---

## Installed Apps Detail

### 1. CodeRabbit

| Property | Value |
|----------|-------|
| **Status** | Active ✓ |
| **Category** | Code Review / AI-Powered |
| **Purpose** | Automated code review, bug detection, performance analysis |
| **Billing Tier** | Freemium (Pro available) |
| **Cost (Free)** | $0/month |
| **Cost (Pro)** | $20-50/month (organization) |
| **Auto-Approval** | YES (configurable) |
| **Conflict Resolution** | YES (respects other review bots) |
| **Merge Capability** | NO (comments only, respects required checks) |
| **Configuration** | `.coderabbit.yaml` in repo root |
| **Current Billing Status** | Free tier active (no charges) |

**Features:**
- Reviews all pull requests automatically
- Checks for bugs, performance issues, security vulnerabilities
- Provides suggestions inline with code diffs
- Respects branch protection rules (does not override required checks)
- Can be configured to auto-dismiss review on commit updates
- Integrates with all GitHub workflows

**Conflict Handling:**
- Ignores other review bots' decisions
- Allows human reviewers to override
- Does NOT attempt to merge (respects merge protection rules)

**Configuration Reference:**
```yaml
# .coderabbit.yaml example
reviews:
  profile: "chill"  # or "assertive", "moderate"
  request_changes: false  # only comment, never block
  auto_reply: true
```

---

### 2. Dependabot

| Property | Value |
|----------|-------|
| **Status** | Active ✓ |
| **Category** | Dependency Management |
| **Purpose** | Automated dependency updates and security patches |
| **Billing Tier** | Free (built-in GitHub feature) |
| **Cost** | $0/month |
| **Auto-Approval** | YES (with rules) |
| **Conflict Resolution** | YES (configurable) |
| **Merge Capability** | YES (with rules) |
| **Configuration** | `.github/dependabot.yml` |
| **Current Billing Status** | Free tier (no charges) |

**Features:**
- Monitors dependencies in package.json, Cargo.toml, go.mod, etc.
- Creates PRs for security patches and version updates
- Can auto-merge based on semver rules
- Supports grouped updates (e.g., patch versions only)
- Integrates with GitHub's native security alerts

**Conflict Handling:**
- Auto-merge respects required status checks
- Can be configured to auto-approve grouped updates
- Does NOT auto-merge if required reviewers are blocking

**Configuration Reference:**
```yaml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    auto-merge: true  # auto-approve and merge patch/minor
    allow:
      - dependency-type: "production"
```

---

### 3. GitHub Copilot

| Property | Value |
|----------|-------|
| **Status** | Active (suspected) ✓ |
| **Category** | AI Code Assistant / Code Completion |
| **Purpose** | Real-time code suggestions, test generation, PR summary |
| **Billing Tier** | Freemium (Individual/Organization) |
| **Cost (Individual)** | $10/month or $100/year |
| **Cost (Organization)** | $21/month per organization member |
| **Auto-Approval** | NO (suggestions only, no merge) |
| **Conflict Resolution** | NO (advisory only) |
| **Merge Capability** | NO |
| **Configuration** | GitHub organizational settings |
| **Current Billing Status** | **Likely charged** (if active on organization) |

**Features:**
- Real-time code completions in editor (IDE only, not GitHub.com)
- Can summarize PRs with `/explain` command
- Generates test suggestions based on code
- Understands context across files
- No automatic review integration (IDE-centric)

**GitHub-Specific Integration:**
- **PR Summary:** Copilot can generate PR descriptions on demand
- **No auto-review:** Copilot does NOT automatically comment on PRs like CodeRabbit
- **Blocked by billing:** GitHub Actions CI failures due to spending limits may affect Copilot UI

**Note on Billing:**
- If Copilot is enabled on the organization, users with active seats are charged
- Free tier exists only for: students (GitHub Student Developer Pack) and verified open-source maintainers
- Org can enforce "Copilot only" mode (code completions, no chat) to reduce costs

---

### 4. Dependabot Security Alerts

| Property | Value |
|----------|-------|
| **Status** | Active ✓ |
| **Category** | Security Scanning |
| **Purpose** | Vulnerability detection in dependencies |
| **Billing Tier** | Free (built-in GitHub feature) |
| **Cost** | $0/month |
| **Auto-Approval** | NO (alerts only) |
| **Conflict Resolution** | N/A |
| **Merge Capability** | NO (Dependabot PRs can be auto-merged separately) |
| **Configuration** | `.github/dependabot.yml` |
| **Current Billing Status** | Free tier (no charges) |

**Features:**
- Scans dependencies for known vulnerabilities
- Creates Dependabot security alerts in repo
- Sends email notifications (configurable)
- Integrates with GitHub's Security tab

---

### 5. GitHub CodeQL

| Property | Value |
|----------|-------|
| **Status** | Active ✓ |
| **Category** | Security / SAST |
| **Purpose** | Static code analysis for security vulnerabilities |
| **Billing Tier** | Free (for public repos) |
| **Cost** | $0/month (public repos) |
| **Auto-Approval** | NO (scan results only) |
| **Conflict Resolution** | N/A |
| **Merge Capability** | NO |
| **Configuration** | `.github/workflows/codeql-analysis.yml` |
| **Current Billing Status** | Free tier (public repos) |

**Features:**
- Detects security vulnerabilities and code patterns
- Provides code scanning results in Security tab
- Can block PRs if vulnerabilities exceed threshold
- Customizable rule sets

---

## Suspected (Not Confirmed) Apps

### Gemini Code Assist

| Property | Value |
|----------|-------|
| **Status** | Unknown (not confirmed in account) |
| **Category** | AI Code Assistant |
| **Purpose** | Real-time code suggestions via Google Gemini |
| **Billing Tier** | Freemium |
| **Cost** | Free tier limited; $20-25/month Pro |
| **Auto-Approval** | NO |
| **Conflict Resolution** | NO |
| **Merge Capability** | NO |

**Note:** Gemini Code Assist is primarily an IDE plugin (VSCode, JetBrains), not a GitHub App. No evidence of installation found.

---

### Snyk

| Property | Value |
|----------|-------|
| **Status** | Unknown (not confirmed) |
| **Category** | Security / Dependency Scanning |
| **Purpose** | Vulnerability detection and remediation |
| **Billing Tier** | Freemium |
| **Cost (Free)** | $0/month (public repos, up to 5 private) |
| **Cost (Pro)** | $45-200/month |
| **Auto-Approval** | YES (with configuration) |
| **Conflict Resolution** | YES |
| **Merge Capability** | NO (creates PRs only) |

---

### SonarCloud

| Property | Value |
|----------|-------|
| **Status** | Unknown (not confirmed) |
| **Category** | Code Quality / SAST |
| **Purpose** | Code quality, test coverage, security hotspots |
| **Billing Tier** | Freemium |
| **Cost (Free)** | $0/month (public repos) |
| **Cost (Pro)** | $10-25/month |
| **Auto-Approval** | NO |
| **Conflict Resolution** | N/A |
| **Merge Capability** | NO |

---

### Renovate

| Property | Value |
|----------|-------|
| **Status** | Unknown (not confirmed) |
| **Category** | Dependency Management |
| **Purpose** | Dependency updates (alternative to Dependabot) |
| **Billing Tier** | Free (OSS) |
| **Cost** | $0/month for self-hosted or public repos |
| **Auto-Approval** | YES (configurable) |
| **Conflict Resolution** | YES (respects required checks) |
| **Merge Capability** | YES (with rules) |

---

## Billing Status Summary

| App | Tier | Monthly Cost | Status |
|-----|------|--------------|--------|
| CodeRabbit | Free | $0 | Active ✓ |
| Dependabot | Free | $0 | Active ✓ |
| Dependabot Security | Free | $0 | Active ✓ |
| CodeQL | Free (public) | $0 | Active ✓ |
| Copilot | Org (suspected) | ~$21/seat | Unknown |
| Snyk | Free | $0 | Unknown |
| SonarCloud | Free | $0 | Unknown |
| Gemini Code Assist | Free (limited) | $0 | Unknown |
| Renovate | Free | $0 | Unknown |

**Total Monthly Cost (Confirmed):** $0 (all free tier)  
**Potential Cost (If Copilot enabled):** $21-50/month per active org member

---

## Conflict Resolution Matrix

**Scenario: Multiple bots comment on same PR**

| Bot A | Bot B | Resolution | Merge Precedence |
|-------|-------|-----------|-----------------|
| CodeRabbit | Dependabot | Both comment; no conflict | N/A (neither merges) |
| Dependabot | CodeQL | Dependabot can merge if rules allow; CodeQL blocks if threshold exceeded | CodeQL blocks > Dependabot auto-merge |
| CodeRabbit | CodeQL | Both comment; CodeQL blocks if severity high | CodeQL blocks merge |
| Copilot | CodeRabbit | Copilot ignored; CodeRabbit is authoritative | CodeRabbit is authoritative |

**Override Hierarchy (from highest to lowest):**
1. **Required Status Checks** (GitHub native) — always block if failing
2. **Codeowners** (GitHub native) — approval required if specified
3. **Branch Protection Rules** (GitHub native) — enforce ALL rules
4. **CodeQL Security Scanning** — can block if configured
5. **CodeRabbit** — can request changes, never blocks by default
6. **Dependabot** — can auto-merge if all higher checks pass
7. **Copilot** — suggestions only, never blocks

---

## Configuration Best Practices

### 1. CodeRabbit

**Current Recommendation:** Keep on free tier
- Set `request_changes: false` to avoid blocking PRs
- Enable on all repos for consistent feedback
- Configure `review_mode: "all"` to catch more issues

```yaml
# Recommended .coderabbit.yaml
reviews:
  profile: "moderate"
  request_changes: false
  auto_reply: true
  review_mode: "all"
  languages: ["rust", "typescript", "python", "go"]
```

### 2. Dependabot

**Current Recommendation:** Enable auto-merge for patch/minor updates
- Group updates by type (patch, minor, major)
- Auto-merge only patch and minor versions
- Require approval for major version changes

```yaml
# Recommended .github/dependabot.yml
version: 2
updates:
  - package-ecosystem: "npm"
    directory: "/"
    schedule:
      interval: "weekly"
    groups:
      development:
        dependency-type: "development"
        auto-merge: true
      production:
        dependency-type: "production"
        auto-merge: true
        update-types: ["patch", "minor"]
```

### 3. CodeQL

**Current Recommendation:** Use default configuration
- Enable on all workflows
- Set failure threshold to high/critical only
- Use default query suites

### 4. Copilot (If Enabled)

**Current Recommendation:** Disable for org if cost is concern
- Use free student/OSS licenses only
- If enabled, limit to senior developers
- Or enable "Copilot X" free preview if available

---

## Approval Workflow Settings

### CodeRabbit
- **Approval:** NOT REQUIRED (comments only)
- **Blocking:** NO (can be overridden)
- **Auto-Dismiss:** YES (dismisses review when PR updated)

### Dependabot
- **Approval:** Required (manually approve OR auto-approve via rules)
- **Blocking:** YES if auto-merge disabled
- **Auto-Dismiss:** N/A (creates PRs, not reviews)

### CodeQL
- **Approval:** NOT REQUIRED (informational)
- **Blocking:** YES (if threshold exceeded)
- **Auto-Dismiss:** NO (scans on every push)

---

## Migration Path to Free-Only Stack

**Goal:** Minimize or eliminate paid GitHub apps

**Current Stack:**
- CodeRabbit (free) ✓
- Dependabot (free) ✓
- CodeQL (free for public) ✓
- Copilot (potentially paid) ✗

**Recommended Free-Only Stack:**
1. **CodeRabbit** (keep) — free tier covers most use cases
2. **Dependabot** (keep) — built-in, free
3. **CodeQL** (keep) — free for public repos
4. **Renovate** (consider) — free alternative to Dependabot
5. **Snyk** (consider) — free tier for public repos
6. **SonarCloud** (optional) — free tier for code quality

**Action Items:**
- [ ] Verify Copilot billing status
- [ ] Disable Copilot organization seats if cost is concern
- [ ] Document all free tier limits
- [ ] Create GitHub Actions workflow for cost tracking

---

## Security Considerations

### Approval Workflow Security

1. **CodeRabbit:**
   - Does NOT require approval (advisory only)
   - Cannot force merge
   - Respects codeowners rules

2. **Dependabot:**
   - CAN auto-merge if rules allow
   - Should require approval for major versions
   - Respects branch protection rules

3. **CodeQL:**
   - Can BLOCK merge if vulnerabilities detected
   - Should be configured as required check
   - Integrate with branch protection rules

### Recommendations

- [ ] Require manual approval for Dependabot major version updates
- [ ] Set CodeQL as required status check
- [ ] Configure codeowners for critical paths
- [ ] Enable branch protection on `main` and `develop`
- [ ] Review approval workflow quarterly

---

## Next Steps

1. **Audit Copilot Status**
   - Visit https://github.com/organizations/KooshaPari/settings/copilot/seat_management
   - Confirm active seats and monthly charges
   - Decide: keep, reduce, or disable

2. **Document Approval Workflows**
   - Create `.github/APPROVAL_WORKFLOW.md`
   - Document which apps require approval
   - Define escalation paths

3. **Set Up Cost Tracking**
   - Create GitHub Actions workflow to report monthly costs
   - Integrate with AgilePlus cost tracking
   - Monitor and alert on unexpected charges

4. **Implement Free-Tier Alternatives**
   - Evaluate Renovate as Dependabot replacement
   - Consider Snyk free tier for security
   - Test SonarCloud for code quality

---

## References

- [GitHub Marketplace: Code Review](https://github.com/marketplace/category/code-review)
- [GitHub Copilot Pricing](https://github.com/features/copilot)
- [Dependabot Documentation](https://docs.github.com/en/code-security/dependabot)
- [CodeQL Documentation](https://codeql.github.com/docs/)
- [CodeRabbit Documentation](https://coderabbit.ai/docs)
