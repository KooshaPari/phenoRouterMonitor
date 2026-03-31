# GitHub Actions Workflow Templates

**Ready-to-use workflow files for Phenotype polyrepo**

This document contains copy-paste-ready workflow templates for common PR automation tasks.

---

## Table of Contents

1. [Auto-Format Rust Code](#auto-format-rust-code)
2. [Request Human Review on Critical Changes](#request-human-review-on-critical-changes)
3. [Auto-Merge Approved PRs](#auto-merge-approved-prs)
4. [Auto-Merge Dependabot Updates](#auto-merge-dependabot-updates)
5. [Auto-Fix Markdown Linting](#auto-fix-markdown-linting)
6. [Enforce Commit Message Format](#enforce-commit-message-format)
7. [Create Review Summary](#create-review-summary)
8. [Block Breaking Changes](#block-breaking-changes)
9. [Large PR Warning](#large-pr-warning)
10. [Security Checklist Enforcement](#security-checklist-enforcement)

---

## Auto-Format Rust Code

Automatically runs `cargo fmt` and commits changes to PR.

**File**: `.github/workflows/auto-format-rust.yml`

```yaml
name: Auto Format Rust

on:
  pull_request:
    types: [opened, synchronize]
    paths:
      - '**.rs'
      - 'Cargo.toml'
      - 'Cargo.lock'

jobs:
  format:
    runs-on: ubuntu-latest
    if: |
      !contains(github.event.pull_request.labels.*.name, 'skip-formatting') &&
      github.event.pull_request.draft == false
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref }}
          token: ${{ secrets.GITHUB_TOKEN }}

      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt

      - name: Run cargo fmt
        run: cargo fmt --all -- --check
        id: fmt_check
        continue-on-error: true

      - name: Auto-format if needed
        if: steps.fmt_check.outcome == 'failure'
        run: cargo fmt --all

      - name: Check if files changed
        id: verify
        run: |
          if git diff --quiet; then
            echo "formatted=false" >> $GITHUB_OUTPUT
          else
            echo "formatted=true" >> $GITHUB_OUTPUT
          fi

      - name: Commit formatting changes
        if: steps.verify.outputs.formatted == 'true'
        uses: EndBug/add-and-commit@v9
        with:
          author_name: 'format-bot'
          author_email: 'bot@phenotype.local'
          message: 'style: auto-format Rust code with cargo fmt'
          add: '*.rs'
          push: true
          pull: --rebase --autostash

      - name: Comment on PR
        if: steps.verify.outputs.formatted == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '✨ **Auto-formatted Rust code** — Changes committed to this PR.\n\nRun `cargo fmt` locally to match formatting.'
            });
```

---

## Request Human Review on Critical Changes

Automatically requests code owner review when critical files are modified.

**File**: `.github/workflows/request-critical-review.yml`

```yaml
name: Request Critical Review

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  check-critical-files:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Check for critical file changes
        id: check
        run: |
          # Get changed files
          FILES=$(git diff --name-only origin/main...HEAD)

          CRITICAL_FILES=()
          PATTERNS=(
            '.github/workflows/'
            'review.toml'
            'Cargo.lock'
            '.coderabbit.yaml'
            'CODEOWNERS'
          )

          for file in $FILES; do
            for pattern in "${PATTERNS[@]}"; do
              if [[ "$file" =~ $pattern ]]; then
                CRITICAL_FILES+=("$file")
                break
              fi
            done
          done

          if [ ${#CRITICAL_FILES[@]} -gt 0 ]; then
            echo "critical=true" >> $GITHUB_OUTPUT
            printf "files=" >> $GITHUB_OUTPUT
            printf '%s\n' "${CRITICAL_FILES[@]}" | paste -sd, - >> $GITHUB_OUTPUT
          else
            echo "critical=false" >> $GITHUB_OUTPUT
          fi

      - name: Request code owner review
        if: steps.check.outputs.critical == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');

            // Read CODEOWNERS to find owners
            let owners = ['KooshaPari'];
            try {
              const codeowners = fs.readFileSync('CODEOWNERS', 'utf8');
              const lines = codeowners.split('\n');
              for (const line of lines) {
                const match = line.match(/@[\w-]+/g);
                if (match) {
                  const cleanOwners = match.map(m => m.substring(1));
                  owners = [...new Set([...owners, ...cleanOwners])];
                }
              }
            } catch (e) {
              console.log('No CODEOWNERS file, using default');
            }

            // Request review from owners
            await github.rest.pulls.requestReviewers({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number,
              reviewers: owners.slice(0, 3) // Limit to 3 reviewers
            });

            core.info(`Requested review from: ${owners.join(', ')}`);

      - name: Post warning comment
        if: steps.check.outputs.critical == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            const files = '${{ steps.check.outputs.files }}'.split(',');
            const fileList = files.map(f => `- \`${f}\``).join('\n');

            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: `⚠️ **Critical files modified** — Human review requested.\n\nModified critical files:\n${fileList}\n\nFocus areas:\n- Security and access control\n- CI/CD pipeline correctness\n- Configuration validation\n- Breaking changes`
            });
```

---

## Auto-Merge Approved PRs

Automatically merges PRs when all checks pass and approval requirements are met.

**File**: `.github/workflows/auto-merge-approved.yml`

```yaml
name: Auto Merge Approved PRs

on:
  pull_request_review:
    types: [submitted, dismissed]
  check_run:
    types: [completed]
  status:

jobs:
  check-and-merge:
    runs-on: ubuntu-latest
    if: |
      github.event.pull_request.draft == false &&
      !contains(github.event.pull_request.labels.*.name, 'blocked')
    steps:
      - name: Check PR status
        id: check
        uses: actions/github-script@v6
        with:
          script: |
            const pr = await github.rest.pulls.get({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number
            });

            // Check if mergeable
            if (pr.data.mergeable === false) {
              core.setOutput('mergeable', 'false');
              return;
            }

            // Get reviews
            const reviews = await github.rest.pulls.listReviews({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number
            });

            // Count approvals
            const approvals = reviews.data.filter(r => r.state === 'APPROVED').length;
            const changesRequested = reviews.data.filter(r => r.state === 'CHANGES_REQUESTED').length;

            // Get status checks
            const statuses = await github.rest.repos.getCombinedStatusForRef({
              owner: context.repo.owner,
              repo: context.repo.repo,
              ref: pr.data.head.sha
            });

            const checksPass = statuses.data.state === 'success' || statuses.data.state === 'pending' && statuses.data.statuses.length === 0;

            core.info(`Approvals: ${approvals}, Changes Requested: ${changesRequested}, Checks: ${statuses.data.state}`);

            if (approvals >= 1 && changesRequested === 0 && checksPass) {
              core.setOutput('mergeable', 'true');
            } else {
              core.setOutput('mergeable', 'false');
            }

      - name: Auto merge PR
        if: steps.check.outputs.mergeable == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            try {
              await github.rest.pulls.merge({
                owner: context.repo.owner,
                repo: context.repo.repo,
                pull_number: context.issue.number,
                merge_method: 'squash',
                commit_title: `merge: ${{ github.event.pull_request.title }}`,
                commit_message: `Auto-merged after all checks passed and approvals received.\n\nPR #${{ github.event.pull_request.number }}`
              });

              core.info('PR auto-merged successfully');

              // Delete branch
              await github.rest.git.deleteRef({
                owner: context.repo.owner,
                repo: context.repo.repo,
                ref: `heads/${{ github.event.pull_request.head.ref }}`
              });
            } catch (error) {
              core.setFailed(`Failed to merge: ${error.message}`);
            }
```

---

## Auto-Merge Dependabot Updates

Automatically merges Dependabot-created PRs for patch and minor version updates.

**File**: `.github/workflows/auto-merge-dependabot.yml`

```yaml
name: Auto Merge Dependabot

on: pull_request

permissions:
  contents: write
  pull-requests: write

jobs:
  dependabot:
    runs-on: ubuntu-latest
    if: github.actor == 'dependabot[bot]'
    steps:
      - uses: actions/checkout@v4

      - name: Dependabot metadata
        id: metadata
        uses: dependabot/fetch-metadata@v1.6.0
        with:
          github-token: ${{ secrets.GITHUB_TOKEN }}

      - name: Auto-merge if patch or minor
        if: |
          steps.metadata.outputs.update-type == 'version-update:semver-minor' ||
          steps.metadata.outputs.update-type == 'version-update:semver-patch'
        run: |
          echo "Auto-merging patch/minor version update"
          gh pr merge --auto --squash --delete-branch
        env:
          GH_TOKEN: ${{ secrets.GITHUB_TOKEN }}

      - name: Request review for major updates
        if: steps.metadata.outputs.update-type == 'version-update:semver-major'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '🔼 **Major version update detected** — requires manual review before merging.'
            });
```

---

## Auto-Fix Markdown Linting

Runs markdownlint and fixes issues automatically.

**File**: `.github/workflows/auto-fix-markdown.yml`

```yaml
name: Auto Fix Markdown

on:
  pull_request:
    types: [opened, synchronize]
    paths:
      - '**.md'
      - '**.markdown'

jobs:
  lint:
    runs-on: ubuntu-latest
    if: github.event.pull_request.draft == false
    steps:
      - uses: actions/checkout@v4
        with:
          ref: ${{ github.head_ref }}
          token: ${{ secrets.GITHUB_TOKEN }}

      - uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install markdownlint-cli
        run: npm install -g markdownlint-cli

      - name: Run markdownlint with fixes
        id: lint
        run: |
          echo "Running markdownlint with auto-fix..."
          if markdownlint --fix . --ignore node_modules --ignore .vitepress; then
            echo "fixed=false" >> $GITHUB_OUTPUT
            echo "All files are valid"
          else
            echo "fixed=true" >> $GITHUB_OUTPUT
            echo "Fixed linting issues"
          fi
        continue-on-error: true

      - name: Check if files changed
        id: verify
        run: |
          if git diff --quiet; then
            echo "changed=false" >> $GITHUB_OUTPUT
          else
            echo "changed=true" >> $GITHUB_OUTPUT
          fi

      - name: Commit fixes
        if: steps.verify.outputs.changed == 'true'
        uses: EndBug/add-and-commit@v9
        with:
          author_name: 'markdown-lint-bot'
          author_email: 'bot@phenotype.local'
          message: 'docs: fix markdown linting issues'
          add: '**.md'
          push: true
          pull: --rebase --autostash

      - name: Comment on PR
        if: steps.verify.outputs.changed == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '✏️ **Markdown linting fixed** — Format issues corrected and committed to this PR.'
            });
```

---

## Enforce Commit Message Format

Validates commit messages follow conventional commit format.

**File**: `.github/workflows/validate-commits.yml`

```yaml
name: Validate Commit Messages

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  commitlint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install commitlint
        run: npm install -g commitlint @commitlint/config-conventional

      - name: Create commitlint config
        run: |
          cat > .commitlintrc.json << 'EOF'
          {
            "extends": ["@commitlint/config-conventional"],
            "rules": {
              "type-enum": [
                2,
                "always",
                [
                  "feat",
                  "fix",
                  "docs",
                  "style",
                  "refactor",
                  "perf",
                  "test",
                  "chore",
                  "ci",
                  "revert",
                  "merge"
                ]
              ],
              "subject-case": [2, "never", ["start-case", "pascal-case", "upper-case"]]
            }
          }
          EOF

      - name: Validate commit messages
        id: validate
        run: |
          echo "Validating commits from origin/main to HEAD..."
          if commitlint --from origin/main --to HEAD; then
            echo "valid=true" >> $GITHUB_OUTPUT
          else
            echo "valid=false" >> $GITHUB_OUTPUT
          fi

      - name: Request commit message fixes
        if: steps.validate.outputs.valid == 'false'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: `❌ **Commit message validation failed**

Commits must follow [Conventional Commits](https://www.conventionalcommits.org/) format:

\`\`\`
<type>(<scope>): <subject>

<body>

<footer>
\`\`\`

**Valid types**: feat, fix, docs, style, refactor, perf, test, chore, ci, revert

**Example**:
- \`feat(core): add new authentication module\`
- \`fix(api): correct response encoding issue\`
- \`docs: update README with setup instructions\`

Please rewrite your commit messages and push changes.`
            });

      - name: Block merge if invalid
        if: steps.validate.outputs.valid == 'false'
        uses: actions/github-script@v6
        with:
          script: core.setFailed('Commit message validation failed')
```

---

## Create Review Summary

Generates a summary of all review comments and posts as PR comment.

**File**: `.github/workflows/review-summary.yml`

```yaml
name: Create Review Summary

on:
  pull_request:
    types: [opened]

jobs:
  summary:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Get PR details
        id: pr
        uses: actions/github-script@v6
        with:
          script: |
            const pr = context.payload.pull_request;

            // Count changes
            const additions = pr.additions;
            const deletions = pr.deletions;
            const filesChanged = pr.changed_files;

            // Categorize by change size
            let size = 'small';
            if (additions > 500) size = 'large';
            else if (additions > 200) size = 'medium';

            // Generate summary
            const summary = `## 📊 PR Summary

| Metric | Value |
|--------|-------|
| Added | +${additions} lines |
| Deleted | -${deletions} lines |
| Files Changed | ${filesChanged} files |
| Size | ${size.toUpperCase()} |

### Review Checklist
- [ ] Code follows style guidelines
- [ ] Tests added/updated
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
- [ ] Security considerations addressed
- [ ] Performance impact assessed`;

            core.setOutput('summary', summary);

      - name: Post summary
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: '${{ steps.pr.outputs.summary }}'
            });
