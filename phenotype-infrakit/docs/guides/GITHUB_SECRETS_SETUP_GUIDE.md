# GitHub Secrets Setup Guide

Step-by-step walkthrough for configuring organization-level GitHub secrets for Sentry DSN tokens.

**Estimated Time:** 10 minutes

**What You'll Create:**
- 3 GitHub organization secrets
- Each secret is scoped to its respective repository
- All secrets are accessible from GitHub Actions workflows

---

## Prerequisites

- **GitHub Account:** With organization admin or "Maintain" role
- **Organization:** `KooshaPari`
- **Sentry DSN Tokens:** You should have obtained these from the previous setup phase
  - `SENTRY_DSN_AGILEPLUS`
  - `SENTRY_DSN_INFRAKIT`
  - `SENTRY_DSN_HELIOSCLI`

---

## Overview: GitHub Organization Secrets

### Why Organization-Level Secrets?

- **Single Source of Truth:** Configure once, use in multiple repositories
- **Consistency:** All 3 projects use the same secret names
- **Security:** Encrypted at rest, only visible in CI/CD contexts
- **Access Control:** Admins manage who sees what

### Secret Scope

Each secret is assigned to specific repositories:
- `SENTRY_DSN_AGILEPLUS` → accessible from `AgilePlus` repo
- `SENTRY_DSN_INFRAKIT` → accessible from `phenotype-infrakit` repo
- `SENTRY_DSN_HELIOSCLI` → accessible from `heliosCLI` repo

---

## Navigation Checklist

Before starting, verify you're in the right place:

1. **Organization Level** (NOT repository level)
   - URL: `https://github.com/organizations/KooshaPari/settings/`
2. **Left Sidebar:** Look for "Security" section
3. **Under Security:** Click **"Secrets and variables"**
4. **Tabs:** You should see: **Secrets | Variables | Dependabot**
5. **Verify:** You're on the "Secrets" tab

---

## Step-by-Step: Add Each Secret

### Secret #1: SENTRY_DSN_AGILEPLUS

#### Step 1: Click "New organization secret"

1. Go to: https://github.com/organizations/KooshaPari/settings/secrets/actions
2. Click **"New organization secret"** (green button, top right)

#### Step 2: Fill in Secret Details

You'll see a form with 3 fields:

**Field 1: Name**
```
Name: SENTRY_DSN_AGILEPLUS
```

**Field 2: Secret value**
```
Paste the full DSN from Sentry:
https://your-key@sentry.io/your-project-id
```

Example:
```
https://abc123def456@sentry.io/9876543210
```

**Field 3: Repository access**

Select: **"Selected repositories"** (radio button)

Then click **"Add repositories"** and search for:
```
AgilePlus
```

Select `KooshaPari/AgilePlus` from the dropdown.

#### Step 3: Save Secret

Click **"Add secret"** (green button at bottom)

**Confirmation:** You should be redirected to the Secrets list and see:
```
SENTRY_DSN_AGILEPLUS — Updated X seconds ago
```

---

### Secret #2: SENTRY_DSN_INFRAKIT

#### Step 1: Click "New organization secret"

1. Click **"New organization secret"** (green button, top right)

#### Step 2: Fill in Secret Details

**Field 1: Name**
```
Name: SENTRY_DSN_INFRAKIT
```

**Field 2: Secret value**
```
Paste the full DSN from Sentry:
https://your-key@sentry.io/your-project-id
```

Example:
```
https://def456ghi789@sentry.io/9876543211
```

**Field 3: Repository access**

Select: **"Selected repositories"**

Click **"Add repositories"** and search for:
```
phenotype-infrakit
```

Select `KooshaPari/phenotype-infrakit` from the dropdown.

#### Step 3: Save Secret

Click **"Add secret"**

**Confirmation:**
```
SENTRY_DSN_INFRAKIT — Updated X seconds ago
```

---

### Secret #3: SENTRY_DSN_HELIOSCLI

#### Step 1: Click "New organization secret"

1. Click **"New organization secret"** (green button, top right)

#### Step 2: Fill in Secret Details

**Field 1: Name**
```
Name: SENTRY_DSN_HELIOSCLI
```

**Field 2: Secret value**
```
Paste the full DSN from Sentry:
https://your-key@sentry.io/your-project-id
```

Example:
```
https://ghi789jkl012@sentry.io/9876543212
```

**Field 3: Repository access**

Select: **"Selected repositories"**

Click **"Add repositories"** and search for:
```
heliosCLI
```

Select `KooshaPari/heliosCLI` from the dropdown.

#### Step 3: Save Secret

Click **"Add secret"**

**Confirmation:**
```
SENTRY_DSN_HELIOSCLI — Updated X seconds ago
```

---

## Verification: Confirm Secrets Are Accessible

After creating all 3 secrets, verify they're accessible from each repository.

### Verify from AgilePlus Repository

