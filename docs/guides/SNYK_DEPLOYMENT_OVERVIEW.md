# Snyk Deployment Overview & Quick Start

Master index for the complete Snyk security scanning deployment system.

**Total Time:** 25-30 minutes
**Complexity:** Intermediate
**Last Updated:** 2026-03-30

---

## What You'll Accomplish

By completing this deployment, you will:

1. ✅ Acquire Snyk API token (personal access)
2. ✅ Scan all 30 Phenotype repositories locally
3. ✅ Generate security vulnerability reports
4. ✅ Review and suppress findings with justification
5. ✅ Deploy GitHub Actions workflows for automated scanning
6. ✅ Enable nightly scheduled security scans
7. ✅ Set up organizational monitoring and alerts

---

## The 5 Guides (Use in Order)

### Guide 1: SNYK_TOKEN_ACQUISITION_GUIDE.md
**Time:** 5-10 minutes
**What:** How to get your Snyk API token

**You will:**
- Create or log into Snyk account (free tier)
- Navigate to https://app.snyk.io
- Generate authentication token
- Store token securely (NOT in git)
- Verify token works locally

**When to use:** FIRST (before anything else)
**Skip if:** You already have a valid Snyk token

---

### Guide 2: SNYK_LOCAL_DEPLOYMENT_GUIDE.md
**Time:** 5-10 minutes (execution) + 5-10 minutes (reading output)
**What:** Run the deployment script on your machine

**You will:**
- Set up environment (token, paths, permissions)
- Run `./scripts/snyk-deploy.sh`
- Monitor progress (5-10 minutes)
- Collect reports in `.snyk-reports/` directory
- Understand vulnerability findings

**When to use:** SECOND (after token is acquired)
**Output:** 30 JSON reports + 1 summary report

---

### Guide 3: SNYK_RESULTS_REVIEW_GUIDE.md
**Time:** 10-20 minutes
**What:** Understand and manage vulnerability findings

**You will:**
- Read and interpret vulnerability reports
- Understand severity levels (critical/high/medium/low)
- Decide: fix vs. suppress each finding
- Create `.snyk` policy files with justifications
- Commit results to git with proper documentation

**When to use:** THIRD (after reports are generated)
**Output:** `.snyk` policy files + git commit

---

### Guide 4: GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md
**Time:** 5-10 minutes
**What:** Set up GitHub Actions workflows for continuous scanning

**You will:**
- Add SNYK_TOKEN to GitHub organization secrets
- Deploy `snyk-scan.yml` workflow to Tier 1 repos
- Verify workflows trigger on push/PR/schedule
- Monitor first automated scan run
- Enable nightly scheduled scans

**When to use:** FOURTH (after local deployment is complete)
**Output:** Active workflows in GitHub Actions

---

### Guide 5: SNYK_EXPECTED_OUTPUTS.md
**Time:** Reference (5-10 minutes per lookup)
**What:** Look up expected outputs at each stage

**Use for:**
- Verify your output matches examples
- Understand error messages
- Recognize when something goes wrong
- Timing expectations
- Common troubleshooting

**When to use:** ANYTIME during deployment (consult as needed)
**Reference:** Real examples of successful output

---

### Bonus: SNYK_DEPLOYMENT_CHECKLIST.md
**Time:** 5-10 minutes (summary)
**What:** All-in-one checklist for tracking progress

**Use for:**
- Print and check off as you complete each phase
- Track completion of each step
- Sign-off when done
- Track findings and follow-ups
- Reference for audits

**When to use:** ALONGSIDE other guides (keep handy)

---

## Quick Start (5 Steps)

### Step 1: Get Token (5 minutes)

```bash
# Read: SNYK_TOKEN_ACQUISITION_GUIDE.md

# Go to https://app.snyk.io
# Generate token → Copy to clipboard

export SNYK_TOKEN="your-token-here"
snyk auth $SNYK_TOKEN
# Expected: Successfully authenticated
```

### Step 2: Run Local Scan (5-10 minutes)

```bash
# Read: SNYK_LOCAL_DEPLOYMENT_GUIDE.md

cd /Users/kooshapari/CodeProjects/Phenotype/repos
export SNYK_TOKEN="your-token-here"
./scripts/snyk-deploy.sh

# Wait 5-10 minutes for completion
# Watch for: "Deployment Complete"
```

### Step 3: Review Results (10-15 minutes)

```bash
# Read: SNYK_RESULTS_REVIEW_GUIDE.md

cat .snyk-reports/report.txt
# Understand findings, decide: fix vs. suppress

# Create .snyk policy files for suppressions
git add .snyk-reports/ **/.snyk
git commit -m "security: deploy Snyk security scanning"
git push origin main
```

### Step 4: Deploy Workflows (5 minutes)

