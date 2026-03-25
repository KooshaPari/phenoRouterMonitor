import { defineConfig } from 'vitepress'
export default defineConfig({
  title: 'PHENOTYPE INFRAKIT',
  description: 'Documentation for PHENOTYPE INFRAKIT',
  outDir: '../docs-dist',
  themeConfig: {
    nav: [{ text: 'Home', link: '/' }],
    sidebar: [{ text: 'Overview', link: '/' }]
  }
})
