import { defineConfig } from 'vitepress'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'AgilePlus'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

export default defineConfig({
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
      { text: 'Reference', link: '/reference/cli' },
      { text: 'Examples', link: '/examples/full-pipeline' },
    ],

    sidebar: {
      '/': [
        {
          text: 'Introduction',
          items: [
            { text: 'Getting Started', link: '/guide/getting-started' },
          ]
        },
        {
          text: 'Concepts',
          items: [
            { text: 'Spec-Driven Development', link: '/concepts/spec-driven-dev' },
            { text: 'Governance & Audit', link: '/concepts/governance' },
            { text: 'Agent Dispatch', link: '/concepts/agent-dispatch' },
          ]
        },
        {
          text: 'Guide',
          items: [
            { text: 'Project Setup', link: '/guide/init' },
            { text: 'Core Workflow', link: '/guide/workflow' },
            { text: 'Triage & Queue', link: '/guide/triage' },
            { text: 'Configuration', link: '/guide/configuration' },
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
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/reference/cli' },
            { text: 'Crate Map', link: '/reference/crates' },
            { text: 'Sub-commands', link: '/reference/subcommands' },
          ]
        },
        {
          text: 'Examples',
          items: [
            { text: 'Full Pipeline', link: '/examples/full-pipeline' },
            { text: 'Triage Workflow', link: '/examples/triage-workflow' },
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
})
