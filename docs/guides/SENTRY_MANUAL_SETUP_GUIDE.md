# Sentry Manual Setup Guide

Complete walkthrough for creating 3 Sentry projects and configuring GitHub organization secrets.

**Estimated Time:** 45 minutes (15 minutes per project)

**What You'll Do:**
1. Create 3 Sentry projects (AgilePlus, phenotype-infrakit, heliosCLI)
2. Obtain DSN tokens for each project
3. Record all details in a tracking template
4. Configure GitHub organization secrets
5. Verify everything works

---

## Prerequisites

- **Sentry Account:** Create free at https://sentry.io/ if you don't have one
- **GitHub Organization Access:** Must have "Admin" or "Maintain" role
- **Safe Location:** Keep DSN tokens in a password manager or secure note (do NOT commit to repo)

---

## Part 1: Sentry Account Setup (One-Time)

### Step 1: Sign Up for Sentry (if needed)

1. Go to https://sentry.io/
2. Click **"Start for Free"**
3. Sign up with GitHub (recommended) or email
4. Verify email if required
5. Complete organization setup

### Step 2: Find Your Organization Slug

Once logged in:
1. Go to **Settings** (gear icon, top right)
2. Select your **Organization**
3. Copy the **Organization Slug** from the URL: `https://sentry.io/settings/{ORG_SLUG}/`
   - Example: `my-company` or `phenotype-org`
   - **Save this** — you'll need it for all 3 projects

### Step 3: Generate Organization API Token (Optional but Recommended)

For automated integrations later:

1. Go to **Settings → Auth Tokens**
2. Click **Create New Token**
3. Select scopes: `project:read`, `project:write`, `org:read`
4. Copy the token
5. Store in password manager as: `SENTRY_ORG_API_TOKEN`

---

## Part 2: Create First Project (AgilePlus)

### Step 1: Start New Project

1. In Sentry, click **Projects** (top nav)
2. Click **Create Project**
3. You'll see the **"Create a new project"** dialog

### Step 2: Select Platform

1. Select **Rust** from the platform list
2. Click **Next**
3. You'll see environment options

### Step 3: Configure Project

1. **Project Name:** `AgilePlus` (or `agileplus`)
2. **Alert Frequency:** Choose `Every issue` (default is fine)
3. Click **Create Project**

### Step 4: Retrieve DSN Token

You'll now see the **Setup Instructions** page with your DSN:

1. Look for the line starting with `SENTRY_DSN=`
2. **Copy the entire URL** (format: `https://key@sentry.io/project-id`)
3. Store in password manager as: `SENTRY_DSN_AGILEPLUS`
4. Also note the **Project ID** (number in the URL)

**Example:**
```
SENTRY_DSN=https://abc123def456@sentry.io/9876543210
Project ID: 9876543210
```

### Step 5: Complete Setup (Optional)

1. Scroll down on the setup page
2. Review the Rust SDK integration example (ignore for now)
3. Click **"Skip onboarding"** or **"Go to Issues"**

### Step 6: Enable Additional Features (Recommended)

1. Go to **Settings** (gear icon in top-right corner of project)
2. Click **Integrations**
3. Enable:
   - **GitHub** (for issue linking and commit tracking)
   - **Slack** (if you use Slack for alerts)

---

## Part 3: Create Second Project (phenotype-infrakit)

Repeat the process from **Part 2** with these changes:

1. **Project Name:** `phenotype-infrakit` (or `infrakit`)
2. **Platform:** Rust (same as before)
3. **Alert Frequency:** `Every issue`
4. **DSN Storage:** `SENTRY_DSN_INFRAKIT`

**Copy the DSN and note the Project ID.**

---

## Part 4: Create Third Project (heliosCLI)

Repeat the process from **Part 2** with these changes:

1. **Project Name:** `heliosCLI` (or `helioscli`)
2. **Platform:** Rust (same as before)
3. **Alert Frequency:** `Every issue`
4. **DSN Storage:** `SENTRY_DSN_HELIOSCLI`

**Copy the DSN and note the Project ID.**

