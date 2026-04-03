# Sentry Setup Documentation Index

Complete guide index for setting up error tracking across the Phenotype ecosystem.

---

## Quick Navigation

### Getting Started (Start Here)
👉 **[SENTRY_SETUP_README.md](./SENTRY_SETUP_README.md)** — High-level overview and quick start guide

### Detailed Guides

1. **[SENTRY_MANUAL_SETUP_GUIDE.md](./SENTRY_MANUAL_SETUP_GUIDE.md)** — Complete Sentry project creation walkthrough
   - Account setup
   - Create 3 projects (AgilePlus, phenotype-infrakit, heliosCLI)
   - Retrieve DSN tokens
   - Enable integrations
   - Troubleshooting

2. **[GITHUB_SECRETS_SETUP_GUIDE.md](./GITHUB_SECRETS_SETUP_GUIDE.md)** — GitHub organization secrets configuration
   - Navigate to org settings
   - Create 3 organization secrets
   - Verify secret accessibility
   - Usage in CI/CD workflows
   - Security best practices

3. **[SENTRY_PROJECTS_TEMPLATE.md](./SENTRY_PROJECTS_TEMPLATE.md)** — Tracking template
   - Record project details as you create them
   - Detailed per-project checklist
   - Verification checklist

### Verification & Reference

4. **[SENTRY_SETUP_READY_CHECKLIST.md](../reference/SENTRY_SETUP_READY_CHECKLIST.md)** — Final readiness checklist
   - Verify all 3 projects created
   - Verify all 3 GitHub secrets configured
   - Verify local `.env` files updated
   - Sign-off section
   - Next phase instructions

5. **[SENTRY_QUICK_REFERENCE.md](../reference/SENTRY_QUICK_REFERENCE.md)** — Quick lookup guide
   - Project summary table
   - Key URLs
   - Local `.env` paths
   - Quick checklists
   - Troubleshooting quick links

---

## Document Structure

### In `/docs/guides/`

```
guides/
├── SENTRY_SETUP_README.md              # Start here (30 min)
├── SENTRY_MANUAL_SETUP_GUIDE.md        # Phase 1 (15 min)
├── GITHUB_SECRETS_SETUP_GUIDE.md       # Phase 2 (10 min)
├── SENTRY_PROJECTS_TEMPLATE.md         # Tracking template
└── SENTRY_SETUP_INDEX.md               # This file
```

### In `/docs/reference/`

```
reference/
├── SENTRY_SETUP_READY_CHECKLIST.md     # Phase 3 (5 min)
├── SENTRY_QUICK_REFERENCE.md           # Lookup guide
└── (other reference docs)
```

---

## Learning Paths

### For First-Time Users

**Recommended order:**
1. Read **SENTRY_SETUP_README.md** (5 min)
2. Follow **SENTRY_MANUAL_SETUP_GUIDE.md** (15 min)
3. Use **SENTRY_PROJECTS_TEMPLATE.md** to track progress (5 min)
4. Follow **GITHUB_SECRETS_SETUP_GUIDE.md** (10 min)
5. Complete **SENTRY_SETUP_READY_CHECKLIST.md** (5 min)
6. Keep **SENTRY_QUICK_REFERENCE.md** handy for lookups

**Total time:** ~45 minutes

### For Experienced Users (Already know Sentry)

1. Skim **SENTRY_SETUP_README.md** for context
2. Jump to **SENTRY_PROJECTS_TEMPLATE.md** to track details
3. Use **SENTRY_QUICK_REFERENCE.md** for URLs and secret names

**Total time:** ~20 minutes

### For GitHub Admins (Handling Secrets)

1. Read **GITHUB_SECRETS_SETUP_GUIDE.md** thoroughly
2. Reference **SENTRY_QUICK_REFERENCE.md** for secret names
3. Use **SENTRY_SETUP_READY_CHECKLIST.md** to verify

**Total time:** ~15 minutes

---

## By Task

### Task: "Create Sentry Projects"
→ **SENTRY_MANUAL_SETUP_GUIDE.md** (Parts 1-4)

### Task: "Set Up GitHub Secrets"
→ **GITHUB_SECRETS_SETUP_GUIDE.md** (All sections)

### Task: "Update Local `.env` Files"
→ **SENTRY_MANUAL_SETUP_GUIDE.md** (Part 7)

