import { defineConfig } from 'vitepress'

const isPagesBuild = process.env.GITHUB_ACTIONS === 'true' || process.env.GITHUB_PAGES === 'true'
const repoName = process.env.GITHUB_REPOSITORY?.split('/')[1] || 'phenotype-infrakit'
const docsBase = isPagesBuild ? `/${repoName}/` : '/'

export default defineConfig({
  title: 'phenotype-infrakit',
  description: 'Rust infrastructure toolkit for event sourcing, caching, policy, and state machines',
  lang: 'en-US',
  base: docsBase,
  lastUpdated: true,
  cleanUrls: true,
  themeConfig: {
    siteTitle: 'phenotype-infrakit',
    nav: [{ text: 'Guide', link: '/guide/' }],
    sidebar: {
      '/guide/': [
        { text: 'Guide', items: [{ text: 'Getting Started', link: '/guide/' }] }
      ]
    },
    socialLinks: [{ icon: 'github', link: `https://github.com/KooshaPari/${repoName}` }],
    search: { provider: 'local' }
  },
  markdown: { lineNumbers: true },
  ignoreDeadLinks: true
})
