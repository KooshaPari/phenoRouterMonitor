# Sentry Quick Reference

Quick lookup for Sentry DSN tokens and GitHub secrets.

---

## Sentry Projects Summary

| Project | Repository | Platform | GitHub Secret | Status |
|---------|------------|----------|---------------|--------|
| AgilePlus | `KooshaPari/AgilePlus` | Rust | `SENTRY_DSN_AGILEPLUS` | ⏳ |
| phenotype-infrakit | `KooshaPari/phenotype-infrakit` | Rust | `SENTRY_DSN_INFRAKIT` | ⏳ |
| heliosCLI | `KooshaPari/heliosCLI` | Rust | `SENTRY_DSN_HELIOSCLI` | ⏳ |

---

## DSN Storage

| Project | DSN | Notes |
|---------|-----|-------|
| AgilePlus | `https://...@sentry.io/...` | Stored securely |
| phenotype-infrakit | `https://...@sentry.io/...` | Stored securely |
| heliosCLI | `https://...@sentry.io/...` | Stored securely |

---

## GitHub Secret Names (Case-Sensitive)

```
SENTRY_DSN_AGILEPLUS
SENTRY_DSN_INFRAKIT
SENTRY_DSN_HELIOSCLI
```

---

## Key URLs

| Purpose | URL |
|---------|-----|
| Sentry Dashboard | https://sentry.io/ |
| Create New Project | https://sentry.io/projects/new/ |
| GitHub Org Secrets | https://github.com/organizations/KooshaPari/settings/secrets/actions |
| AgilePlus Repo | https://github.com/KooshaPari/AgilePlus |
| phenotype-infrakit Repo | https://github.com/KooshaPari/phenotype-infrakit |
| heliosCLI Repo | https://github.com/KooshaPari/heliosCLI |

---

## Local `.env` Paths

```
/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.env
/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit/.env
/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/.env
```

---

## `.env` Template

```bash
# Sentry Error Tracking Configuration
SENTRY_DSN=https://your-key@sentry.io/your-project-id
SENTRY_ENVIRONMENT=development
# SENTRY_RELEASE=0.1.0
# RUST_LOG=sentry=debug
```

---

## Sentry Project Setup Checklist

- [ ] Create project in Sentry
- [ ] Select Rust platform
- [ ] Copy DSN token
- [ ] Store DSN securely (password manager)
- [ ] Note Project ID
- [ ] Enable GitHub integration
- [ ] Set alert frequency to "Every issue"

---

## GitHub Secrets Setup Checklist

- [ ] Go to: https://github.com/organizations/KooshaPari/settings/secrets/actions
- [ ] Create secret: `SENTRY_DSN_AGILEPLUS` → assign to `AgilePlus`
- [ ] Create secret: `SENTRY_DSN_INFRAKIT` → assign to `phenotype-infrakit`
- [ ] Create secret: `SENTRY_DSN_HELIOSCLI` → assign to `heliosCLI`
- [ ] Verify each secret is visible from its repository

---

## Local `.env` Setup Checklist

- [ ] Copy `.env.example` to `.env` (for each repo)
- [ ] Set `SENTRY_DSN` to your project DSN
- [ ] Set `SENTRY_ENVIRONMENT=development`
- [ ] Verify `.env` is in `.gitignore`
- [ ] Do NOT commit `.env` to git

---

## Troubleshooting Quick Links

| Issue | Solution |
|-------|----------|
| Can't find secrets tab | Go to Org Settings (not repo), click "Secrets and variables" |
| Secret not visible in repo | Verify repo was added to the secret's access list |
| DSN format seems wrong | Copy directly from Sentry project settings |
| Workflow can't access secret | Check secret name is exact match (case-sensitive) |

---

## Next Phase: SDK Integration

Once setup is complete:

1. Install Sentry SDK: `sentry-rs` crate
2. Configure in each project's `Cargo.toml`
3. Initialize in application startup
4. Add error hooks to send errors to Sentry
5. Test with sample errors

---

## Documentation Index

| Document | Purpose |
|----------|---------|
| `SENTRY_SETUP_README.md` | Getting started guide |
| `SENTRY_MANUAL_SETUP_GUIDE.md` | Detailed Sentry project setup |
| `GITHUB_SECRETS_SETUP_GUIDE.md` | GitHub secrets configuration |
| `SENTRY_PROJECTS_TEMPLATE.md` | Tracking template |
| `SENTRY_SETUP_READY_CHECKLIST.md` | Final verification |
| `SENTRY_QUICK_REFERENCE.md` | This file |

---

## Quick Start

1. **Go to:** https://sentry.io/
2. **Create:** 3 projects (AgilePlus, phenotype-infrakit, heliosCLI)
3. **Copy:** DSN for each project
4. **Go to:** https://github.com/organizations/KooshaPari/settings/secrets/actions
5. **Create:** 3 secrets with DSN values
6. **Update:** Local `.env` files
7. **Done:** Ready for SDK integration

---

## Support Resources

- Sentry Docs: https://docs.sentry.io/
- Sentry Rust Guide: https://docs.sentry.io/platforms/rust/
- GitHub Secrets: https://docs.github.com/en/actions/security-guides/encrypted-secrets
- This Documentation: `/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/`