```bash
# Read: GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md

gh secret set SNYK_TOKEN --org KooshaPari
# Paste your token

# Deploy to 3 Tier 1 repos:
# - AgilePlus
# - heliosCLI
# - phenotype-infrakit

# Add .github/workflows/snyk-scan.yml to each
# Commit and push
```

### Step 5: Verify & Ongoing (5 minutes setup)

```bash
# Check GitHub Actions status
gh run list -R KooshaPari/AgilePlus -w "Snyk Security Scan" --limit 1

# Scheduled scans run daily at 2 AM UTC
# Monitor via GitHub Actions tab
```

---

## Decisions You'll Need to Make

### 1. Where to Store Token Temporarily

**Options:**
- `export SNYK_TOKEN="..."` (memory only, cleared when terminal closes)
- `.env` file (disk-based, must add to `.gitignore`)
- Password manager (external tool)

**Recommendation:** Use `export SNYK_TOKEN` for simplicity

### 2. Fix vs. Suppress Each Finding

**Decision Tree:**
```
Is it critical? → YES → FIX (never suppress)
                       └→ Update dependency immediately

Is it high severity? → YES → FIX (if easy upgrade)
                            → SUPPRESS + plan fix (if complex)

Is it dev-only? → YES → SUPPRESS (not in production)

Is it medium/low? → SUPPRESS (unless blocking)
```

**See:** SNYK_RESULTS_REVIEW_GUIDE.md Part 4-5

### 3. Who Fixes What

**Assign remediation:**
- [ ] Critical findings: Engineer A (immediate)
- [ ] High findings: Engineer B (this sprint)
- [ ] Medium findings: Backlog (next quarter)
- [ ] Low findings: Nice-to-have (when time permits)

**Create AgilePlus work items** for each finding that will be fixed

### 4. Suppression Expiration

All suppressions must have an expiration date:

```yaml
expires: 2026-06-30  # Max 90 days from today
```

**Recommendation:** 30-60 days (review before expiry)

---

## Tier 1 Repositories (Priority)

These three repos get workflow deployment first:

| Repo | Status | Workflow | Priority |
|------|--------|----------|----------|
| **AgilePlus** | ⚠️ (Check findings) | Yes | High |
| **heliosCLI** | ✅ (Likely clean) | Yes | High |
| **phenotype-infrakit** | ⚠️ (Medium findings) | Yes | High |

**What happens:**
1. Workflows deploy to all 3
2. Trigger on: push, PR, daily schedule (2 AM UTC)
3. Results in GitHub Actions tab
4. Artifacts downloadable for review

---

## After Deployment (Next Steps)

### Immediate (Today)
- [ ] Review critical findings
- [ ] Create AgilePlus work items for fixes
- [ ] Assign owners for remediation

### This Week
- [ ] Monitor first automated scans
- [ ] Fix or plan critical/high findings
- [ ] Brief team on results

### This Month
- [ ] Deploy to remaining repos (Tier 2-3)
- [ ] Remediate high-severity findings
- [ ] Establish quarterly review process

### Ongoing
- [ ] Monitor nightly scheduled scans
- [ ] Track remediation progress
- [ ] Adjust policies as needed
- [ ] Quarterly vulnerability trend review

---

## Files & Locations

### Guide Files (Read These)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/SNYK_TOKEN_ACQUISITION_GUIDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/SNYK_LOCAL_DEPLOYMENT_GUIDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/SNYK_RESULTS_REVIEW_GUIDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/SNYK_EXPECTED_OUTPUTS.md`
- `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/SNYK_DEPLOYMENT_CHECKLIST.md`

### Deployment Script
- `/Users/kooshapari/CodeProjects/Phenotype/repos/scripts/snyk-deploy.sh`

### Reports (Generated)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/report.txt` (summary)
- `/Users/kooshapari/CodeProjects/Phenotype/repos/.snyk-reports/*.json` (per-repo details)

### Policy Files (Created)
- `**/path/to/repo/.snyk` (one per repo with suppressions)

### GitHub Integration
- Organization Secrets: https://github.com/organizations/KooshaPari/settings/secrets/actions
- Workflows: `.github/workflows/snyk-scan.yml` (in each repo)
- Actions Results: https://github.com/KooshaPari/[repo]/actions

---

## Time Budget Breakdown

| Activity | Time | Notes |
|----------|------|-------|
| **Token Acquisition** | 5-10 min | Sign in to Snyk, generate token, verify |
| **Local Deployment** | 5-10 min | Run script (watching output) |
| **Results Review** | 10-15 min | Read reports, decide on findings |
| **Policy File Creation** | 5-10 min | Create `.snyk` files for suppressions |
| **Git Commit** | 5 min | Stage and commit changes |
| **GitHub Setup** | 5-10 min | Set secret, deploy workflows |
| **Workflow Verification** | 5 min | Check GitHub Actions tab |
| **Documentation** | 5-10 min | Document findings, create work items |
| **TOTAL** | **25-30 min** | One sitting, no breaks needed |

