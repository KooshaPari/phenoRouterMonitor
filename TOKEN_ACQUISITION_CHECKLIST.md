# Token Acquisition Checklist

**OBJECTIVE:** Acquire and validate both Sentry and Snyk tokens for Phase 1 completion
**TIME ESTIMATE:** 8-10 minutes total
**CRITICAL:** Both tokens are required to proceed with automation

---

## Pre-Flight Check

Before starting, verify you have access to:

- [ ] Sentry organization (phenotype-org)
  - Login: https://sentry.io/
  - Check: Can you see "phenotype-org" in the org dropdown?

- [ ] Snyk account
  - Login: https://app.snyk.io/
  - Check: Can you access your account?

- [ ] GitHub CLI authenticated
  ```bash
  gh auth status
  # Should show: "Logged in to github.com as <your-username>"
  ```

- [ ] Sentry CLI installed and functional
  ```bash
  sentry-cli --version
  # Should show: sentry-cli 1.18.8
  ```

- [ ] Snyk CLI installed
  ```bash
  snyk --version
  # Should show: version 1.1303.2
  ```

---

## SECTION 1: Sentry Token Regeneration

### Overview

The Sentry token in `~/.sentryclirc` exists but has outdated scope. You need to regenerate it with `project:admin` permissions so the automation script can create projects.

### Current Token Status

```bash
# Check current token (this shows masked output for security)
cat ~/.sentryclirc
```

Expected output:
```
[auth]
token = sn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

[defaults]
org = phenotype-org
```

### Step-by-Step: Regenerate Sentry Token

#### Step 1a: Access Sentry Auth Tokens Page

1. Open your browser and navigate to:
   ```
   https://sentry.io/organizations/phenotype-org/settings/auth-tokens/
   ```

2. You should see a page titled **Auth Tokens** under Organization Settings

3. If redirected to login, enter your Sentry credentials

**CHECKPOINT:** Can you see the Auth Tokens page? ✅ YES / ❌ NO

---

#### Step 1b: Delete Old Token (if exists)

1. Look for any existing tokens (like `phenotype-automation` or `phenotype-infrakit-automation`)

2. If found, click the **trash icon** or **Delete** button on the right

3. Click **Delete** in the confirmation dialog

4. Wait for page to refresh

**CHECKPOINT:** Old token deleted or none found? ✅ YES / ❌ NO

---

#### Step 1c: Create New Token with Correct Scope

1. Click **Create New Token** (top-right button)

2. Fill in the form:

   **Field:** Name
   ```
   phenotype-automation-phase1
   ```

   **Field:** Scopes (Select ALL of these)
   - [ ] `project:admin` (required for creating projects)
   - [ ] `project:write` (required for modifying projects)
   - [ ] `org:read` (required for org access)
   - [ ] `team:read` (required for team access)

   **If you see other scope options:**
   - ❌ DON'T select: `team:admin`, `team:write`, `org:admin`, `org:write`
   - ✅ DO select all listed required scopes

3. Click **Create Token**

**CHECKPOINT:** Form filled with correct scopes? ✅ YES / ❌ NO

---

#### Step 1d: Copy the New Token

1. You'll see a modal with the token string (looks like):
   ```
   sn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
   ```

2. Click **Copy to Clipboard** button (or manually select and copy)

3. **IMPORTANT:** This is the only time you'll see this token. Save it for the next step.

4. Click **Close** to dismiss the modal

**CHECKPOINT:** Token copied to clipboard? ✅ YES / ❌ NO

---

#### Step 1e: Update ~/.sentryclirc

1. Open your terminal and edit the file:
   ```bash
   nano ~/.sentryclirc
   ```

2. You should see:
   ```
   [auth]
   token = sn_xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx

   [defaults]
   org = phenotype-org
   ```

3. **Replace the token value** on the line starting with `token =`:
   - Select the old token (after `token = `)
   - Delete it
   - Paste the new token from step 1d

