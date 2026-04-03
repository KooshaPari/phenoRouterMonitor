# Phenotype Docs Chassis Interface

## Overview

The Phenotype Docs Chassis is a shared VitePress configuration and theme system that provides consistent documentation styling, navigation, and governance patterns across the Phenotype organization.

**Package**: `@phenotype/docs` (published to GitHub Packages)
**Location**: Phenotype monorepo → packages/phenotype-docs/
**Current Version**: 0.1.0+
**Stability**: Stable (backward compatible)

---

## Section 1: Provided by Chassis

The Chassis provides the following to all consuming projects:

### 1.1 Design Tokens & CSS

- **CSS Reset**: Impeccable baseline (github.com/pbakaus/impeccable)
  - Universal box-sizing, antialiased rendering, optimized typography
  - Imported as `@phenotype/docs/styles/impeccable.css`

- **Color System**: Dark-first theme tokens
  - Background: `#0a0e27` (dark navy)
  - Text: `#e0e6ed` (light gray)
  - Accent: `#4f9ff0` (sky blue)
  - Success/Warning/Error palette

- **Typography Scale**: System sans-serif with monospace fallback
  - Body: 16px (1rem)
  - Headings: H1 (2.5rem) → H6 (1.125rem)
  - Code: `font-family: 'Menlo', 'Monaco', monospace`

- **Component Library Tokens**: Radix UI / shadcn integration
  - Button variants (primary, secondary, ghost, destructive)
  - Card and layout spacing systems
  - Modal and dialog theming

### 1.2 VitePress Configuration Builder

**Function**: `createPhenotypeConfig(siteMeta)`

```typescript
// From @phenotype/docs/config

export interface SiteMeta {
  base: string              // Document base path (e.g., '/' or '/AgilePlus/')
  repoName: string          // Repository name (for sidebar/breadcrumbs)
  title?: string            // Site title (default: repoName)
  description?: string      // Meta description
  socialImage?: string      // OG image URL
}

export function createPhenotypeConfig(meta: SiteMeta): DefaultTheme.Config
```

**Returns**: Fully configured VitePress config object with:
- Sidebar navigation structure
- Search configuration
- Theme settings (dark mode default)
- Footer and breadcrumb navigation
- Analytics integration hooks

### 1.3 Theme Components

Pre-built Vue components for common documentation patterns:

- `PhenotypeLayout`: Main layout wrapper (header, sidebar, footer)
- `SidebarNav`: Auto-generated sidebar from frontmatter
- `CodeBlock`: Syntax-highlighted code with copy button
- `ADRTemplate`: Architecture Decision Record format
- `FRTraceTable`: Functional Requirement traceability table
- `SpecStatusTracker`: PRD/ADR/FR status dashboard
- `TimelineView`: Worklog and timeline visualization

### 1.4 Governance Patterns & Hooks

- **FR Traceability Hook**: Validates test file references to FR-XXX-NNN IDs
- **Spec Verification Integration**: Reads .agileplus/specs/ and generates dashboard
- **Worklog Integration**: Parses docs/worklogs/ and renders activity feed
- **Change Log Parser**: Converts CHANGELOG.md to rich event timeline

---

## Section 2: Consumed by Remotes

Projects using the Chassis must provide or configure:

### 2.1 Import in `docs/package.json`

```json
{
  "name": "docs",
  "private": true,
  "devDependencies": {
    "@phenotype/docs": "^0.1.0",
    "vitepress": "^1.6.3"
  }
}
```

Install via:
```bash
npm install @phenotype/docs
# or
bun install @phenotype/docs
# or
pnpm install @phenotype/docs
```

### 2.2 Configure `.npmrc` (if needed)

If GitHub Packages authentication is required:

```
@phenotype:registry=https://npm.pkg.github.com
//npm.pkg.github.com/:_authToken=${GITHUB_TOKEN}
```

**Local Development**: Use GitHub token from `~/.gitconfig` or `$GITHUB_TOKEN` env var.

### 2.3 Update `docs/.vitepress/config.ts` (or `.mts`)

Replace the config with:

```typescript
import { createPhenotypeConfig } from '@phenotype/docs/config'
import { createSiteMeta } from './site-meta.mjs'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'MyProject'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

const siteMeta = createSiteMeta({ base: docsBase, repoName })

export default createPhenotypeConfig(siteMeta)
```

**Create `docs/.vitepress/site-meta.mjs`**:

```javascript
export function createSiteMeta({ base, repoName }) {
  return {
    base,
    repoName,
    title: `${repoName} Documentation`,
    description: `Documentation for ${repoName}`,
    socialImage: `https://raw.githubusercontent.com/KooshaPari/${repoName}/main/docs-meta.png`
  }
}
```

### 2.4 Import Theme in `docs/.vitepress/theme/index.ts`

If you have a custom theme file:

```typescript
import { getPhenotypeTheme } from '@phenotype/docs/theme'
import Layout from './Layout.vue'

export default {
  extends: getPhenotypeTheme(),
  Layout,
  // Override or add custom components
}
```

### 2.5 Provide Required Documentation Files

The Chassis expects projects to maintain:

| File | Purpose | Template |
|------|---------|----------|
| `/PRD.md` | Product Requirements Document | Reference `/docs/reference/PHENOTYPE_DOCS_CHASSIS_INTERFACE.md` (this file) |
| `/ADR.md` | Architecture Decision Records | See `/docs/adr/` in canonical |
| `/FUNCTIONAL_REQUIREMENTS.md` | Functional requirements with FR-XXX-NNN IDs | See global CLAUDE.md |
| `/PLAN.md` | Phased work breakdown with DAG | See `/docs/reference/PHASED_WBS_TEMPLATE.md` |
| `/USER_JOURNEYS.md` | User journey flows | See `/docs/reference/USER_JOURNEY_TEMPLATE.md` |
| `docs/worklogs/` | Per-phase worklog entries | Format: `{PHASE}-worklog.md` |
| `docs/reference/` | Quick references, trackers, CODE_ENTITY_MAP | See docs/reference/ in canonical |

---

## Section 3: Integration Points

### 3.1 File Paths & Imports

| What | Import Path | Notes |
|------|-------------|-------|
| CSS Reset | `@phenotype/docs/styles/impeccable.css` | Add to `docs/.vitepress/theme/globals.css` |
| Config Builder | `@phenotype/docs/config` | Main entry point for `createPhenotypeConfig()` |
| Theme Components | `@phenotype/docs/components` | Export: `SidebarNav`, `CodeBlock`, `ADRTemplate`, etc. |
| Theme hooks | `@phenotype/docs/theme` | Export: `getPhenotypeTheme()`, `useSpecVerification()` |
| Type definitions | `@phenotype/docs` | TypeScript types for SiteMeta, Config, etc. |

### 3.2 Directory Structure in Consumer

```
myproject/
├── docs/
│   ├── .vitepress/
│   │   ├── config.ts          # Imports createPhenotypeConfig()
│   │   ├── site-meta.mjs      # Local site metadata
│   │   ├── theme/
│   │   │   ├── index.ts       # Extends getPhenotypeTheme()
│   │   │   └── globals.css    # Imports @phenotype/docs/styles
│   │   └── public/
│   ├── index.md               # Docs home
│   ├── guides/                # How-tos
│   ├── reference/             # Quick refs, trackers, maps
│   ├── adr/                   # Architecture decisions
│   ├── worklogs/              # Phase worklog entries
│   └── package.json           # Includes @phenotype/docs dependency
├── .npmrc                      # GitHub Packages auth (if private)
├── PRD.md                      # Root-level spec files
├── ADR.md
├── FUNCTIONAL_REQUIREMENTS.md
├── PLAN.md
└── USER_JOURNEYS.md
```

### 3.3 Code Examples: Using Chassis Components

**Example: Custom ADR template**

```vue
<!-- docs/adr/0001-my-decision.md -->

---
status: accepted
date: 2026-03-29
---

# ADR-001: My Architecture Decision

<ADRTemplate
  context="Why we're making this decision"
  decision="What we decided"
  consequences="Impact on the codebase"
/>
```

**Example: Inline FR traceability table**

```vue
<!-- docs/reference/FR_TRACKER.md -->