### Task: "Verify Everything Works"
→ **SENTRY_SETUP_READY_CHECKLIST.md** (All sections)

### Task: "Find a URL or Secret Name"
→ **SENTRY_QUICK_REFERENCE.md** (Any section)

### Task: "Troubleshoot an Issue"
→ **SENTRY_MANUAL_SETUP_GUIDE.md** (Troubleshooting)
→ **GITHUB_SECRETS_SETUP_GUIDE.md** (Troubleshooting)
→ **SENTRY_QUICK_REFERENCE.md** (Troubleshooting Quick Links)

---

## File Descriptions

### SENTRY_SETUP_README.md
**Type:** Overview & Quick Start
**Length:** ~400 lines
**Purpose:** High-level introduction to the 3-phase setup process
**Covers:**
- What you'll do in each phase
- Prerequisites checklist
- 30-second summary
- Step-by-step overview
- FAQ
- Links to detailed guides

### SENTRY_MANUAL_SETUP_GUIDE.md
**Type:** Detailed Walkthrough
**Length:** ~850 lines
**Purpose:** Complete step-by-step guide for creating Sentry projects
**Covers:**
- Account setup (one-time)
- Create 3 projects (AgilePlus, phenotype-infrakit, heliosCLI)
- Retrieve DSN tokens
- Record project details
- Update local `.env` files
- Verification checklist
- Troubleshooting section

### GITHUB_SECRETS_SETUP_GUIDE.md
**Type:** Detailed Walkthrough
**Length:** ~500 lines
**Purpose:** Complete step-by-step guide for GitHub organization secrets
**Covers:**
- Prerequisites and navigation checklist
- Add 3 secrets (step-by-step)
- Verify secrets are accessible from each repo
- Usage in CI/CD workflows
- Troubleshooting section
- Security best practices

### SENTRY_PROJECTS_TEMPLATE.md
**Type:** Tracking Template & Checklist
**Length:** ~200 lines
**Purpose:** Record project details as you create them
**Covers:**
- Summary table for all 3 projects
- Detailed checklist per project
- Overall verification checklist
- Notes section

### SENTRY_SETUP_READY_CHECKLIST.md
**Type:** Verification Checklist
**Length:** ~300 lines
**Purpose:** Final verification before moving to next phase
**Covers:**
- Sentry account setup verification
- All 3 projects created and verified
- All 3 GitHub secrets configured and verified
- Local `.env` files updated and secured
- Security verification
- Sign-off section
- Next phase instructions

### SENTRY_QUICK_REFERENCE.md
**Type:** Quick Lookup Reference
**Length:** ~200 lines
**Purpose:** Fast reference for key information
**Covers:**
- Projects summary table
- DSN storage table
- GitHub secret names
- Key URLs
- Local `.env` paths
- Quick checklists
- Troubleshooting quick links

---

## Setup Timeline

| Phase | Documents | Time | Outcome |
|-------|-----------|------|---------|
| **Phase 1: Manual Setup** | SENTRY_SETUP_README.md, SENTRY_MANUAL_SETUP_GUIDE.md | 15 min | 3 Sentry projects created, 3 DSN tokens obtained |
| **Phase 2: GitHub Secrets** | GITHUB_SECRETS_SETUP_GUIDE.md, SENTRY_QUICK_REFERENCE.md | 10 min | 3 GitHub organization secrets created & verified |
| **Phase 3: Verification** | SENTRY_SETUP_READY_CHECKLIST.md | 5 min | All items verified, sign-off complete |
| **Total** | **All documents** | **~45 min** | **Ready for SDK integration** |

---

## Key Information Snippets

### Sentry Projects

```
Project 1: AgilePlus
  Platform: Rust
  Secret Name: SENTRY_DSN_AGILEPLUS

Project 2: phenotype-infrakit
  Platform: Rust
  Secret Name: SENTRY_DSN_INFRAKIT

Project 3: heliosCLI
  Platform: Rust
  Secret Name: SENTRY_DSN_HELIOSCLI
```

### GitHub Secret Names (Case-Sensitive)

```
SENTRY_DSN_AGILEPLUS
SENTRY_DSN_INFRAKIT
SENTRY_DSN_HELIOSCLI
```

### Local `.env` Template

```bash
SENTRY_DSN=https://your-key@sentry.io/your-project-id
SENTRY_ENVIRONMENT=development
```