1. Go to: https://github.com/KooshaPari/AgilePlus
2. Click **Settings** (top navigation)
3. Click **Secrets and variables** → **Actions** (left sidebar)
4. Under **"Organization secrets"** section, you should see:
   ```
   ✓ SENTRY_DSN_AGILEPLUS — Available
   ```

If not visible:
- The secret may not have been assigned to this repo
- Go back to Org Settings and re-edit the secret
- Add `AgilePlus` to the repository list

### Verify from phenotype-infrakit Repository

1. Go to: https://github.com/KooshaPari/phenotype-infrakit
2. Click **Settings** (top navigation)
3. Click **Secrets and variables** → **Actions** (left sidebar)
4. Under **"Organization secrets"** section, you should see:
   ```
   ✓ SENTRY_DSN_INFRAKIT — Available
   ```

If not visible:
- Go back to Org Settings and re-edit the secret
- Add `phenotype-infrakit` to the repository list

### Verify from heliosCLI Repository

1. Go to: https://github.com/KooshaPari/heliosCLI
2. Click **Settings** (top navigation)
3. Click **Secrets and variables** → **Actions** (left sidebar)
4. Under **"Organization secrets"** section, you should see:
   ```
   ✓ SENTRY_DSN_HELIOSCLI — Available
   ```

If not visible:
- Go back to Org Settings and re-edit the secret
- Add `heliosCLI` to the repository list

---

## Using Secrets in GitHub Actions Workflows

Once configured, secrets are available in GitHub Actions as environment variables.

### Example: Access in CI/CD Workflow

In a `.github/workflows/ci.yml` file:

```yaml
name: CI
on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Set Sentry DSN
        run: |
          echo "SENTRY_DSN=${{ secrets.SENTRY_DSN_AGILEPLUS }}" >> $GITHUB_ENV

      - name: Run tests
        run: cargo test
```

The `${{ secrets.SENTRY_DSN_AGILEPLUS }}` will be replaced with the actual DSN value at runtime.

---

## Troubleshooting

### Problem: "Repository not found" when adding repo to secret

**Possible Causes:**
1. Repository name is misspelled
2. You don't have admin access to the organization
3. The repository doesn't exist yet

**Solution:**
1. Double-check the repository name in the GitHub URL
2. Make sure you're logged in with an account that has admin access
3. Verify the repository exists at `https://github.com/KooshaPari/{repo-name}`

### Problem: Secret not visible in repository settings

**Possible Causes:**
1. Secret was created at repository level (not organization level)
2. Repository wasn't added to the secret access list
3. Repository name in the access list is incorrect

**Solution:**
1. Go back to **Organization Settings** (not repository settings)
2. Edit the secret and verify the repository is in the access list
3. Check for typos in the repository name

### Problem: Workflow can't access the secret

**Possible Causes:**
1. Secret name is wrong (case-sensitive)
2. Repository isn't in the secret's access list
3. Workflow is using the wrong syntax

**Solution:**
1. Verify the secret name matches exactly: `SENTRY_DSN_AGILEPLUS` (not `sentry_dsn_agileplus`)
2. Go to Org Settings and confirm the repository is in the access list
3. Use correct syntax in workflow: `${{ secrets.SENTRY_DSN_AGILEPLUS }}`

### Problem: "Insufficient permissions" error

**Possible Causes:**
1. Your GitHub account doesn't have admin/maintain role in the organization
2. Organization has additional security policies

**Solution:**
1. Ask organization owner to confirm your role
2. Request admin or "Maintain" access if needed
3. Check organization security settings

---

## Security Best Practices

1. **Never Log Secrets**
   - Don't print secrets in logs
   - Use `::add-mask::` in GitHub Actions to hide from logs

2. **Limited Repository Access**
   - Only add repositories that actually need the secret
   - Don't add "all repositories" unless necessary

3. **Rotate Regularly**
   - Treat secrets like passwords
   - Rotate if there's any suspicion of compromise

4. **Audit Access**
   - Check periodically which repos have access to each secret
   - Remove access for archived/deleted repos

---

## Summary Checklist

- [ ] Organization secrets page opened (Org Settings → Secrets and variables → Actions)
- [ ] `SENTRY_DSN_AGILEPLUS` secret created and assigned to `AgilePlus`
- [ ] `SENTRY_DSN_INFRAKIT` secret created and assigned to `phenotype-infrakit`
- [ ] `SENTRY_DSN_HELIOSCLI` secret created and assigned to `heliosCLI`
- [ ] Verified each secret is accessible from its repository
- [ ] Verified secret names match exactly (case-sensitive)
- [ ] Verified DSN values are complete (full `https://...` URL)

---

## Next Steps

Once all secrets are configured and verified:

1. **Local Development:** Use local `.env` files with the DSN values
2. **CI/CD Integration:** GitHub Actions workflows will automatically access secrets
3. **SDK Installation:** Next phase will integrate the Sentry SDK into each project
4. **Testing:** Send test errors to verify Sentry is capturing data