<FRTraceTable
  spec="FUNCTIONAL_REQUIREMENTS.md"
  filters={{ category: "AUTH" }}
/>
```

**Example: Spec verification dashboard**

```vue
<!-- docs/index.md -->

<SpecStatusTracker
  repoName="MyProject"
  showPRD={true}
  showFRs={true}
  showTests={true}
/>
```

---

## Section 4: Success Criteria

### 4.1 Build Success

- `bun install @phenotype/docs` completes without errors
- `bun docs:build` completes in < 5 seconds
- No console warnings about missing components or imports

### 4.2 Theme Rendering

- Homepage loads with Phenotype dark theme
- Sidebar navigation displays correctly
- Syntax highlighting works in code blocks
- Responsive layout on mobile (< 768px)

### 4.3 Integration Across Repos (3+)

Successful integration must be verified in at least **3 projects**:
1. AgilePlus (canonical reference)
2. phenotype-shared (test subject)
3. One additional repo (e.g., thegent or heliosCLI)

For each:
- [ ] @phenotype/docs installed and up-to-date
- [ ] docs/.vitepress/config.ts uses createPhenotypeConfig()
- [ ] `npm docs:build` succeeds with no errors
- [ ] docs-dist/index.html opens in browser with correct theme
- [ ] Sidebar navigation populated
- [ ] Search functionality works

### 4.4 Specification Metadata

Projects using Chassis must have:
- [ ] `/PRD.md` with epic/story structure
- [ ] `/FUNCTIONAL_REQUIREMENTS.md` with FR-XXX-NNN IDs
- [ ] `/docs/reference/FR_TRACKER.md` linking FRs to code
- [ ] `/docs/reference/CODE_ENTITY_MAP.md` (forward and reverse)
- [ ] Tests tagged with `@pytest.mark.requirement("FR-XXX-NNN")` or equivalent

### 4.5 Governance Compliance

Chassis integration verifies:
- [ ] All FRs have corresponding tests
- [ ] No orphan tests (all tests map to FRs)
- [ ] Lint/type checks pass (0 errors)
- [ ] Coverage meets threshold (80%+)
- [ ] Architecture boundaries enforced (import-linter, eslint-plugin-boundaries)

---

## Section 5: Versioning & Updates

### 5.1 Semantic Versioning

`@phenotype/docs@MAJOR.MINOR.PATCH`

- **MAJOR**: Breaking changes (config API, theme restructure)
- **MINOR**: New components, new governance hooks
- **PATCH**: Bug fixes, documentation updates

### 5.2 Update Strategy

Consumer projects:
1. Update `docs/package.json`: `"@phenotype/docs": "^0.X.0"` (caret = allows MINOR + PATCH)
2. Run `npm install @phenotype/docs@latest`
3. Test: `npm docs:build && npm docs:preview`
4. Commit and push

The Chassis aims for **backward compatibility** within major versions.

### 5.3 Breaking Changes

When a breaking change occurs:
- Update to MAJOR version (e.g., 0.X.0 → 1.0.0)
- Document migration guide in `@phenotype/docs` CHANGELOG
- Update all consumer projects in coordinated PR stack

---

## FAQ

**Q: Do I have to use Chassis components?**
A: No. The CSS and config are required, but theme components are opt-in. You can write custom VitePress themes that extend the base.

**Q: Can I override Chassis styles?**
A: Yes. Import Chassis CSS first, then override in your `docs/.vitepress/theme/globals.css`.

**Q: What if my project is not a Phenotype member?**
A: You can still use `@phenotype/docs` — it's published to GitHub Packages. Install and configure as above.

**Q: How do I report bugs or suggest features?**
A: Open issues in the phenotype monorepo under `packages/phenotype-docs/`.

**Q: Is Chassis tied to AgilePlus?**
A: No. Chassis is a documentation + design system. AgilePlus is a separate (but complementary) governance system.

---

## Related Docs

- **AgilePlus Governance Chassis**: See `/docs/reference/AGILEPLUS_GOVERNANCE_CHASSIS.md`
- **VitePress Documentation**: https://vitepress.dev
- **Design Tokens Reference**: See `@phenotype/docs` package in monorepo
- **Impeccable CSS**: https://github.com/pbakaus/impeccable

