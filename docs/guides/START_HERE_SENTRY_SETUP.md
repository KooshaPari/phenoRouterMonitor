# START HERE: Sentry Setup Guide

Complete documentation for manual Sentry project creation and GitHub organization secrets configuration.

**You are here:** Main entry point for Sentry setup
**Status:** Ready to use
**Time to completion:** ~45 minutes

---

## What You'll Do

This guide walks you through:

1. **Create 3 Sentry projects** (AgilePlus, phenotype-infrakit, heliosCLI)
2. **Obtain DSN tokens** for error tracking
3. **Configure GitHub secrets** for CI/CD access
4. **Verify everything works** with a comprehensive checklist

---

## Quick Start (TL;DR)

If you're in a hurry, here's the 30-second version:

1. Go to https://sentry.io/
2. Create 3 projects (one per repository, Rust platform)
3. Copy the DSN from each project
4. Go to https://github.com/organizations/KooshaPari/settings/secrets/actions
5. Create 3 organization secrets (SENTRY_DSN_AGILEPLUS, SENTRY_DSN_INFRAKIT, SENTRY_DSN_HELIOSCLI)
6. Update local `.env` files with DSN values
7. Verify everything works using the checklist

**For detailed instructions, see below.**

---

## 3-Phase Setup Process

### Phase 1: Create Sentry Projects (15 minutes)
Follow **SENTRY_MANUAL_SETUP_GUIDE.md** to create all 3 Sentry projects and obtain DSN tokens.

**What you'll have after:**
- 3 Sentry projects (one per repository)
- 3 DSN tokens (stored securely)
- Project IDs for each project

### Phase 2: Configure GitHub Secrets (10 minutes)
Follow **GITHUB_SECRETS_SETUP_GUIDE.md** to create GitHub organization secrets.

**What you'll have after:**
- 3 GitHub organization secrets configured
- Secrets scoped to their respective repositories
- Verification that secrets are accessible

### Phase 3: Verify Setup (5 minutes)
Use **SENTRY_SETUP_READY_CHECKLIST.md** to verify everything is configured correctly.

**What you'll have after:**
- Verified all 3 projects are created
- Verified all 3 GitHub secrets are configured
- Sign-off confirming readiness
- Instructions for next phase

---

## Documentation Files

### In `/docs/guides/` (Setup Guides)

| File | Purpose | Time | Status |
|------|---------|------|--------|
| **SENTRY_SETUP_README.md** | Overview and quick start | 5 min | ✅ |
| **SENTRY_MANUAL_SETUP_GUIDE.md** | Step-by-step Sentry projects | 15 min | ✅ |
| **GITHUB_SECRETS_SETUP_GUIDE.md** | Step-by-step GitHub secrets | 10 min | ✅ |
| **SENTRY_PROJECTS_TEMPLATE.md** | Tracking template | 5 min | ✅ |
| **SENTRY_SETUP_INDEX.md** | Complete documentation index | 10 min | ✅ |
| **SENTRY_DELIVERABLES_SUMMARY.md** | What was created | 5 min | ✅ |
| **START_HERE_SENTRY_SETUP.md** | This file | 2 min | ✅ |

### In `/docs/reference/` (Reference & Verification)

| File | Purpose | Status |
|------|---------|--------|
| **SENTRY_SETUP_READY_CHECKLIST.md** | Final verification checklist | ✅ |
| **SENTRY_QUICK_REFERENCE.md** | Quick lookup guide | ✅ |

---

## Choose Your Path

### Path A: Complete Beginner
You've never used Sentry or GitHub secrets.

1. Read: **SENTRY_SETUP_README.md** (5 min)
2. Follow: **SENTRY_MANUAL_SETUP_GUIDE.md** (15 min)
3. Track: **SENTRY_PROJECTS_TEMPLATE.md** (5 min)
4. Follow: **GITHUB_SECRETS_SETUP_GUIDE.md** (10 min)
5. Verify: **SENTRY_SETUP_READY_CHECKLIST.md** (5 min)
6. Reference: **SENTRY_QUICK_REFERENCE.md** (as needed)

