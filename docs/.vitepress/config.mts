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

  head: [
    ['link', { rel: 'icon', href: `${docsBase}favicon.ico` }],
  ],

  themeConfig: {
    siteTitle: 'AgilePlus',

    nav: [
      { text: 'Guide', link: '/guide/getting-started' },
      { text: 'Architecture', link: '/architecture/overview' },
      { text: 'CLI Reference', link: '/reference/cli' },
      { text: 'Crates', link: '/reference/crates' }
    ],

    sidebar: {
      '/guide/': [
        {
          text: 'Guide',
          items: [
            { text: 'Getting Started', link: '/guide/getting-started' },
            { text: 'Workflow', link: '/guide/workflow' },
            { text: 'Configuration', link: '/guide/configuration' }
          ]
        }
      ],
      '/architecture/': [
        {
          text: 'Architecture',
          items: [
            { text: 'Overview', link: '/architecture/overview' },
            { text: 'Domain Model', link: '/architecture/domain-model' },
            { text: 'Port Traits', link: '/architecture/ports' }
          ]
        }
      ],
      '/reference/': [
        {
          text: 'Reference',
          items: [
            { text: 'CLI Commands', link: '/reference/cli' },
            { text: 'Crate Map', link: '/reference/crates' },
            { text: 'Sub-commands', link: '/reference/subcommands' }
          ]
        }
      ]
    },

    socialLinks: [
      { icon: 'github', link: `https://github.com/KooshaPari/${repoName}` }
    ],

    footer: {
      message: 'Released under the MIT License.',
      copyright: 'Copyright © 2025 Phenotype'
    },

    search: {
      provider: 'local'
    },

    editLink: {
      pattern: `https://github.com/KooshaPari/${repoName}/edit/main/docs/:path`,
      text: 'Edit this page on GitHub'
    },

    outline: {
      level: [2, 3],
      label: 'On this page'
    }
  },

  markdown: {
    lineNumbers: true,
    theme: {
      light: 'github-light',
      dark: 'github-dark'
    }
  },
  ignoreDeadLinks: true
})
