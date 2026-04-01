# SonarCloud + CircleCI Setup Guide

**Generated:** 2026-03-31

---

## SonarCloud Setup

### Step 1: Create SonarCloud Account

1. Go to: https://sonarcloud.io
2. Click "Log in" → "Log in with GitHub"
3. Authorize the SonarCloud GitHub App
4. Organization: `stealth-startup-3u` (auto-created from GitHub org)

### Step 2: Generate SonarCloud Token

1. Go to: https://sonarcloud.io/account/security/
2. Generate new token: `phenotype-ci-token`
3. **Copy the token immediately** (shown only once)

### Step 3: Add Token to GitHub Secrets

```bash
# Repo-level (if no org access)
gh secret set SONAR_TOKEN --repo KooshaPari/AgilePlus --body "YOUR_TOKEN_HERE"
gh secret set SONAR_TOKEN --repo KooshaPari/heliosCLI --body "YOUR_TOKEN_HERE"
gh secret set SONAR_TOKEN --repo KooshaPari/phenotype-infrakit --body "YOUR_TOKEN_HERE"

# Org-level (if you have admin:org scope)
gh secret set SONAR_TOKEN --org KooshaPari --body "YOUR_TOKEN_HERE"
```

### Step 4: Import Projects into SonarCloud

1. Go to: https://sonarcloud.io/projects/create
2. Select "Import GitHub organization"
3. Choose `KooshaPari`
4. Select repositories to analyze:
   - ✅ AgilePlus
   - ✅ heliosCLI
   - ✅ phenotype-infrakit
   - ⬜ thegent
   - ⬜ Other repos (Phase 2)

### Step 5: Configure Quality Gates

1. Go to: https://sonarcloud.io/organizations/stealth-startup-3u/quality_gates
2. Create new gate: `Phenotype Standards`
3. Add conditions:
   - Coverage < 80%
   - Duplicated Lines > 3%
   - Security Hotspots > 0 (critical)
   - Vulnerabilities > 0 (critical)

---

## CircleCI Setup

### Step 1: Authorize CircleCI

1. Go to: https://circleci.com/vcs-engaggio
2. Click "Log in with GitHub"
3. Authorize CircleCI GitHub App
4. Select `KooshaPari` organization

### Step 2: Add CircleCI Token to GitHub

```bash
# Generate token at: https://app.circleci.com/settings/user/tokens
gh secret set CIRCLECI_TOKEN --repo KooshaPari/AgilePlus --body "YOUR_CIRCLECI_TOKEN"
```

### Step 3: Trigger Pipeline

```bash
# Via CircleCI CLI
circleci pipeline trigger main KooshaPari/AgilePlus

# Or push a commit to trigger automatically
git add .circleci/config.yml
git commit -m "chore(ci): add CircleCI config"
git push
```

### Step 4: Verify Setup

1. Check pipeline at: https://app.circleci.com/pipelines/github/KooshaPari/AgilePlus
2. You should see:
   - rust-check job ✅
   - go-check job (for Go repos) ✅
   - rust-coverage job ✅
   - security-audit job ✅

---

## GitHub Apps Installed (Verify)

To verify installed apps:

1. Go to: https://github.com/settings/connections/applications
2. Check for:
   - SonarCloud ✅ (if logged in)
   - CircleCI ✅ (if authorized)
   - Snyk (pending)
   - GitHub Actions (built-in)

---

## Troubleshooting

### "Insufficient privileges" on SonarCloud
- Ensure you're an admin of the SonarCloud organization
- Go to: https://sonarcloud.io/organizations/stealth-startup-3u/members

### "Token not found" for CircleCI
- Verify token at: https://app.circleci.com/settings/user/tokens
- Ensure token has "All" scope

### Coverage not appearing
- Check that `coverage/` directory is being created
- Verify `sonar-scanner` is finding the `coverage.xml` file
- Check SonarCloud project → Activity → Background Tasks for errors

---

## Files Created

| File | Purpose |
|------|---------|
| `.circleci/config.yml` | CircleCI pipeline definition |
| `.github/workflows/sonarcloud.yml` | SonarCloud GitHub Actions |
| `AgilePlus/sonar-project.properties` | SonarCloud project config |
| `heliosCLI/sonar-project.properties` | SonarCloud project config |
| `phenotype-infrakit/sonar-project.properties` | SonarCloud project config |

---

## Next Steps

1. [ ] Create SonarCloud token
2. [ ] Add `SONAR_TOKEN` to GitHub Secrets
3. [ ] Import repos into SonarCloud dashboard
4. [ ] Configure quality gates
5. [ ] Authorize CircleCI
6. [ ] Verify first pipeline runs
7. [ ] Enable PR decorations
