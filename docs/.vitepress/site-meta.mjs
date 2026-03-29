export function createSiteMeta({ base = '/' } = {}) {
  return {
    base,
    title: 'apps/AgilePlus',
    description: 'Documentation',
    themeConfig: {
      nav: [
        { text: 'Home', link: base || '/' },
        { text: 'Guide', link: '/guide/' },
      ],
    },
  }
}
