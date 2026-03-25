import { defineConfig } from 'vitepress'
export default defineConfig({
  title: 'Documentation',
  description: 'Project Documentation',
  outDir: '../docs-dist',
  themeConfig: {
    nav: [{ text: 'Home', link: '/' }],
    sidebar: [{ text: 'Overview', link: '/' }]
  }
})
