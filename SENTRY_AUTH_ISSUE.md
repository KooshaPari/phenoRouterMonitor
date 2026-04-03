# Sentry Project Creation — Authentication Issue Report

**Status:** ❌ BLOCKED — Auth token insufficient permissions

**Date:** 2026-03-31
**Attempted:** Automated Sentry project creation via API and sentry-cli

---

## Issue Summary

The stored Sentry auth token (`~/.sentryclirc`) does not have sufficient permissions to:
- Create projects in the organization
- Read organization details
- List existing projects

**Token Details:**
- **Token (truncated):** `sntrys_eyJpYXQiOjE3NzQ5MjY2OTYuNjI4MTA4LCJ1cmwiOiJodHRwczovL3NlbnRyeS5pbyIsInJlZ2lvbl91cmwiOiJodHRwczovL3VzLnNlbnRyeS5pbyIsIm9yZyI6InN0ZWFsdGgtc3RhcnR1cC0zdSJ9_LC2...` (full token in config)
- **Organization:** `stealth-startup-3u`
- **Extracted from:** `~/.sentryclirc`

**Error Messages:**
```
HTTP 403: You do not have permission to perform this action
```

**Attempted Endpoints:**
1. `GET /api/0/organizations/stealth-startup-3u/` — 403 Forbidden
2. `POST /api/0/organizations/stealth-startup-3u/projects/` — 403 Forbidden
3. `sentry-cli organizations list` — 403 Forbidden

---

## Root Cause Analysis

The token may be one of the following:

1. **Insufficient Scopes:** Token created with read-only or limited scopes (e.g., no project creation)
2. **Revoked/Expired:** Token was previously valid but has been revoked or is expired
3. **Organization Mismatch:** Token belongs to a different organization than `stealth-startup-3u`
4. **Inactive Account:** Organization membership was removed or account was suspended

---

## Required Actions

### Step 1: Verify Token Ownership
1. Log into Sentry dashboard: https://sentry.io/
2. Navigate to Settings → Auth Tokens (or API Tokens)
3. Verify the token matches the one in `~/.sentryclirc`
4. Check if token is active and has "project:admin" or "org:admin" scope

### Step 2: Regenerate or Create New Token
If token is insufficient:

1. Go to https://sentry.io/settings/auth-tokens/
2. Create a new token with scopes:
   - `project:admin` (create/delete projects)
   - `org:read` (read organization details)
   - `team:admin` (team management)
3. Copy the new token

### Step 3: Update Config
```bash
# Replace token in ~/.sentryclirc
sed -i 's/^token=.*/token=<NEW_TOKEN_HERE>/' ~/.sentryclirc
```

### Step 4: Verify Access
```bash
export SENTRY_AUTH_TOKEN=<NEW_TOKEN>
sentry-cli organizations list
```

### Step 5: Re-run Project Creation
Once verified, run the automated project creation script:
```bash
bash /Users/kooshapari/CodeProjects/Phenotype/repos/scripts/create-sentry-projects.sh
```

---

## Workaround (Manual)

If you cannot regenerate the token via API, create projects manually via the Sentry dashboard:

1. Go to https://sentry.io/organizations/stealth-startup-3u/projects/
2. Click "Create Project"
3. Create 3 projects:
   - **Project 1:** AgilePlus (Platform: Rust)
   - **Project 2:** phenotype-infrakit (Platform: Rust)
   - **Project 3:** heliosCLI (Platform: Rust)
4. Copy each DSN token after creation
5. Configure GitHub Secrets manually:
   ```bash
   gh secret set SENTRY_DSN_AGILEPLUS --body '<DSN1>'
   gh secret set SENTRY_DSN_INFRAKIT --body '<DSN2>'
   gh secret set SENTRY_DSN_HELIOSCLI --body '<DSN3>'
   ```

---

## Next Steps

1. **User Action Required:** Regenerate auth token with proper scopes
2. **Update Config:** Replace token in `~/.sentryclirc`
3. **Verification:** Test with `sentry-cli organizations list`
4. **Re-attempt:** Re-run automated script or proceed with manual creation

---

## Reference Links

- Sentry Auth Tokens: https://sentry.io/settings/auth-tokens/
- Sentry API Scopes: https://docs.sentry.io/api/authentication/#token-auth
- sentry-cli Documentation: https://docs.sentry.io/cli/
