import assert from 'node:assert/strict'
import { test } from 'node:test'

import { createSiteMeta } from '../../.vitepress/site-meta.mjs'

test('site meta exposes the expected navigation', () => {
  const meta = createSiteMeta({ base: '/', repoName: 'AgilePlus' })

  assert.equal(meta.title, 'AgilePlus')
  assert.ok(meta.nav.some((item) => item.link === '/reference/cli'))
})