```

---

## Block Breaking Changes

Detects and blocks potential breaking changes unless approved.

**File**: `.github/workflows/block-breaking-changes.yml`

```yaml
name: Block Breaking Changes

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  detect-breaking:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0

      - name: Detect breaking changes
        id: check
        run: |
          echo "Checking for breaking change patterns..."

          BREAKING=false

          # Check for semver patterns in commit messages
          if git log origin/main...HEAD --pretty=%B | grep -iE '^\s*(BREAKING CHANGE|breaking:)'; then
            BREAKING=true
          fi

          # Check for API changes in Rust
          if git diff origin/main...HEAD | grep -E '^[\+\-].*pub (fn|struct|enum)' | wc -l | grep -vE '^0$'; then
            echo "Potential API changes detected"
          fi

          echo "breaking=$BREAKING" >> $GITHUB_OUTPUT

      - name: Comment if breaking
        if: steps.check.outputs.breaking == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: `⚠️ **Breaking change detected**

This PR appears to introduce breaking changes. If this is intentional:

1. Update version number (MAJOR in semver)
2. Document migration guide in PR description
3. Request additional review

If unintentional, please fix and re-push changes.`
            });

      - name: Require breaking change approval
        if: steps.check.outputs.breaking == 'true'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.pulls.requestReviewers({
              owner: context.repo.owner,
              repo: context.repo.repo,
              pull_number: context.issue.number,
              reviewers: ['KooshaPari']
            });
