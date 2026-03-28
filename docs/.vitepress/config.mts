import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid({
  title: 'phenotype-infrakit',
  description: 'Rust infrastructure toolkit: event sourcing, caching, policy evaluation, and state machine crates.',
  appearance: 'dark',
  lastUpdated: true,
  themeConfig: {
    nav: [{ text: 'Home', link: '/' }],
    sidebar: [],
    search: { provider: 'local' },
  },
  mermaid: { theme: 'dark' },
})
