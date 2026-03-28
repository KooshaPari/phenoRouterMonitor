import assert from 'node:assert/strict'
import { test } from 'node:test'

import { routes } from '../helpers/site-meta.mjs'

test('docs routes are expressed as expected', () => {
  assert.deepEqual(routes, [
    '/',
    '/guide/getting-started',
    '/guide/quick-start',
    '/concepts/spec-driven-dev',
    '/sdk/grpc-api',
    '/reference/cli',
    '/roadmap/',
    '/zh-CN/',
    '/zh-TW/',
    '/fa/',
    '/fa-Latn/',
  ])
})