### Key URLs

```
Sentry: https://sentry.io/
GitHub Secrets: https://github.com/organizations/KooshaPari/settings/secrets/actions
AgilePlus: https://github.com/KooshaPari/AgilePlus
phenotype-infrakit: https://github.com/KooshaPari/phenotype-infrakit
heliosCLI: https://github.com/KooshaPari/heliosCLI
```

---

## Common Questions

### Q: Where do I start?

**A:** Read **SENTRY_SETUP_README.md** first. It's a 5-minute overview.

### Q: Where's the step-by-step guide?

**A:** **SENTRY_MANUAL_SETUP_GUIDE.md** has the complete walkthrough for Sentry projects.

### Q: How do I set up GitHub secrets?

**A:** **GITHUB_SECRETS_SETUP_GUIDE.md** has the complete GitHub configuration guide.

### Q: What are the GitHub secret names?

**A:** Check **SENTRY_QUICK_REFERENCE.md** for the exact names (case-sensitive).

### Q: How do I verify everything works?

**A:** **SENTRY_SETUP_READY_CHECKLIST.md** has the complete verification checklist.

### Q: I'm stuck. Where's the troubleshooting?

**A:** See troubleshooting sections in:
- **SENTRY_MANUAL_SETUP_GUIDE.md** (Sentry issues)
- **GITHUB_SECRETS_SETUP_GUIDE.md** (GitHub issues)
- **SENTRY_QUICK_REFERENCE.md** (Quick troubleshooting links)

### Q: What's next after setup?

**A:** See **SENTRY_SETUP_READY_CHECKLIST.md** → "Next Phase: SDK Finalization"

---

## Document Dependencies

```
SENTRY_SETUP_README.md
├── SENTRY_MANUAL_SETUP_GUIDE.md
│   └── SENTRY_PROJECTS_TEMPLATE.md
├── GITHUB_SECRETS_SETUP_GUIDE.md
│   └── SENTRY_QUICK_REFERENCE.md
└── SENTRY_SETUP_READY_CHECKLIST.md
```

All documents are standalone and can be read independently, but the order above is recommended.

---

## File Sizes

| File | Lines | Approx. Size | Read Time |
|------|-------|--------------|-----------|
| SENTRY_SETUP_README.md | 400 | 14 KB | 5 min |
| SENTRY_MANUAL_SETUP_GUIDE.md | 850 | 30 KB | 15 min |
| GITHUB_SECRETS_SETUP_GUIDE.md | 500 | 18 KB | 10 min |
| SENTRY_PROJECTS_TEMPLATE.md | 200 | 8 KB | 3 min |
| SENTRY_SETUP_READY_CHECKLIST.md | 300 | 11 KB | 5 min |
| SENTRY_QUICK_REFERENCE.md | 200 | 7 KB | 2 min |
| **TOTAL** | **2,450** | **88 KB** | **~45 min** |

---

## Editing & Updates

These documents should be updated when:

- Sentry UI changes (update screenshots/descriptions)
- GitHub UI changes (update navigation instructions)
- New projects are added (update templates and checklists)
- New GitHub features become available
- Feedback indicates sections are confusing

### Location

```
/Users/kooshapari/CodeProjects/Phenotype/repos/docs/guides/
```

### Maintenance

- Review annually or when tools change
- Update based on user feedback
- Keep examples current with latest versions

---

## Next Phase Documentation

Once Sentry setup is complete, the next phase will have documentation for:

1. **SDK Installation Guide** — Installing sentry-rs in Rust projects
2. **Error Handling Integration** — Configuring error hooks
3. **Release Tracking** — Managing versions in Sentry
4. **Testing Guide** — Verifying error capture
5. **Monitoring & Dashboards** — Setting up alerts and views

---

## Success Metrics

You've successfully completed Sentry setup when:

✅ All items in **SENTRY_SETUP_READY_CHECKLIST.md** are marked complete

Once that's done:
→ Move to next phase: SDK integration

---

## Contact & Support

For questions about:
- **Sentry:** See https://docs.sentry.io/ or https://forum.sentry.io/
- **GitHub Secrets:** See https://docs.github.com/en/actions/security-guides/encrypted-secrets
- **These Guides:** Check troubleshooting sections or ask the team

---

**Last Updated:** 2026-03-30

