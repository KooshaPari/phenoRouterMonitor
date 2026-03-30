import { withMermaid } from 'vitepress-plugin-mermaid'

export default withMermaid({
  title: 'phenotype-infrakit',
  description: 'Rust infrastructure toolkit: event sourcing, caching, policy evaluation, and state machine crates.',
  appearance: 'dark',
  lastUpdated: true,
  // Large audit/research trees use Rust generics and `<` comparisons; Vue treats them as HTML. Ship as repo files, not VP pages.
  srcExclude: ['worklogs/**', 'research/**', 'reports/**', 'sessions/**', 'audits/**'],
  themeConfig: {
    nav: [{ text: 'Home', link: '/' }],
    sidebar: [],
    search: { provider: 'local' },
  },
  mermaid: { theme: 'dark' },
})