4. **Result should look like:**
   ```
   [auth]
   token = sn_<your-new-token-here>

   [defaults]
   org = phenotype-org
   ```

5. Save and exit:
   - Press `Ctrl+O` (Mac: `Cmd+O`)
   - Press `Enter`
   - Press `Ctrl+X` (Mac: `Cmd+X`)

**CHECKPOINT:** Token updated in ~/.sentryclirc? ✅ YES / ❌ NO

---

#### Step 1f: Validate Sentry Token

Run this command to verify the token works:

```bash
sentry-cli projects list --org phenotype-org
```

**Expected output:**
```
Using organization: phenotype-org
No projects found
```

**Possible error outputs:**

| Error | Cause | Solution |
|-------|-------|----------|
| `Authentication failed` | Wrong token or expired | Go back to Step 1c and regenerate |
| `Organization not found` | Wrong org slug | Check `~/.sentryclirc` has `org = phenotype-org` |
| `Permission denied` | Wrong scope | Go back to Step 1c, delete token, regenerate with all scopes |

**CHECKPOINT:** Sentry token validated? ✅ YES / ❌ NO

---

## SECTION 2: Snyk Token Acquisition

### Overview

Snyk requires an API token for CLI authentication. You'll generate this from your Snyk account settings.

### Step-by-Step: Get Snyk Token

#### Step 2a: Access Snyk Account Settings

1. Open your browser and navigate to:
   ```
   https://app.snyk.io/account/settings
   ```

2. You should see your account settings page

3. If redirected to login, enter your Snyk credentials

**CHECKPOINT:** On Snyk account settings page? ✅ YES / ❌ NO

---

#### Step 2b: Find API Token Section

1. Scroll down the page to find the **API Token** section

2. You should see:
   - **Label:** "API Token"
   - **A hidden token** (appears as dots or hidden)
   - **A button:** "Generate" or "Regenerate" (or "Copy")

3. If you see "Copy", the token already exists. Click **Copy**.

4. If you see "Generate" or "Regenerate", click that button to create a new one.

**CHECKPOINT:** Found API Token section? ✅ YES / ❌ NO

---

#### Step 2c: Generate or Copy Token

**If you clicked "Generate" or "Regenerate":**
1. The system generates a new token
2. You'll see a string like: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
3. The token appears in the field and is selected
4. **Click "Copy"** to copy to clipboard

**If you clicked "Copy":**
1. The existing token is copied to clipboard
2. You're done with this step

**IMPORTANT:** You won't be able to see this token again. If you lose it, come back and regenerate a new one.

**CHECKPOINT:** Token copied to clipboard? ✅ YES / ❌ NO

---

#### Step 2d: Authenticate Snyk CLI

1. Open your terminal

2. Run:
   ```bash
   snyk auth
   ```

3. You'll see a prompt:
   ```
   ? Authentication token (or leave blank to authenticate via browser):
   ```

4. Paste the token from step 2c:
   ```
   <paste-token-here>
   ```

5. Press Enter

**Expected output:**
```
Your account has been authenticated. Snyk is now ready to use.
```

**CHECKPOINT:** Snyk auth command completed? ✅ YES / ❌ NO

---

#### Step 2e: Validate Snyk Token

Run this command to verify authentication:

```bash
snyk whoami
```

**Expected output:**
```
You are logged in as your-email@example.com

Ready to start using Snyk!
```

**Possible error outputs:**

| Error | Cause | Solution |
|-------|-------|----------|
| `Authentication failed` | Token format wrong or expired | Re-run Step 2d with the correct token |
| `Invalid token` | Token doesn't exist | Go back to Step 2c and generate a new one |
| `401 Unauthorized` | Token revoked | Go back to Step 2c, regenerate, then re-run Step 2d |

**CHECKPOINT:** Snyk token validated? ✅ YES / ❌ NO

---

## Verification Summary

### Combined Token Validation

Run both validation commands in sequence:

