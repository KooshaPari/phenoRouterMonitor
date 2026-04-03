# Snyk Token Acquisition Guide

Complete step-by-step guide to obtain your Snyk API token and verify authentication.

**Time Required:** 5-10 minutes
**Complexity:** Beginner
**Prerequisites:** Email address, web browser

---

## Overview

This guide walks you through:
1. Creating or logging into a Snyk account (free tier)
2. Accessing your account settings
3. Generating an authentication token
4. Securing the token temporarily
5. Verifying authentication locally

---

## Part 1: Snyk Account Access

### Step 1.1: Visit Snyk

Open your web browser and navigate to:
```
https://app.snyk.io
```

**Expected Result:** You should see the Snyk login page with:
- Email/password login form
- "Sign up" link
- "Forgot password" link

### Step 1.2: Login or Create Account

#### Option A: Existing Snyk Account
If you already have a Snyk account:
1. Click the email field and enter your registered email
2. Click the password field and enter your password
3. Click **"Sign In"**
4. Proceed to Step 1.3

#### Option B: New Snyk Account
If you don't have a Snyk account:
1. Click **"Sign up"** (in the bottom-right of login form, or at the top)
2. On the signup page, you have options:
   - Sign up with GitHub (recommended for this project)
   - Sign up with email and password
   - Sign up with Google

**Recommended:** Use GitHub SSO (Sign up with GitHub):
- Click **"Sign up with GitHub"**
- Authorize Snyk to access your GitHub account
- Review permissions (Snyk needs read access to repos)
- Click **"Authorize KooshaPari"**
- You'll be redirected to Snyk dashboard

**Alternative:** Sign up with email:
- Click **"Sign up"**
- Enter your email
- Create a password (minimum 8 characters, include numbers and special characters)
- Verify your email (Snyk will send a verification link)
- Check your email and click the verification link
- Return to https://app.snyk.io and log in

**Expected Result after login/signup:**
- Dashboard showing "Welcome to Snyk"
- Snyk logo in top-left
- Navigation menu with Projects, Reports, Settings, etc.

---

## Part 2: Token Generation

### Step 2.1: Navigate to Account Settings

From the Snyk dashboard:

1. Look for the **user icon** in the top-right corner of the screen
   - It typically shows your initials or a profile picture
2. Click the user icon
3. A dropdown menu appears with options including:
   - Your email address (read-only)
   - "Account settings" or "Settings"
   - "Preferences"
   - "Sign out"

**Click "Account settings"** (or "Settings")

**Expected Result:**
- You're now on the Settings page
- URL shows: `https://app.snyk.io/account/...`
- Left sidebar has options: "Profile", "Auth Token", "Preferences", etc.

### Step 2.2: Navigate to Auth Token

In the left sidebar, look for **"Auth Token"** or **"Authentication"**:

1. Click **"Auth Token"**
2. You'll see a section titled "Authentication Token" or "Auth Token"
3. There may be an existing token (masked or hidden)

**Expected Result:**
- Auth Token section is visible
- Shows existing token (if any), usually masked as `****...****`
- "Regenerate" or "Generate" button is visible

### Step 2.3: Generate a New Token

If you don't see an existing token, or want to generate a fresh one:

