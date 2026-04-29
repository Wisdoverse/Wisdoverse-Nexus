import { expect, test } from '@playwright/test'

test.describe('Auth + Rooms flow', () => {
  test('user can sign in and load empty rooms page', async ({ page }) => {
    await page.route('**/api/v1/rooms', async (route) => {
      if (route.request().method() === 'GET') {
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: '[]',
        })
        return
      }

      await route.continue()
    })

    await page.goto('/login')

    await page.getByLabel('Token').fill('test-token')
    await page.getByLabel('Member ID').fill('member-1')
    await page.getByRole('button', { name: 'Sign in' }).click()

    await expect(page).toHaveURL(/\/app\/rooms/)
    await expect(page.getByRole('heading', { name: 'Rooms' })).toBeVisible()
    await expect(page.getByText('No rooms yet. Create one to get started!')).toBeVisible()
  })
})