```bash
# Test Sentry token
echo "Testing Sentry token..."
sentry-cli projects list --org phenotype-org 2>&1 | head -3

# Test Snyk token
echo "Testing Snyk token..."
snyk whoami
```

**Expected output:**
```
Testing Sentry token...
Using organization: phenotype-org
No projects found
Testing Snyk token...
You are logged in as your-email@example.com
```

### Full Verification Checklist

Before proceeding to automation:

- [ ] Sentry token regenerated with `project:admin` scope
- [ ] Sentry token updated in `~/.sentryclirc`
- [ ] `sentry-cli projects list --org phenotype-org` returns success
- [ ] Snyk token generated or copied
- [ ] Snyk authentication completed with `snyk auth`
- [ ] `snyk whoami` returns your email address
- [ ] Both CLIs are authenticated and ready

---

## Troubleshooting

### Sentry Issues

**Problem:** `Authentication failed` when running `sentry-cli projects list`

**Diagnosis:**
1. Check that token is in `~/.sentryclirc`:
   ```bash
   grep "token =" ~/.sentryclirc
   ```
2. Verify the token starts with `sn_`:
   ```bash
   cat ~/.sentryclirc | grep "sn_"
   ```

**Solutions:**
1. Go back to Sentry (Step 1c) and regenerate
2. Make sure you selected `project:admin` scope
3. Try again: `sentry-cli projects list --org phenotype-org`

---

**Problem:** `Organization not found` error

**Diagnosis:**
- Check org slug in `~/.sentryclirc`:
  ```bash
  grep "org =" ~/.sentryclirc
  ```

**Solution:**
- Should be: `org = phenotype-org`
- If different, update it manually:
  ```bash
  nano ~/.sentryclirc
  ```

---

**Problem:** `Permission denied` error

**Diagnosis:**
- Token was created with wrong scope

**Solution:**
1. Go back to Sentry (Step 1b)
2. Delete the current token
3. Create a new one with ALL scopes: `project:admin`, `project:write`, `org:read`, `team:read`

---

### Snyk Issues

**Problem:** `Authentication failed` when running `snyk auth`

**Diagnosis:**
- Wrong token format

**Solution:**
1. Check token format (should be UUID-like): `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx`
2. Go back to Snyk (Step 2c) and regenerate a new token
3. Re-run `snyk auth` with the new token

---

**Problem:** `snyk whoami` returns `401 Unauthorized`

**Diagnosis:**
- Token was revoked or expired

**Solution:**
1. Go to https://app.snyk.io/account/settings
2. Click "Regenerate" on the API Token
3. Re-run `snyk auth` with the new token
4. Verify with `snyk whoami`

---

## Token Metadata Reference

### Sentry Token Details

**Location:** `~/.sentryclirc`

**Token Format:**
```
sn_<32-character-hex-string>
```

**Scopes Required:**
- `project:admin` — Create/delete projects
- `project:write` — Modify project settings
- `org:read` — Read organization info
- `team:read` — Read team info

**Expiration:** Never (unless manually revoked)

**Revocation:** Can be revoked at https://sentry.io/organizations/phenotype-org/settings/auth-tokens/

---

### Snyk Token Details

**Location:** `~/.snyk`

**Token Format:**
```
<32-character-hex-string>
```
(Created when you run `snyk auth`)

**Permissions:** Full access to your Snyk account

**Expiration:** Never (unless manually revoked)

**Revocation:** Can be revoked at https://app.snyk.io/account/settings

---

## Next Steps

Once both tokens are validated:

1. ✅ Close this checklist
2. ✅ Open `PHASE1_EXECUTION_NOW.md`
3. ✅ Follow the **Automation Execution** section
4. ✅ Run both automation scripts
5. ✅ Run verification script
6. ✅ Confirm Phase 1 complete (92% → 100%)

---

## Sign-Off

I confirm that:

- [ ] Both tokens have been acquired
- [ ] Both tokens have been validated
- [ ] I'm ready to proceed with automation

**Estimated time to Phase 1 completion:** 23 minutes from this point