**Total time:** ~45 minutes

### Path B: Experienced with Sentry
You know Sentry but need GitHub secrets help.

1. Skim: **SENTRY_SETUP_README.md** (2 min)
2. Follow: **GITHUB_SECRETS_SETUP_GUIDE.md** (10 min)
3. Verify: **SENTRY_SETUP_READY_CHECKLIST.md** (5 min)
4. Reference: **SENTRY_QUICK_REFERENCE.md** (as needed)

**Total time:** ~20 minutes

### Path C: Just Need the Quick Reference
You know what you're doing.

1. Check: **SENTRY_QUICK_REFERENCE.md** (2 min)
2. Create projects, secrets, verify

**Total time:** ~30 minutes (setup) + 2 min (docs)

---

## Before You Start

### Prerequisites

- [ ] Sentry account (free tier at https://sentry.io/)
- [ ] GitHub organization admin or "Maintain" role
- [ ] Password manager or secure note for storing DSN tokens
- [ ] ~45 uninterrupted minutes

### What NOT to Do

- ❌ Do NOT commit `.env` files to git (already git-ignored)
- ❌ Do NOT share DSN tokens in Slack/email (use password manager)
- ❌ Do NOT create repository-level secrets (use organization-level)
- ❌ Do NOT use one project for multiple repos (create separate projects)

---

## Step-by-Step Overview

### Step 1: Create 3 Sentry Projects

**Guide:** SENTRY_MANUAL_SETUP_GUIDE.md (Parts 1-4)

Create projects:
```
Project 1: AgilePlus (Rust)
Project 2: phenotype-infrakit (Rust)
Project 3: heliosCLI (Rust)
```

Copy DSN for each (format: `https://key@sentry.io/project-id`)

### Step 2: Record Project Details

**Guide:** SENTRY_PROJECTS_TEMPLATE.md

Fill in the template with:
- Project IDs
- DSN URLs
- Project names
- Creation dates

### Step 3: Create GitHub Organization Secrets

**Guide:** GITHUB_SECRETS_SETUP_GUIDE.md

Create 3 secrets:
```
SENTRY_DSN_AGILEPLUS → https://...@sentry.io/...
SENTRY_DSN_INFRAKIT → https://...@sentry.io/...
SENTRY_DSN_HELIOSCLI → https://...@sentry.io/...
```

### Step 4: Update Local `.env` Files

**Guide:** SENTRY_MANUAL_SETUP_GUIDE.md (Part 7)

For each repository:
```bash
cp .env.example .env
# Edit .env and add DSN value
```

### Step 5: Verify Everything

**Guide:** SENTRY_SETUP_READY_CHECKLIST.md

Check all boxes to confirm:
- All 3 projects created
- All 3 GitHub secrets configured
- All 3 `.env` files updated
- All secrets accessible

---

## Key Information (Copy-Paste Ready)

### Sentry Projects

| Project | Platform | Secret Name |
|---------|----------|-------------|
| AgilePlus | Rust | SENTRY_DSN_AGILEPLUS |
| phenotype-infrakit | Rust | SENTRY_DSN_INFRAKIT |
| heliosCLI | Rust | SENTRY_DSN_HELIOSCLI |

### GitHub Secret Names (Case-Sensitive)

```
SENTRY_DSN_AGILEPLUS
SENTRY_DSN_INFRAKIT
SENTRY_DSN_HELIOSCLI
```

### Key URLs

```
Sentry: https://sentry.io/
GitHub Secrets: https://github.com/organizations/KooshaPari/settings/secrets/actions
AgilePlus: https://github.com/KooshaPari/AgilePlus
phenotype-infrakit: https://github.com/KooshaPari/phenotype-infrakit
heliosCLI: https://github.com/KooshaPari/heliosCLI
```

### Local `.env` Locations

```
/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.env
/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit/.env
/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/.env
```

---

## Troubleshooting

### I don't know where to start
→ Read **SENTRY_SETUP_README.md** (5 minutes)

### I'm stuck creating Sentry projects
→ See troubleshooting in **SENTRY_MANUAL_SETUP_GUIDE.md**

### I'm stuck setting up GitHub secrets
→ See troubleshooting in **GITHUB_SECRETS_SETUP_GUIDE.md**

### I need to find a quick answer
→ Check **SENTRY_QUICK_REFERENCE.md**

### I need to understand the full process
→ Read **SENTRY_SETUP_INDEX.md**

### I want to see what was created
→ Read **SENTRY_DELIVERABLES_SUMMARY.md**

---

## File Organization

```
docs/
├── guides/
│   ├── START_HERE_SENTRY_SETUP.md          ← You are here
│   ├── SENTRY_SETUP_README.md              (Overview)
│   ├── SENTRY_MANUAL_SETUP_GUIDE.md        (Sentry projects)
│   ├── GITHUB_SECRETS_SETUP_GUIDE.md       (GitHub secrets)
│   ├── SENTRY_PROJECTS_TEMPLATE.md         (Tracking)
│   ├── SENTRY_SETUP_INDEX.md               (Documentation index)
│   └── SENTRY_DELIVERABLES_SUMMARY.md      (What was created)
└── reference/
    ├── SENTRY_SETUP_READY_CHECKLIST.md     (Verification)
    └── SENTRY_QUICK_REFERENCE.md           (Quick lookup)
```

All files are in `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/`

---

## FAQ

### Q: How long does this take?
**A:** ~45 minutes total (15 min Sentry + 10 min GitHub + 5 min verification + 15 min buffer)

### Q: Do I need a paid Sentry account?
**A:** No! Free tier includes 5,000 events/month per project.

### Q: What if I make a mistake?
**A:** You can delete projects and secrets and start over. No harm done.

### Q: What's a DSN?
**A:** A unique URL that identifies your Sentry project for error reporting.
Format: `https://key@sentry.io/project-id`

### Q: Why 3 separate projects?
**A:** To track errors independently per repository and set different rules for each.

### Q: What comes next?
**A:** SDK integration into each project's code (next phase).

---

## Success Criteria

You'll know you're done when:

✅ All 3 Sentry projects are created
✅ All 3 GitHub secrets are configured
✅ All 3 local `.env` files are updated
✅ All items in the checklist are marked complete
✅ You've received the "Ready for SDK finalization" message

---

## Next Phase

Once this setup is complete:

1. **SDK Installation** — Install Sentry Rust SDK in each project
2. **Error Handling** — Configure error hooks to send to Sentry
3. **Testing** — Send test errors to verify integration
4. **Monitoring** — Set up dashboards and alerts

The next phase documentation will be provided once this setup is verified.

---

## Support

If you get stuck:

1. **Check the troubleshooting section** in the relevant guide
2. **Review the FAQ** (see above)
3. **Consult quick reference** (SENTRY_QUICK_REFERENCE.md)
4. **Ask on team chat** after completing this phase

---

## Now, Let's Begin

Choose your path above and start with the appropriate guide.

**Most people should start here:**
👉 **[SENTRY_SETUP_README.md](./SENTRY_SETUP_README.md)**

If you want to jump right in:
👉 **[SENTRY_MANUAL_SETUP_GUIDE.md](./SENTRY_MANUAL_SETUP_GUIDE.md)**

For quick lookup:
👉 **[SENTRY_QUICK_REFERENCE.md](../reference/SENTRY_QUICK_REFERENCE.md)**

---

**Questions?** Refer to the appropriate guide above.

**Ready?** Let's set up Sentry! 🚀

