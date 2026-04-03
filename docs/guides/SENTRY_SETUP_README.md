# Sentry Setup — Quick Start

Complete reference for setting up Sentry error tracking across the Phenotype ecosystem.

**Total Time:** ~45 minutes
**Difficulty:** Beginner-friendly (no Sentry experience required)
**What You'll Get:** Production-ready error tracking for 3 Rust projects

---

## Overview

This guide walks you through creating 3 Sentry projects and configuring GitHub organization secrets for:
- **AgilePlus** (Rust backend)
- **phenotype-infrakit** (Rust library)
- **heliosCLI** (Rust CLI)

All errors will be automatically reported to Sentry from your CI/CD pipelines and local development environments.

---

## 3-Phase Process

### Phase 1: Manual Sentry Setup (15 minutes)
Create 3 Sentry projects and obtain DSN tokens.

**See:** `SENTRY_MANUAL_SETUP_GUIDE.md`

### Phase 2: GitHub Secrets Configuration (10 minutes)
Configure organization-level GitHub secrets for CI/CD access.

**See:** `GITHUB_SECRETS_SETUP_GUIDE.md`

### Phase 3: Verification (5 minutes)
Verify everything is configured correctly using the checklist.

**See:** `SENTRY_SETUP_READY_CHECKLIST.md`

---

## Quick Links

| Document | Purpose | Time |
|----------|---------|------|
| [SENTRY_MANUAL_SETUP_GUIDE.md](./SENTRY_MANUAL_SETUP_GUIDE.md) | Step-by-step Sentry project creation | 15 min |
| [GITHUB_SECRETS_SETUP_GUIDE.md](./GITHUB_SECRETS_SETUP_GUIDE.md) | GitHub organization secrets configuration | 10 min |
| [SENTRY_PROJECTS_TEMPLATE.md](./SENTRY_PROJECTS_TEMPLATE.md) | Tracking template for recording project details | 5 min |
| [SENTRY_SETUP_READY_CHECKLIST.md](../reference/SENTRY_SETUP_READY_CHECKLIST.md) | Final verification checklist | 5 min |

---

## Before You Start

### Prerequisites Checklist

- [ ] **Sentry Account:** You have a Sentry.io account (free tier is fine)
  - Sign up: https://sentry.io/
  - No payment required for free tier
- [ ] **GitHub Access:** You have admin or "Maintain" role in the `KooshaPari` organization
  - Verify: https://github.com/organizations/KooshaPari/people
- [ ] **Safe Password Manager:** You have a place to store DSN tokens securely
  - Examples: 1Password, LastPass, Bitwarden, Apple Keychain
- [ ] **Time:** You have ~45 uninterrupted minutes

### What NOT to Do

