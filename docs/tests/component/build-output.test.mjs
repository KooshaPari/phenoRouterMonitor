import assert from 'node:assert/strict'
import { existsSync } from 'node:fs'
import test from 'node:test'

test('vitepress config exists', () => {
  assert.ok(
    existsSync('.vitepress/config.mts') || existsSync('.vitepress/config.ts')
  )
})
