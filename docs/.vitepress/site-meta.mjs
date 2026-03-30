export function createSiteMeta({ base = '/' } = {}) {
  return {
    base,
    title: 'phenotype-infrakit',
    description: 'Infrastructure toolkit',
    themeConfig: {
      nav: [
        { text: 'Home', link: base || '/' },
        { text: 'Guide', link: '/guide/' },
        { text: 'Reference', link: '/reference/' },
      ],
    },
  }
}