1. Click the **"Regenerate token"** button (or **"Generate token"** if none exists)
2. A confirmation dialog may appear asking:
   - "Are you sure you want to regenerate your token? This will invalidate the existing token."
   - (If you're generating for the first time, no confirmation)
3. Click **"Regenerate"** to confirm

**Expected Result:**
- A new token appears in plain text
- It looks like: `xxxxxxxx-xxxx-xxxx-xxxx-xxxxxxxxxxxx` (UUID format)
- A **"Copy"** button is available next to the token

### Step 2.4: Copy Token to Clipboard

1. Click the **"Copy"** button next to your token
2. The button may flash "Copied!" confirmation
3. Your token is now in your clipboard

**Important Security Note:**
- This token gives full access to your Snyk account
- Never commit it to git
- Never share it via email or Slack
- Never post it in public repositories
- Use it only for local deployment or GitHub Secrets (covered in later guides)

---

## Part 3: Secure Token Storage

### Temporary Storage (For Local Deployment)

For the local deployment script, you'll need the token temporarily accessible.

#### Option 1: Environment Variable (Recommended for Script Execution)

Store the token in a temporary environment variable:

```bash
export SNYK_TOKEN="<your-token-here>"
```

**Steps:**
1. Open your terminal
2. Paste the command above, replacing `<your-token-here>` with your actual token
3. Press Enter
4. Verify with: `echo $SNYK_TOKEN`
   - Expected output: Your token (or a portion of it)

**Example:**
```bash
export SNYK_TOKEN="12345678-abcd-1234-abcd-1234567890ab"
echo $SNYK_TOKEN
# Output: 12345678-abcd-1234-abcd-1234567890ab
```

**Security Notes for Environment Variable:**
- ✅ Token is in memory only (not on disk)
- ✅ Token is cleared when you close the terminal
- ⚠️ Token appears in shell history if not careful (see below)

**To Prevent Token in Shell History:**
Before running the export command, add a space at the beginning:
```bash
 export SNYK_TOKEN="your-token"  # Note the space at the start
```

Many shells are configured to exclude commands starting with a space from history.

#### Option 2: Temporary File (Alternative, Less Secure)

If you prefer not to use environment variables:

```bash
# Create a temporary file
echo "12345678-abcd-1234-abcd-1234567890ab" > ~/.snyk-token-temp

# Source it when needed
source ~/.snyk-token-temp

# After deployment, delete it
rm ~/.snyk-token-temp
```

**Security Notes:**
- ⚠️ Token is written to disk (less secure than env var)
- ⚠️ File could be exposed if disk is compromised
- ✅ Easy to verify and delete

#### Option 3: .env File with gitignore (For Development)

If running the script multiple times:

```bash
# Create .env file in deployment directory
cat > /Users/kooshapari/CodeProjects/Phenotype/repos/.env << EOF
SNYK_TOKEN="your-token-here"
EOF

# Verify .env is ignored by git
echo ".env" >> /Users/kooshapari/CodeProjects/Phenotype/repos/.gitignore

# Load it in your terminal
set -a
source /Users/kooshapari/CodeProjects/Phenotype/repos/.env
set +a
```

**Security Notes:**
- ✅ File is gitignored (won't be committed)
- ⚠️ Token is written to disk
- ✅ Convenient for multiple deployments

---

## Part 4: Token Verification

### Step 4.1: Install Snyk CLI (if not already installed)

First, check if Snyk CLI is installed:

```bash
snyk --version
```

**Expected Output:**
```
Snyk CLI version X.XXX.X
```

**If not installed:**

Using Homebrew (macOS/Linux):
```bash
brew install snyk
```

Using npm:
```bash
npm install -g snyk
```

Using direct download:
- Visit: https://github.com/snyk/cli/releases
- Download the latest release for your OS
- Extract and add to PATH

**Verify installation:**
```bash
snyk --version
```

### Step 4.2: Authenticate with Token

Once you have your token copied and Snyk CLI installed:

```bash
snyk auth your-token-here
```

Replace `your-token-here` with your actual token.

**Example:**
```bash
snyk auth 12345678-abcd-1234-abcd-1234567890ab
```

**Expected Output:**
```
Successfully authenticated
```

**Alternative: Using Environment Variable**

If you set `SNYK_TOKEN` as an environment variable (Part 3, Option 1):

```bash
snyk auth $SNYK_TOKEN
```

**Expected Output:**
```
Successfully authenticated
```

### Step 4.3: Verify Authentication

Test that your token works:

```bash
snyk test --help
```

**Expected Output:**
```
Test open source packages for known vulnerabilities

Usage
  $ snyk test [<PATH>] [OPTIONS]

...
```

Or run a quick test:

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos
snyk test --dry-run
```

**Expected Output:**
```
Snyk CLI test command
...
```

If you see an error like `Unauthorized`, your token is invalid. Return to Part 2 and regenerate.

---

## Part 5: Common Issues & Troubleshooting

### Issue: "Command 'snyk' not found"

**Cause:** Snyk CLI not installed or not in PATH

**Solution:**
```bash
# Option 1: Install with Homebrew
brew install snyk

# Option 2: Install with npm
npm install -g snyk

# Option 3: Verify PATH
echo $PATH
which snyk
```

### Issue: "Unauthorized" or "Invalid Token"

**Cause:** Token is incorrect or has been revoked

**Solution:**
1. Return to https://app.snyk.io
2. Go to Settings → Auth Token
3. Generate a new token
4. Copy it exactly (no extra spaces)
5. Run: `snyk auth <new-token>`

### Issue: Token in Shell History

**Cause:** Token was visible in terminal history

**Solution:**
1. Clear shell history:
   ```bash
   history -c  # Clear history in current session
   ```
2. Delete the token from `~/.bash_history` or `~/.zsh_history`:
   ```bash
   grep -v "SNYK_TOKEN" ~/.bash_history > ~/.bash_history.tmp
   mv ~/.bash_history.tmp ~/.bash_history
   ```
3. Regenerate your token at https://app.snyk.io (as a precaution)

### Issue: "SNYK_TOKEN environment variable not set"

**Cause:** Environment variable wasn't properly exported

**Solution:**
```bash
# Verify token is set
echo $SNYK_TOKEN

# If empty, set it again
export SNYK_TOKEN="your-token-here"

# Verify again
echo $SNYK_TOKEN
```

### Issue: Token Works with `snyk auth` but Not in Scripts

**Cause:** Environment variable not passed to subprocess

**Solution:**
```bash
# Option 1: Export in same shell before running script
export SNYK_TOKEN="your-token"
./scripts/snyk-deploy.sh

# Option 2: Pass directly to script
SNYK_TOKEN="your-token" ./scripts/snyk-deploy.sh

# Option 3: Use .env file and source it
source /path/to/.env
./scripts/snyk-deploy.sh
```

---

## Part 6: Security Checklist

Before proceeding to local deployment:

- [ ] Token successfully generated at https://app.snyk.io
- [ ] Token copied to clipboard
- [ ] Token stored securely (env var or temp file, NOT committed to git)
- [ ] Snyk CLI installed (`snyk --version` works)
- [ ] Authentication verified (`snyk auth <token>` succeeds)
- [ ] `snyk --version` outputs version number
- [ ] No token in shell history
- [ ] .env file (if used) is in .gitignore

---

## Next Steps

Once you've completed this guide:

1. **Proceed to SNYK_LOCAL_DEPLOYMENT_GUIDE.md** to run the deployment script
2. Keep your token stored securely for the deployment
3. After deployment completes, you can revoke this token at https://app.snyk.io if desired
4. When you add SNYK_TOKEN to GitHub Secrets, you'll generate a fresh token (GitHub will handle storage)

---

## Reference

- **Snyk Account:** https://app.snyk.io
- **Snyk CLI Documentation:** https://docs.snyk.io/cli
- **Snyk API Token:** https://docs.snyk.io/cli/authenticate-using-your-snyk-token
- **Snyk Security Best Practices:** https://docs.snyk.io/getting-started/best-practices-for-snyk-tokens

---

**Last Updated:** 2026-03-30
**Status:** Ready for deployment
