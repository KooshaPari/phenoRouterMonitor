import { expect, test } from '@playwright/test'

const BASE_URL = process.env.BASE_URL || 'http://localhost:5173'

test.describe('AgilePlus docs', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto(BASE_URL)
  })

  test('homepage loads', async ({ page }) => {
    await expect(page).toHaveTitle(/AgilePlus/i)
  })

  for (const route of [
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
  ] as const) {
    test(`route ${route} is reachable`, async ({ page }) => {
      await page.goto(`${BASE_URL}${route}`)
      await expect(page.locator('#VPContent')).toBeVisible()
    })
  }
})