---

## Part 5: Record Project Details

Use the **SENTRY_PROJECTS_TEMPLATE.md** to record all details:

| Project Name | Platform | Project ID | DSN | GitHub Secret Name | Status |
|---|---|---|---|---|---|
| AgilePlus | Rust | 9876543210 | https://abc123@sentry.io/9876543210 | SENTRY_DSN_AGILEPLUS | ✅ Created |
| phenotype-infrakit | Rust | 9876543211 | https://def456@sentry.io/9876543211 | SENTRY_DSN_INFRAKIT | ✅ Created |
| heliosCLI | Rust | 9876543212 | https://ghi789@sentry.io/9876543212 | SENTRY_DSN_HELIOSCLI | ✅ Created |

---

## Part 6: Configure GitHub Organization Secrets

### Overview

GitHub Secrets are encrypted environment variables available to all workflows in your organization. We'll store the 3 DSN tokens as organization secrets.

### Prerequisites

- Must have **Organization admin** or **Maintain** role
- Organization: `KooshaPari` (verify this is correct)

### Step 1: Navigate to Organization Settings

1. Go to your GitHub Organization: https://github.com/organizations/KooshaPari/settings/
2. Click **Settings** (left sidebar)
3. Click **Secrets and variables** (left sidebar, under "Security")
4. Click **Actions** (you should see three tabs: Secrets, Variables, Dependabot)

You should now see the **"Secrets"** tab with a list of existing secrets (if any).

### Step 2: Add First Secret (SENTRY_DSN_AGILEPLUS)

1. Click **"New organization secret"** (green button, top right)
2. **Name:** `SENTRY_DSN_AGILEPLUS`
3. **Value:** Paste the DSN you copied from Sentry
   - Example: `https://abc123def456@sentry.io/9876543210`
4. **Repository access:** Select **"Selected repositories"**
5. **Add repositories:** Choose:
   - `AgilePlus`
   - (or search and select by name)
6. Click **"Add secret"**

**Verification:** You should see `SENTRY_DSN_AGILEPLUS` in the Secrets list.

### Step 3: Add Second Secret (SENTRY_DSN_INFRAKIT)

1. Click **"New organization secret"** (green button)
2. **Name:** `SENTRY_DSN_INFRAKIT`
3. **Value:** Paste the DSN for phenotype-infrakit
4. **Repository access:** Select **"Selected repositories"**
5. **Add repositories:** Choose:
   - `phenotype-infrakit`
6. Click **"Add secret"**

**Verification:** You should see `SENTRY_DSN_INFRAKIT` in the Secrets list.

### Step 4: Add Third Secret (SENTRY_DSN_HELIOSCLI)

1. Click **"New organization secret"** (green button)
2. **Name:** `SENTRY_DSN_HELIOSCLI`
3. **Value:** Paste the DSN for heliosCLI
4. **Repository access:** Select **"Selected repositories"**
5. **Add repositories:** Choose:
   - `heliosCLI`
6. Click **"Add secret"**

**Verification:** You should see `SENTRY_DSN_HELIOSCLI` in the Secrets list.

### Step 5: Verify Secrets Are Accessible

For each repository, verify that CI workflows can access the secret:

#### For AgilePlus:

1. Go to https://github.com/KooshaPari/AgilePlus
2. Click **Settings** (top nav)
3. Click **Secrets and variables → Actions** (left sidebar)
4. You should see **SENTRY_DSN_AGILEPLUS** in the organization secrets list

#### For phenotype-infrakit:

1. Go to https://github.com/KooshaPari/phenotype-infrakit
2. Click **Settings** (top nav)
3. Click **Secrets and variables → Actions** (left sidebar)
4. You should see **SENTRY_DSN_INFRAKIT** in the organization secrets list

#### For heliosCLI:

1. Go to https://github.com/KooshaPari/heliosCLI
2. Click **Settings** (top nav)
3. Click **Secrets and variables → Actions** (left sidebar)
4. You should see **SENTRY_DSN_HELIOSCLI** in the organization secrets list

---

## Part 7: Update Local `.env` Files

