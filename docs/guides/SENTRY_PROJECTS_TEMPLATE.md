# Sentry Projects Template

Use this template to record all Sentry project details as you create them.

**Purpose:** Tracking artifact to ensure all 3 projects are created and configured correctly.

---

## Project Recording Table

Fill in this table as you create each Sentry project:

| Project Name | Platform | Project ID | DSN | GitHub Secret Name | Created Date | Status |
|---|---|---|---|---|---|---|
| AgilePlus | Rust | | | SENTRY_DSN_AGILEPLUS | | ⏳ Pending |
| phenotype-infrakit | Rust | | | SENTRY_DSN_INFRAKIT | | ⏳ Pending |
| heliosCLI | Rust | | | SENTRY_DSN_HELIOSCLI | | ⏳ Pending |

---

## Detailed Project Information

### Project 1: AgilePlus

**Project Setup**
- [ ] Project created in Sentry
- [ ] Platform selected: Rust
- [ ] Project ID: _______________
- [ ] DSN retrieved: _______________
- [ ] GitHub Integration enabled
- [ ] Alert rules configured

**GitHub Configuration**
- [ ] Secret created: `SENTRY_DSN_AGILEPLUS`
- [ ] Secret value: `https://...` (stored securely)
- [ ] Repository assigned: `AgilePlus`
- [ ] Secret accessible from repo: ✅ / ❌

**Local Configuration**
- [ ] `.env` file created (from `.env.example`)
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] `.env` is git-ignored

**Status:** ⏳ / ✅

---

### Project 2: phenotype-infrakit

**Project Setup**
- [ ] Project created in Sentry
- [ ] Platform selected: Rust
- [ ] Project ID: _______________
- [ ] DSN retrieved: _______________
- [ ] GitHub Integration enabled
- [ ] Alert rules configured

**GitHub Configuration**
- [ ] Secret created: `SENTRY_DSN_INFRAKIT`
- [ ] Secret value: `https://...` (stored securely)
- [ ] Repository assigned: `phenotype-infrakit`
- [ ] Secret accessible from repo: ✅ / ❌

**Local Configuration**
- [ ] `.env` file created (from `.env.example`)
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] `.env` is git-ignored

**Status:** ⏳ / ✅

---

### Project 3: heliosCLI

**Project Setup**
- [ ] Project created in Sentry
- [ ] Platform selected: Rust
- [ ] Project ID: _______________
- [ ] DSN retrieved: _______________
- [ ] GitHub Integration enabled
- [ ] Alert rules configured

**GitHub Configuration**
- [ ] Secret created: `SENTRY_DSN_HELIOSCLI`
- [ ] Secret value: `https://...` (stored securely)
- [ ] Repository assigned: `heliosCLI`
- [ ] Secret accessible from repo: ✅ / ❌

**Local Configuration**
- [ ] `.env` file created (from `.env.example`)
- [ ] `SENTRY_DSN` set to project DSN
- [ ] `SENTRY_ENVIRONMENT` set to `development`
- [ ] `.env` is git-ignored

**Status:** ⏳ / ✅

---

## Verification Checklist

### All Projects Created

- [ ] AgilePlus project exists
- [ ] phenotype-infrakit project exists
- [ ] heliosCLI project exists

### All DSNs Retrieved

- [ ] AgilePlus DSN stored securely
- [ ] phenotype-infrakit DSN stored securely
- [ ] heliosCLI DSN stored securely

### All GitHub Secrets Configured

- [ ] `SENTRY_DSN_AGILEPLUS` secret created
- [ ] `SENTRY_DSN_INFRAKIT` secret created
- [ ] `SENTRY_DSN_HELIOSCLI` secret created

### All Local `.env` Files Updated

- [ ] AgilePlus `.env` configured
- [ ] phenotype-infrakit `.env` configured
- [ ] heliosCLI `.env` configured

### Security

- [ ] DSNs stored in password manager (NOT in git)
- [ ] `.env` files in `.gitignore`
- [ ] GitHub Secrets are organization-level (not repository-level)

---

## Notes

Use this section to record any issues, special notes, or deviations from the standard process:

```
Notes:
-
-
-
```

---

## Instructions for Completion

1. **As you create each project**, fill in the Project ID and DSN in the table above
2. **Check off boxes** as each step is completed
3. **Update Status column** with ✅ when all steps for a project are done
4. **When all 3 projects are complete**, the overall status should show: **READY FOR SDK FINALIZATION**

---

## When to Report Completion

Once all sections show ✅ or checked checkboxes:

**Tell the team:** "All 3 Sentry projects are created and configured. Ready for SDK finalization phase."

The next phase will integrate the Sentry SDK into each project using these DSN tokens.

