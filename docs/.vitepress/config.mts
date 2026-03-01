import { defineConfig } from 'vitepress'
import { withMermaid } from 'vitepress-plugin-mermaid'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'AgilePlus'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

export default withMermaid(
  defineConfig({
    title: 'AgilePlus',
    description: 'Spec-driven development engine',
    lang: 'en-US',
    base: docsBase,
    lastUpdated: true,
    cleanUrls: true,
    appearance: 'dark',

    head: [
      ['link', { rel: 'icon', href: `${docsBase}favicon.ico` }],
      ['link', { rel: 'preconnect', href: 'https://fonts.googleapis.com' }],
      ['link', { rel: 'preconnect', href: 'https://fonts.gstatic.com', crossorigin: '' }],
    ],

    themeConfig: {
      siteTitle: 'AgilePlus',

      nav: [
        { text: 'Docs', link: '/guide/getting-started' },
        { text: 'Concepts', link: '/concepts/spec-driven-dev' },
        { text: 'SDK', link: '/sdk/grpc-api' },
        { text: 'Reference', link: '/reference/cli' },
      ],

      sidebar: {
        '/': [
          {
            text: 'Introduction',
            items: [
              { text: 'Getting Started', link: '/guide/getting-started' },
              { text: 'Quick Start', link: '/guide/quick-start' },
            ]
          },
          {
            text: 'Concepts',
            items: [
              { text: 'Spec-Driven Development', link: '/concepts/spec-driven-dev' },
              { text: 'Governance & Audit', link: '/concepts/governance' },
              { text: 'Agent Dispatch', link: '/concepts/agent-dispatch' },
              { text: 'Feature Lifecycle', link: '/concepts/feature-lifecycle' },
            ]
          },
          {
            text: 'For Agents',
            collapsed: true,
            items: [
              { text: 'Prompt Format', link: '/agents/prompt-format' },
              { text: 'Governance Constraints', link: '/agents/governance-constraints' },
              { text: 'Harness Integration', link: '/agents/harness-integration' },
            ]
          },
          {
            text: 'For Developers',
            collapsed: true,
            items: [
              { text: 'Contributing', link: '/developers/contributing' },
              { text: 'Extending AgilePlus', link: '/developers/extending' },
              { text: 'Testing Guide', link: '/developers/testing' },
            ]
          },
          {
            text: 'SDK / API',
            collapsed: true,
            items: [
              { text: 'gRPC API', link: '/sdk/grpc-api' },
              { text: 'MCP Tools', link: '/sdk/mcp-tools' },
              { text: 'Storage Port', link: '/sdk/storage-port' },
              { text: 'VCS Port', link: '/sdk/vcs-port' },
            ]
          },
          {
            text: 'Workflow Phases',
            collapsed: true,
            items: [
              { text: 'Specify', link: '/workflow/specify' },
              { text: 'Clarify', link: '/workflow/clarify' },
              { text: 'Research', link: '/workflow/research' },
              { text: 'Plan', link: '/workflow/plan' },
              { text: 'Tasks & Work Packages', link: '/workflow/tasks' },
              { text: 'Implement', link: '/workflow/implement' },
              { text: 'Review', link: '/workflow/review' },
              { text: 'Accept', link: '/workflow/accept' },
              { text: 'Merge', link: '/workflow/merge' },
            ]
          },
          {
            text: 'Process',
            collapsed: true,
            items: [
              { text: 'Constitution', link: '/process/constitution' },
              { text: 'Checklists', link: '/process/checklists' },
              { text: 'Analyze', link: '/process/analyze' },
              { text: 'Retrospectives', link: '/process/retrospective' },
              { text: 'Status & Dashboard', link: '/process/status-dashboard' },
            ]
          },
          {
            text: 'Guide',
            items: [
              { text: 'Project Setup', link: '/guide/init' },
              { text: 'Core Workflow', link: '/guide/workflow' },
              { text: 'Triage & Queue', link: '/guide/triage' },
              { text: 'Configuration', link: '/guide/configuration' },
              { text: 'Sync (Plane + GitHub)', link: '/guide/sync' },
            ]
          },
          {
            text: 'Architecture',
            items: [
              { text: 'Overview', link: '/architecture/overview' },
              { text: 'Domain Model', link: '/architecture/domain-model' },
              { text: 'Port Traits', link: '/architecture/ports' },
            ]
          },
          {
            text: 'Roadmap & Planning',
            collapsed: true,
            items: [
              { text: 'Roadmap', link: '/roadmap/' },
              { text: 'Release Notes', link: '/roadmap/release-notes' },
            ]
          },
          {
            text: 'Reference',
            items: [
              { text: 'CLI Commands', link: '/reference/cli' },
              { text: 'Crate Map', link: '/reference/crates' },
              { text: 'Sub-commands', link: '/reference/subcommands' },
              { text: 'Environment Variables', link: '/reference/env-vars' },
            ]
          },
          {
            text: 'Doc System',
            collapsed: true,
            items: [
              { text: 'Documentation Layers', link: '/doc-system/layers' },
              { text: 'Frontmatter Schema', link: '/doc-system/frontmatter' },
              { text: 'Federation & PhenoDocs', link: '/doc-system/federation' },
            ]
          },
          {
            text: 'Examples',
            items: [
              { text: 'Full Pipeline', link: '/examples/full-pipeline' },
              { text: 'Triage Workflow', link: '/examples/triage-workflow' },
              { text: 'Agent Integration', link: '/examples/agent-integration' },
            ]
          },
        ],
      },

      socialLinks: [
        { icon: 'github', link: `https://github.com/KooshaPari/${repoName}` }
      ],

      footer: {
        message: 'MIT License',
        copyright: '© 2025 Phenotype',
      },

      search: {
        provider: 'local'
      },

      editLink: {
        pattern: `https://github.com/KooshaPari/${repoName}/edit/main/docs/:path`,
        text: 'Edit this page',
      },

      outline: {
        level: [2, 3],
        label: 'On this page',
      },
    },

    markdown: {
      lineNumbers: true,
      theme: {
        light: 'github-light',
        dark: 'vitesse-dark',
      },
    },
    ignoreDeadLinks: true,

    transformPageData(pageData) {
      // Expose audience frontmatter for client-side filtering
      if (pageData.frontmatter?.audience) {
        ;(pageData as any).audience = pageData.frontmatter.audience
      }
    },
  })
)