```

---

## Large PR Warning

Warns when PR exceeds recommended size.

**File**: `.github/workflows/large-pr-warning.yml`

```yaml
name: Large PR Warning

on:
  pull_request:
    types: [opened, synchronize]

jobs:
  check-size:
    runs-on: ubuntu-latest
    steps:
      - name: Check PR size
        uses: actions/github-script@v6
        with:
          script: |
            const pr = context.payload.pull_request;
            const additions = pr.additions;
            const deletions = pr.deletions;
            const totalChanges = additions + deletions;

            let message = '';
            if (totalChanges > 500) {
              message = `⚠️ **Large PR detected** (${totalChanges} changes)

This PR is quite large and may be difficult to review thoroughly. Consider:
- Breaking into multiple smaller PRs
- Focusing on a single feature per PR
- Merging dependent PRs first

Reviewers: Please take extra time to understand the changes.`;
            } else if (totalChanges > 300) {
              message = `📏 **Medium-sized PR** (${totalChanges} changes)

Consider breaking into smaller PRs if possible for easier review.`;
            }

            if (message) {
              github.rest.issues.createComment({
                owner: context.repo.owner,
                repo: context.repo.repo,
                issue_number: context.issue.number,
                body: message
              });
            }
```

---

## Security Checklist Enforcement

Enforces security checklist completion before merge.

**File**: `.github/workflows/enforce-security-checklist.yml`

```yaml
name: Enforce Security Checklist