Each repository already has a `.env.example` file. Now update your local `.env`:

### For AgilePlus

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/AgilePlus
cp .env.example .env
```

Edit `.env`:
```bash
SENTRY_DSN=https://abc123def456@sentry.io/9876543210
SENTRY_ENVIRONMENT=development
```

### For phenotype-infrakit

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/phenotype-infrakit
cp .env.example .env
```

Edit `.env`:
```bash
SENTRY_DSN=https://def456ghi789@sentry.io/9876543211
SENTRY_ENVIRONMENT=development
```

### For heliosCLI

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos/heliosCLI
cp .env.example .env
```

Edit `.env`:
```bash
SENTRY_DSN=https://ghi789jkl012@sentry.io/9876543212
SENTRY_ENVIRONMENT=development
```

**Important:** Each `.env` file is **git-ignored** and will NOT be committed. Keep it local.

---

## Verification Checklist

After completing all steps, verify everything works:

### Sentry Project Verification

- [ ] **AgilePlus** project exists in Sentry
  - Project ID: ___________
  - DSN stored securely: ✅
- [ ] **phenotype-infrakit** project exists in Sentry
  - Project ID: ___________
  - DSN stored securely: ✅
- [ ] **heliosCLI** project exists in Sentry
  - Project ID: ___________
  - DSN stored securely: ✅

### GitHub Secrets Verification

- [ ] Organization secret `SENTRY_DSN_AGILEPLUS` exists
  - Accessible from AgilePlus repository: ✅
- [ ] Organization secret `SENTRY_DSN_INFRAKIT` exists
  - Accessible from phenotype-infrakit repository: ✅
- [ ] Organization secret `SENTRY_DSN_HELIOSCLI` exists
  - Accessible from heliosCLI repository: ✅

### Local Environment Verification

- [ ] AgilePlus `.env` file exists with SENTRY_DSN set
- [ ] phenotype-infrakit `.env` file exists with SENTRY_DSN set
- [ ] heliosCLI `.env` file exists with SENTRY_DSN set
- [ ] All `.env` files are in `.gitignore` (should be already)

---

## Troubleshooting

### Problem: Cannot Find "Secrets and variables" in GitHub Settings

**Solution:**
1. Make sure you're in the Organization settings, not a repository
2. The correct URL is: `https://github.com/organizations/{ORG_NAME}/settings/`
3. Click **"Secrets and variables"** in the left sidebar (under "Security")

### Problem: DSN Format Seems Wrong

**Solution:**
- DSN format is always: `https://{KEY}@sentry.io/{PROJECT_ID}`
- Copy from Sentry project settings directly
- Check that there are no extra spaces or line breaks

### Problem: Secret Not Visible in Repository

**Solution:**
1. Go to the repository settings
2. Verify the secret was assigned to that repository during creation
3. If missing, go to Org settings and re-edit the secret
4. Add the repository to the access list

### Problem: Can't Sign Up for Sentry

**Solution:**
1. Use GitHub OAuth (recommended): Click **"Sign up with GitHub"**
2. If you already have an account, sign in directly at https://sentry.io/auth/login/
3. Contact Sentry support if you have account issues

### Problem: Sentry Project Shows "No Data" After Setup

**This is normal.** Data appears only after:
1. SDK is installed in the project (coming next)
2. An error occurs in the running application
3. Error is sent to Sentry

---

## Next Steps

Once you've completed this guide:

1. **Notify:** Tell the team that Sentry setup is complete
2. **SDK Finalization:** The next phase will integrate the Sentry SDK into each project
3. **CI/CD Integration:** GitHub Actions workflows will use these secrets to configure Sentry
4. **Testing:** Send test errors to verify everything is working

---

## Summary

You've successfully:

✅ Created 3 Sentry projects (AgilePlus, phenotype-infrakit, heliosCLI)
✅ Obtained DSN tokens for each project
✅ Configured GitHub organization secrets
✅ Updated local `.env` files
✅ Verified accessibility from each repository

The DSN tokens are now ready for SDK integration in the next phase.