- ❌ Do NOT commit `.env` files to git (they're already git-ignored)
- ❌ Do NOT share DSN tokens in Slack, email, or any unencrypted channel
- ❌ Do NOT create repository-level secrets (create organization-level instead)
- ❌ Do NOT use the same project for multiple environments (create separate projects)

---

## 30-Second Summary

1. **Create 3 Sentry projects** (one per repo, Rust platform)
2. **Copy DSN tokens** from each project
3. **Create 3 GitHub organization secrets** with those DSNs
4. **Update local `.env` files** with DSN values
5. **Verify** secrets are accessible from each repository

That's it! The next phase will integrate the SDK.

---

## Step-by-Step Overview

### Step 1: Create Sentry Projects

Go to https://sentry.io/ and create 3 projects:

| Project | Platform | DSN Storage |
|---------|----------|-------------|
| AgilePlus | Rust | `SENTRY_DSN_AGILEPLUS` |
| phenotype-infrakit | Rust | `SENTRY_DSN_INFRAKIT` |
| heliosCLI | Rust | `SENTRY_DSN_HELIOSCLI` |

**Full Guide:** `SENTRY_MANUAL_SETUP_GUIDE.md` Part 2-4

### Step 2: Record Project Details

Use `SENTRY_PROJECTS_TEMPLATE.md` to track:
- Project IDs
- DSN URLs
- GitHub secret names
- Creation dates

### Step 3: Create GitHub Secrets

Go to https://github.com/organizations/KooshaPari/settings/secrets/actions and create 3 secrets:

```
Secret Name: SENTRY_DSN_AGILEPLUS
Value: https://your-key@sentry.io/your-project-id
Repositories: AgilePlus

Secret Name: SENTRY_DSN_INFRAKIT
Value: https://your-key@sentry.io/your-project-id
Repositories: phenotype-infrakit

Secret Name: SENTRY_DSN_HELIOSCLI
Value: https://your-key@sentry.io/your-project-id
Repositories: heliosCLI
```

**Full Guide:** `GITHUB_SECRETS_SETUP_GUIDE.md`

### Step 4: Update Local `.env` Files

For each repository:

```bash
# AgilePlus
cp /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.env.example \
   /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.env

# Edit .env and set:
SENTRY_DSN=https://your-key@sentry.io/your-project-id
SENTRY_ENVIRONMENT=development
```

Repeat for phenotype-infrakit and heliosCLI.

### Step 5: Verify Everything

Use `SENTRY_SETUP_READY_CHECKLIST.md` to verify:
- [ ] All 3 Sentry projects exist
- [ ] All 3 GitHub secrets are configured
- [ ] All 3 local `.env` files are updated
- [ ] All secrets are accessible from their repositories

---

## FAQ

### Q: What's a DSN?

**A:** DSN = Data Source Name. It's a unique URL that identifies your Sentry project:
```
https://abc123def456@sentry.io/9876543210
     └────────────┬──────────┘  └──────────┘
      Key (public)       Project ID
```

### Q: Can I use one project for all 3 repos?

**A:** No. Create separate projects so you can:
- Track errors by repo independently
- Set different alert rules per project
- Manage team access per project
- See clean issue lists per product

### Q: What if I lose my DSN?

**A:** You can always retrieve it:
1. Go to Sentry project settings
2. Click **"Client Keys (DSN)"**
3. Copy the DSN again

No problem — it's not a secret like a password.

### Q: Are there costs?

**A:** Free tier includes:
- 5,000 events/month per project
- Up to 7 projects
- Basic features (issue tracking, error grouping)
- No credit card required

Upgrade to paid if you exceed limits.

### Q: How long does setup take?

**A:** ~45 minutes total:
- Sentry projects: 15 min
- GitHub secrets: 10 min
- Local `.env`: 5 min
- Verification: 5 min
- Buffer: 10 min

### Q: Can I undo if I make a mistake?

**A:** Yes! Just:
1. Delete the project in Sentry (Settings → Danger Zone → Delete Project)
2. Delete the GitHub secret (Settings → Secrets → Delete)
3. Start over — no harm done

### Q: What happens after this setup?

**A:** Next phase:
1. Install Sentry Rust SDK in each project
2. Configure error handling to send to Sentry
3. Test with sample errors
4. Set up dashboards and alerts

---

## Documentation Files

### In `/docs/guides/`

1. **SENTRY_MANUAL_SETUP_GUIDE.md** (850+ lines)
   - Complete walkthrough for Sentry setup
   - Screenshots/descriptions for each step
   - Troubleshooting section
   - Verification procedures

2. **GITHUB_SECRETS_SETUP_GUIDE.md** (500+ lines)
   - Step-by-step GitHub configuration
   - Exact navigation paths
   - Verification from each repository
   - Security best practices

3. **SENTRY_PROJECTS_TEMPLATE.md** (200+ lines)
   - Table for recording project details
   - Checklist for each project
   - Verification checklist

### In `/docs/reference/`

4. **SENTRY_SETUP_READY_CHECKLIST.md** (300+ lines)
   - Final verification checklist
   - Sign-off section
   - Summary table
   - Next phase instructions

---

## When You're Done

Once all items in `SENTRY_SETUP_READY_CHECKLIST.md` are checked ✅:

**Notify:** Tell the team: "Sentry setup complete. Ready for SDK integration."

The next phase will:
1. Install `sentry-rs` crate in each project
2. Configure error hooks
3. Set up release tracking
4. Test end-to-end

---

## Support

If you get stuck:

1. **Check the troubleshooting section** in each guide
2. **Verify prerequisites** are met
3. **Double-check URLs** and secret names (case-sensitive)
4. **Review Sentry docs:** https://docs.sentry.io/platforms/rust/
5. **Review GitHub docs:** https://docs.github.com/en/actions/security-guides/encrypted-secrets

---

## Next Steps

### Immediate (Now)

1. Read `SENTRY_MANUAL_SETUP_GUIDE.md`
2. Create 3 Sentry projects
3. Record details in `SENTRY_PROJECTS_TEMPLATE.md`

### Short Term (Today)

1. Read `GITHUB_SECRETS_SETUP_GUIDE.md`
2. Create 3 GitHub organization secrets
3. Verify access from each repository

### Completion (Today)

1. Update local `.env` files
2. Complete `SENTRY_SETUP_READY_CHECKLIST.md`
3. Notify team when done

### Future (Next Phase)

1. SDK integration in each project
2. Error handling configuration
3. Testing and validation

---

## Document Organization

```
docs/
├── guides/
│   ├── SENTRY_SETUP_README.md          ← You are here
│   ├── SENTRY_MANUAL_SETUP_GUIDE.md    ← Step-by-step
│   ├── GITHUB_SECRETS_SETUP_GUIDE.md   ← GitHub config
│   └── SENTRY_PROJECTS_TEMPLATE.md     ← Tracking template
└── reference/
    └── SENTRY_SETUP_READY_CHECKLIST.md ← Final verification
```

---

## Success Criteria

You'll know you're successful when:

✅ All 3 Sentry projects exist and have DSNs
✅ All 3 GitHub secrets are created and verified
✅ Each repository can access its organization secret
✅ Local `.env` files are updated and git-ignored
✅ All items in the checklist are marked complete

---

## Timeline

| Phase | Time | Next |
|-------|------|------|
| **Manual Sentry Setup** | 15 min | Record details |
| **GitHub Secrets Config** | 10 min | Verify access |
| **Local `.env` Update** | 5 min | Complete checklist |
| **Verification** | 5 min | Report completion |
| **TOTAL** | **~45 min** | **SDK Integration** |

---

**You've got this! 🚀**