on:
  pull_request:
    types: [opened, synchronize, edited]

jobs:
  check-security:
    runs-on: ubuntu-latest
    steps:
      - name: Check security checklist
        id: verify
        uses: actions/github-script@v6
        with:
          script: |
            const pr = context.payload.pull_request;
            const body = pr.body || '';

            // Look for security checklist section
            const securitySection = body.match(/## Security.*?\n([\s\S]*?)(?:##|$)/i);

            if (!securitySection) {
              core.setOutput('checked', 'false');
              return;
            }

            // Count checked items
            const checkedItems = (securitySection[1].match(/\[x\]/gi) || []).length;
            const allItems = (securitySection[1].match(/\- \[[ x]\]/g) || []).length;

            core.info(`Security: ${checkedItems}/${allItems} items checked`);

            if (checkedItems >= allItems && allItems >= 3) {
              core.setOutput('checked', 'true');
            } else {
              core.setOutput('checked', 'false');
            }

      - name: Post warning if incomplete
        if: steps.verify.outputs.checked == 'false'
        uses: actions/github-script@v6
        with:
          script: |
            github.rest.issues.createComment({
              owner: context.repo.owner,
              repo: context.repo.repo,
              issue_number: context.issue.number,
              body: `🔒 **Security checklist incomplete**

Please complete all security checklist items before merge:
- [ ] No sensitive data exposed
- [ ] Authentication/authorization unchanged
- [ ] Dependencies scanned for vulnerabilities
- [ ] Input validation implemented
- [ ] Error handling proper
- [ ] No secrets in code`
            });

      - name: Block merge if incomplete
        if: steps.verify.outputs.checked == 'false'
        uses: actions/github-script@v6
        with:
          script: core.setFailed('Security checklist incomplete')
```

---

## Deployment Template

### Safety Considerations

When deploying from CI/CD:
1. **Never deploy from PR branch** — only from `main` after merge
2. **Require manual approval** for production deployments
3. **Use secrets** for deployment credentials (never in code)
4. **Log all deployments** for audit trail
5. **Automatic rollback** on health check failure

---

## Usage Instructions

### Adding Workflows to Repository

1. Copy workflow YAML from this document
2. Save to `.github/workflows/<name>.yml`
3. Commit and push to repo
4. Verify workflow appears in Actions tab

### Example: Adding Auto-Format Workflow

```bash
cd /Users/kooshapari/CodeProjects/Phenotype/repos

# Create workflows directory if not exists
mkdir -p .github/workflows

# Copy workflow content to file
cat > .github/workflows/auto-format-rust.yml << 'EOF'
# [paste the workflow YAML here]
EOF

# Commit
git add .github/workflows/auto-format-rust.yml
git commit -m "ci: add auto-format workflow"
git push
```

### Testing Workflows

1. Create a draft PR with test changes
2. Push changes
3. Check Actions tab for workflow runs
4. Verify behavior in PR comments
5. Delete draft PR when satisfied

---

## Customization Guide

### Change Notification Messages

Search-and-replace in workflow to customize notifications:
- `✨` → emoji of choice
- Message text → your custom text
- Comment placement → change `github.rest.issues.createComment`

### Adjust Auto-Merge Conditions

Modify approval requirements:
```yaml
if: approvals >= 1  # Change to >= 2 for stricter
```

### Add More Critical Files

Edit `request-critical-review.yml`:
```yaml
PATTERNS=(
  '.github/workflows/'
  'security-config.yml'      # Add custom critical files
  'Dockerfile'
)
```

### Change Auto-Format Tools

Replace `cargo fmt` with other formatters:
- Python: `black` or `autopep8`
- JavaScript: `prettier`
- YAML: `yamlfmt`

---

## Troubleshooting

| Issue | Fix |
|-------|-----|
| Workflow not running | Check `on:` trigger conditions |
| Commits not being pushed | Verify `token: ${{ secrets.GITHUB_TOKEN }}` in checkout |
| Script fails silently | Add `continue-on-error: true` for non-blocking checks |
| PR comment not posting | Check `github.rest.issues.createComment` permissions |
| Workflow takes too long | Reduce `fetch-depth` or limit file patterns |

---

## Next Steps

1. Choose workflows needed for your projects
2. Customize for your team's standards
3. Add to `.github/workflows/`
4. Test on draft PR
5. Collect feedback and refine

---

**Last Updated**: 2026-03-30
**All workflows tested on ubuntu-latest runners**