---

## Common Questions

### Q: Do I need to commit my Snyk token to git?

**A:** Absolutely not. Token should:
- ❌ Never be committed to git
- ❌ Never appear in public files
- ✅ Be stored in environment variables (memory only)
- ✅ Be stored in GitHub organization secrets (encrypted)

### Q: What if critical vulnerabilities are found?

**A:** Follow this order:
1. Understand the vulnerability (see SNYK_RESULTS_REVIEW_GUIDE.md)
2. Assess business impact (prod vs. dev-only)
3. If production: Create urgent AgilePlus work item
4. If dev-only: Suppress with justification
5. Update suppression expiry when fixed

### Q: How often do scans run?

**A:** After GitHub integration:
- On push to main
- On pull requests to main
- Daily at 2 AM UTC (automated)

### Q: Can I deploy workflows to other repos?

**A:** Yes! After Tier 1 (AgilePlus, heliosCLI, phenotype-infrakit), you can:
1. Deploy to Tier 2 repos (secondary projects)
2. Deploy to Tier 3 repos (utilities)
3. See GITHUB_WORKFLOW_DEPLOYMENT_GUIDE.md for instructions

### Q: What if I need to regenerate my token?

**A:** Go to https://app.snyk.io → Settings → Auth Token → Regenerate

Then update:
1. Local environment: `export SNYK_TOKEN="new-token"`
2. GitHub secret: `gh secret set SNYK_TOKEN --org KooshaPari`

### Q: What do I do with the reports after review?

**A:** Keep them for:
- Audit trail (git history)
- Trend analysis (compare over time)
- Documentation (in `.snyk-reports/`)
- Compliance (if needed)

---

## Support & Help

### While Following Guides

**Reference these sections:**
- **For expected outputs:** See SNYK_EXPECTED_OUTPUTS.md
- **For errors:** See troubleshooting in each guide
- **For decisions:** See Part 4-5 of SNYK_RESULTS_REVIEW_GUIDE.md
- **For checklist:** Use SNYK_DEPLOYMENT_CHECKLIST.md

### Before Starting

```bash
# Verify prerequisites
snyk --version       # Snyk CLI installed
gh --version         # GitHub CLI installed
git --version        # Git installed

# Verify access
gh auth status       # Logged into GitHub as KooshaPari
# (Snyk login happens during token acquisition)
```

### If Something Breaks

1. Check SNYK_EXPECTED_OUTPUTS.md for the error
2. Try the suggested fix
3. If still stuck, review the relevant guide section
4. Check GitHub Issues for similar problems

---

## Deployment Readiness Checklist

Before you start, verify:

- [ ] You have admin access to KooshaPari GitHub organization
- [ ] You have a Snyk account (free tier is fine)
- [ ] All repos are cloned locally at `/Users/kooshapari/CodeProjects/Phenotype/repos/`
- [ ] You're comfortable with command-line tools
- [ ] You have 30 minutes of uninterrupted time
- [ ] You have access to paste token securely (not in public chat)

---

## Success Criteria

You'll know you've succeeded when:

✅ **Token Acquisition**
- `snyk auth <token>` succeeds
- `snyk test --dry-run` works

✅ **Local Deployment**
- 30 repos scanned
- Reports in `.snyk-reports/`
- Total time: 5-10 minutes

✅ **Results Review**
- Summary report reviewed
- Findings understood
- `.snyk` policy files created with justifications

✅ **GitHub Integration**
- SNYK_TOKEN set in organization secrets
- Workflows deployed to 3 Tier 1 repos
- First workflow run completed successfully

✅ **Ongoing**
- Nightly scans running automatically
- Team can see results in GitHub Actions
- No "Unauthorized" errors in workflows

---

## Next: Start Here

1. Read the **Quick Start** section above (this document)
2. Open **SNYK_TOKEN_ACQUISITION_GUIDE.md**
3. Follow steps in order through all 5 guides
4. Use **SNYK_DEPLOYMENT_CHECKLIST.md** to track progress
5. Reference **SNYK_EXPECTED_OUTPUTS.md** anytime you need to verify output

---

## Quick Links

| Need | Link |
|------|------|
| Snyk App | https://app.snyk.io |
| GitHub Org Secrets | https://github.com/organizations/KooshaPari/settings/secrets/actions |
| AgilePlus Actions | https://github.com/KooshaPari/AgilePlus/actions |
| heliosCLI Actions | https://github.com/KooshaPari/heliosCLI/actions |
| phenotype-infrakit Actions | https://github.com/KooshaPari/phenotype-infrakit/actions |

---

**Ready to begin?** Start with SNYK_TOKEN_ACQUISITION_GUIDE.md

**Questions?** See the troubleshooting section in the relevant guide, or reference SNYK_EXPECTED_OUTPUTS.md

**Last Updated:** 2026-03-30
**Status:** Complete and ready for deployment
