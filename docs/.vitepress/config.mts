<<<<<<< HEAD
import { createPhenotypeConfig } from '@phenotype/docs/config'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'AgilePlus'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

import { createSiteMeta } from './site-meta.mjs'

const siteMeta = createSiteMeta({ base: docsBase, repoName })

export default createPhenotypeConfig(siteMeta)
=======
import { withMermaid } from 'vitepress-plugin-mermaid'
import type { DefaultTheme } from 'vitepress'

const referenceSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Reference',
    items: [
      { text: 'AgilePlus WP quick reference', link: '/reference/AGILEPLUS_WP_QUICK_REFERENCE_2026-03-30' },
      { text: 'Code ↔ entity map', link: '/reference/CODE_ENTITY_MAP' },
      { text: 'Configuration standards', link: '/reference/CONFIGURATION_STANDARDS' },
      { text: 'FR tracker', link: '/reference/FR_TRACKER' },
      { text: 'Phase 1 execution plan', link: '/reference/PHASE_1_EXECUTION_PLAN' },
      { text: 'Traceability map', link: '/reference/TRACEABILITY_MAP' },
      { text: 'Validation standards', link: '/reference/VALIDATION_STANDARDS' },
    ],
  },
]

const adoptionSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Crate adoption',
    items: [
      { text: 'Overview', link: '/adoption/' },
      { text: 'phenotype-config-core', link: '/adoption/phenotype-config-core' },
      { text: 'phenotype-crypto', link: '/adoption/phenotype-crypto' },
      { text: 'phenotype-error-core', link: '/adoption/phenotype-error-core' },
      { text: 'phenotype-health', link: '/adoption/phenotype-health' },
      { text: 'phenotype-iter', link: '/adoption/phenotype-iter' },
      { text: 'phenotype-logging', link: '/adoption/phenotype-logging' },
      { text: 'phenotype-port-traits', link: '/adoption/phenotype-port-traits' },
      { text: 'phenotype-retry', link: '/adoption/phenotype-retry' },
      { text: 'phenotype-string', link: '/adoption/phenotype-string' },
      { text: 'phenotype-time', link: '/adoption/phenotype-time' },
    ],
  },
]

const overviewSidebar: DefaultTheme.SidebarItem[] = [
  {
    text: 'Overview',
    items: [
      { text: 'Home', link: '/' },
      { text: 'Architecture', link: '/architecture' },
      { text: 'Defensive patterns', link: '/DEFENSIVE_PATTERNS' },
      { text: 'LOC reduction opportunities', link: '/LOC_REDUCTION_OPPORTUNITIES' },
      { text: 'Work log', link: '/WORKLOG' },
    ],
  },
  {
    text: 'Sections',
    items: [
      { text: 'Guide', link: '/guide/' },
      { text: 'Reference', link: '/reference/TRACEABILITY_MAP' },
      { text: 'Governance', link: '/governance/ADR-001-external-package-adoption' },
      { text: 'Adoption', link: '/adoption/' },
    ],
  },
  {
    text: 'Languages',
    collapsed: true,
    items: [
      { text: 'فارسی', link: '/fa/' },
      { text: 'فارسی (لاتین)', link: '/fa-Latn/' },
      { text: '简体中文', link: '/zh-CN/' },
      { text: '繁體中文', link: '/zh-TW/' },
    ],
  },
]

export default withMermaid({
  title: 'phenotype-infrakit',
  description: 'Rust infrastructure toolkit: event sourcing, caching, policy evaluation, and state machine crates.',
  appearance: 'dark',
  lastUpdated: true,
  srcExclude: ['worklogs/**', 'research/**', 'reports/**', 'sessions/**', 'audits/**'],
  themeConfig: {
    nav: [
      { text: 'Home', link: '/' },
      { text: 'Guide', link: '/guide/' },
      { text: 'Reference', link: '/reference/TRACEABILITY_MAP' },
      { text: 'Governance', link: '/governance/ADR-001-external-package-adoption' },
      { text: 'Adoption', link: '/adoption/' },
      { text: 'Architecture', link: '/architecture' },
    ],
    sidebar: {
      '/reference/': referenceSidebar,
      '/guide/': [
        {
          text: 'Guide',
          items: [{ text: 'Getting started', link: '/guide/' }],
        },
      ],
      '/governance/': [
        {
          text: 'Governance',
          items: [
            {
              text: 'ADR-001 external package adoption',
              link: '/governance/ADR-001-external-package-adoption',
            },
          ],
        },
      ],
      '/adoption/': adoptionSidebar,
      '/fa/': overviewSidebar,
      '/fa-Latn/': overviewSidebar,
      '/zh-CN/': overviewSidebar,
      '/zh-TW/': overviewSidebar,
      '/': overviewSidebar,
    },
    search: { provider: 'local' },
  },
  mermaid: { theme: 'dark' },
})
>>>>>>> origin/main
