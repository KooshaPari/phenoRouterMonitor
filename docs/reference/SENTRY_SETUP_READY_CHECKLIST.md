# Sentry Setup Ready Checklist

Use this checklist to verify that all Sentry projects and GitHub secrets are configured correctly.

**Date Completed:** _______________

---

## Sentry Account Setup

- [ ] Sentry account created (free tier at https://sentry.io/)
- [ ] Organization created and verified
- [ ] Organization slug noted: `_______________`
- [ ] Organization API token generated (optional): `_______________`

---

## Sentry Projects Created

### Project 1: AgilePlus

- [ ] Project created in Sentry
- [ ] Platform selected: **Rust**
- [ ] Project name: `AgilePlus` (or similar)
- [ ] Project ID: `_______________`
- [ ] DSN obtained: `https://_______________@sentry.io/_______________`
- [ ] GitHub integration enabled
- [ ] Alert frequency set to "Every issue"
- [ ] DSN stored securely (password manager, not git)

**Status:** ⏳ / ✅

### Project 2: phenotype-infrakit

- [ ] Project created in Sentry
- [ ] Platform selected: **Rust**
- [ ] Project name: `phenotype-infrakit` (or similar)
- [ ] Project ID: `_______________`
- [ ] DSN obtained: `https://_______________@sentry.io/_______________`
- [ ] GitHub integration enabled
- [ ] Alert frequency set to "Every issue"
- [ ] DSN stored securely (password manager, not git)

**Status:** ⏳ / ✅

### Project 3: heliosCLI

- [ ] Project created in Sentry
- [ ] Platform selected: **Rust**
- [ ] Project name: `heliosCLI` (or similar)
- [ ] Project ID: `_______________`
- [ ] DSN obtained: `https://_______________@sentry.io/_______________`
- [ ] GitHub integration enabled
- [ ] Alert frequency set to "Every issue"
- [ ] DSN stored securely (password manager, not git)

**Status:** ⏳ / ✅

---

## GitHub Secrets Configuration

### Organization Secrets Created

- [ ] Navigated to: `https://github.com/organizations/KooshaPari/settings/secrets/actions`
- [ ] `SENTRY_DSN_AGILEPLUS` secret created with full DSN value
- [ ] `SENTRY_DSN_INFRAKIT` secret created with full DSN value
- [ ] `SENTRY_DSN_HELIOSCLI` secret created with full DSN value

### Repository Access Verified

**AgilePlus Repository**
- [ ] `https://github.com/KooshaPari/AgilePlus/settings/secrets/actions` opened
- [ ] `SENTRY_DSN_AGILEPLUS` visible in organization secrets
- [ ] Status shows: "Available"

**phenotype-infrakit Repository**
- [ ] `https://github.com/KooshaPari/phenotype-infrakit/settings/secrets/actions` opened
- [ ] `SENTRY_DSN_INFRAKIT` visible in organization secrets
- [ ] Status shows: "Available"

**heliosCLI Repository**
- [ ] `https://github.com/KooshaPari/heliosCLI/settings/secrets/actions` opened
- [ ] `SENTRY_DSN_HELIOSCLI` visible in organization secrets
- [ ] Status shows: "Available"

---

## Local Environment Configuration

### AgilePlus

- [ ] `.env` file created from `.env.example`
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] File location: `/Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus/.env`
- [ ] File is in `.gitignore`: ✅
- [ ] Content verified (not committed to git): ✅

### phenotype-infrakit

- [ ] `.env` file created from `.env.example`
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] File location: `/Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit/.env`
- [ ] File is in `.gitignore`: ✅
- [ ] Content verified (not committed to git): ✅

### heliosCLI

- [ ] `.env` file created from `.env.example`
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] File location: `/Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI/.env`
- [ ] File is in `.gitignore`: ✅
- [ ] Content verified (not committed to git): ✅

---

## Documentation & Reference

- [ ] `SENTRY_MANUAL_SETUP_GUIDE.md` completed
- [ ] `SENTRY_PROJECTS_TEMPLATE.md` filled with all project details
- [ ] `GITHUB_SECRETS_SETUP_GUIDE.md` referenced for secret creation
- [ ] This checklist signed and dated

---

## Security Verification

- [ ] DSNs stored in password manager (NOT in git)
- [ ] No `.env` files committed to repositories
- [ ] All `.env` files in `.gitignore`
- [ ] GitHub secrets are organization-level (not repository-level)
- [ ] Secrets scoped to specific repositories (not "all repositories")
- [ ] No secrets logged or printed in CI/CD workflows

---

## Final Readiness

All items above are checked ✅?

**YES → READY FOR SDK FINALIZATION PHASE**

**NO → Complete missing items above and recheck**

---

## Completion Summary

| Component | Status |
|-----------|--------|
| Sentry Projects | ⏳ / ✅ |
| GitHub Secrets | ⏳ / ✅ |
| Local `.env` Files | ⏳ / ✅ |
| Documentation | ⏳ / ✅ |
| Security Verification | ⏳ / ✅ |
| **OVERALL READINESS** | **⏳ / ✅** |

---

## Next Phase: SDK Finalization

Once this checklist is complete, the next phase will:

1. **Install Sentry SDK** in each project's Rust code
2. **Configure error handling** to send errors to Sentry
3. **Set up release tracking** for version management
4. **Enable breadcrumbs** for debugging context
5. **Configure environment-specific settings** (dev, staging, production)
6. **Write integration tests** to verify data flow
7. **Document troubleshooting** procedures

---

## Sign-Off

**Completed By:** _______________

**Date:** _______________

**Notes:**

```
-
-
-
```

---

## Report to Team

Once complete, notify the team:

**Message Template:**
```
✅ All 3 Sentry projects created and configured:
   - AgilePlus (Project ID: ___________)
   - phenotype-infrakit (Project ID: ___________)
   - heliosCLI (Project ID: ___________)

✅ GitHub organization secrets configured and verified:
   - SENTRY_DSN_AGILEPLUS
   - SENTRY_DSN_INFRAKIT
   - SENTRY_DSN_HELIOSCLI

✅ Local .env files updated and secured

Ready for SDK finalization phase.
```

