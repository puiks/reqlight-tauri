import { test, expect } from '@playwright/test'

test.describe('Smoke Tests', () => {
  test('page loads and shows welcome state', async ({ page }) => {
    await page.goto('/')
    await expect(page.locator('body')).toBeVisible()
    await expect(page.getByText('Welcome to Reqlight')).toBeVisible()
  })

  test('sidebar is visible with toolbar buttons', async ({ page }) => {
    await page.goto('/')
    // The "+" button for new collection has title "New Collection"
    await expect(page.getByTitle(/New Collection/)).toBeVisible()
  })

  test('can create a new collection', async ({ page }) => {
    await page.goto('/')
    await page.getByTitle(/New Collection/).click()
    await expect(page.getByText('New Collection', { exact: true })).toBeVisible()
  })
})
