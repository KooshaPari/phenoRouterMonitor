# Snyk Automation — Quick Reference Card

## 1-Minute Setup

```bash
# Get token
export SNYK_TOKEN="your-token-from-https://app.snyk.io/account/settings"

# Install CLI
brew install snyk  # or: npm install -g snyk

# Verify
snyk auth $SNYK_TOKEN
snyk whoami
```

## Deploy Phase 1 (5 minutes)

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Scan Tier 1 repos
./scripts/snyk-deploy.sh "$SNYK_TOKEN"

# Or specific repos
./scripts/snyk-deploy.sh "$SNYK_TOKEN" AgilePlus phenotype-infrakit heliosCLI
```

## Commit Policy Files

```bash
# After deployment succeeds
git add AgilePlus/.snyk phenotype-infrakit/.snyk heliosCLI/.snyk
git commit -m "chore(security): add Snyk policy files"
git push
```

## Deploy GitHub Workflow (3 minutes)

```bash
# Copy workflow to each repo
for repo in AgilePlus phenotype-infrakit heliosCLI; do
  mkdir -p "$repo/.github/workflows"
  cp .github/workflows/snyk-scan.yml "$repo/.github/workflows/"
  cd "$repo"
  git add .github/workflows/snyk-scan.yml
  git commit -m "chore(ci): add Snyk security scan"
  git push
  cd ..
done
```

## Add GitHub Secret (3 minutes)

**Organization Secret** (recommended):
1. Go: https://github.com/KooshaPari/settings/secrets
2. New secret: Name=`SNYK_TOKEN`, Value=your-token

**Or per-repo**:
1. Go: https://github.com/KooshaPari/REPO/settings/secrets
2. New secret: Name=`SNYK_TOKEN`, Value=your-token

## Common Commands

| Task | Command |
|------|---------|
| Test repo | `snyk test` |
| Auto-fix | `snyk fix --force` |
| Monitor | `snyk monitor` |
| Generate policy | `./scripts/snyk-policy-generator.sh ./REPO` |
| Deploy all Tier 1 | `./scripts/snyk-deploy.sh $SNYK_TOKEN` |
| Deploy Tier 2 | `./scripts/snyk-deploy.sh $SNYK_TOKEN agent-wave agentapi-plusplus KaskMan` |
| Check auth | `snyk whoami` |
| View reports | `ls -la .snyk-reports/` |

## Outputs After Deployment

```
.snyk-reports/
├── snyk-deployment-TIMESTAMP.txt  ← Summary report
├── AgilePlus/
│   ├── test-TIMESTAMP.json        ← Raw vulnerabilities
│   ├── test-TIMESTAMP.txt         ← Human-readable report
│   └── snyk.sarif                 ← GitHub Code Scanning format
├── phenotype-infrakit/
│   ├── test-TIMESTAMP.json
│   ├── test-TIMESTAMP.txt
│   └── snyk.sarif
└── heliosCLI/
    ├── test-TIMESTAMP.json
    ├── test-TIMESTAMP.txt
    └── snyk.sarif
```

## View Results

1. **Local**: `cat .snyk-reports/snyk-deployment-*.txt`
2. **GitHub Code Scanning**: https://github.com/KooshaPari/REPO/security/code-scanning
3. **Snyk Dashboard**: https://app.snyk.io/dashboard

## Severity Levels

- 🔴 **Critical**: Exploit available → Must fix
- 🟠 **High**: Exploitable → Fix within 2 weeks
- 🟡 **Medium**: Possible under conditions → Fix within 30 days
- 🟢 **Low**: Difficult to exploit → Can defer

## Policy File (`.snyk`)

Created automatically in each repo. Example:

```yaml
version: v1.25.0

# Suppress vulnerabilities
ignore:
  'VULN-ID':
    - '> package':
        reason: 'Why this is approved'
        expires: '2025-12-31T00:00:00.000Z'

exclude:
  global:
    - /node_modules
    - /vendor
    - /.git

patch: {}
fix: {}
```

## Troubleshooting

| Problem | Solution |
|---------|----------|
| CLI not found | `brew install snyk` or `npm install -g snyk` |
| Auth failed | `snyk auth $SNYK_TOKEN` or check token validity |
| No vulns found | Check `.snyk` file (may be suppressed); run `snyk test --debug` |
| SARIF upload fails | Verify GitHub token has `security-events` permission |
| Workflow not running | Check repo has `.github/workflows/snyk-scan.yml` |

## Files Reference

| File | Purpose | Location |
|------|---------|----------|
| **snyk-deploy.sh** | Deploy across repos | `scripts/` |
| **snyk-policy-generator.sh** | Create .snyk files | `scripts/` |
| **snyk-scan.yml** | GitHub Actions workflow | `.github/workflows/` |
| **SNYK_SETUP_GUIDE.md** | Detailed guide | `docs/reference/` |
| **SNYK_AUTOMATION_READY.md** | Deployment status | Root |

## Timeline

| Phase | Repos | Timeline | Command |
|-------|-------|----------|---------|
| **1** | AgilePlus, phenotype-infrakit, heliosCLI | ~35 min | `./scripts/snyk-deploy.sh $SNYK_TOKEN` |
| **2** | agent-wave, agentapi-plusplus, KaskMan | ~25 min | `./scripts/snyk-deploy.sh $SNYK_TOKEN agent-wave agentapi-plusplus KaskMan` |
| **3** | forgecode, zen, vibeproxy | ~25 min | `./scripts/snyk-deploy.sh $SNYK_TOKEN forgecode zen vibeproxy` |
| **4+** | Monitoring | Ongoing | Nightly scans + PR checks (automated) |

## Next Steps

1. ✅ All infrastructure ready
2. ⏳ **Get Snyk token**: https://app.snyk.io/account/settings
3. ⏳ **Install CLI**: `brew install snyk`
4. ⏳ **Run Phase 1**: `./scripts/snyk-deploy.sh $SNYK_TOKEN`
5. ⏳ **Add GitHub secret**: Add `SNYK_TOKEN` to org settings
6. ⏳ **Deploy workflow**: Copy `.github/workflows/snyk-scan.yml` to each repo

## Documentation

- Full setup guide: `docs/reference/SNYK_SETUP_GUIDE.md`
- Automation status: `SNYK_AUTOMATION_READY.md`
- Snyk docs: https://docs.snyk.io
- CLI reference: https://docs.snyk.io/snyk-cli/cli-reference
