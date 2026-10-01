import { expect, test } from '@playwright/test'

test.describe('Auth + Rooms flow', () => {
  test('user can sign in with a validated token and load the empty rooms page', async ({ page }) => {
    let sessionAuthorization: string | undefined
    let roomsAuthorization: string | undefined

    await page.route('**/api/v1/auth/session', async (route) => {
      sessionAuthorization = route.request().headers().authorization
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({
          memberId: 'member-1',
          memberType: 'human',
          expiresAt: Date.now() + 60_000,
          refreshSupported: false,
        }),
      })
    })

    await page.route('**/api/v1/rooms', async (route) => {
      if (route.request().method() === 'GET') {
        roomsAuthorization = route.request().headers().authorization
        await route.fulfill({
          status: 200,
          contentType: 'application/json',
          body: JSON.stringify({ rooms: [], total: 0 }),
        })
        return
      }

      await route.continue()
    })

    await page.goto('/login')

    await expect(page.getByLabel('Token')).toBeVisible()
    await expect(page.getByLabel('Member ID')).toHaveCount(0)
    await page.getByLabel('Token').fill('test-token')
    await page.getByRole('button', { name: 'Sign in' }).click()

    await expect(page).toHaveURL(/\/app\/rooms/)
    await expect(page.getByRole('heading', { name: 'Rooms' })).toBeVisible()
    await expect(page.getByText('No rooms yet. Create one to get started!')).toBeVisible()
    expect(sessionAuthorization).toBe('Bearer test-token')
    expect(roomsAuthorization).toBe('Bearer test-token')
  })

  test('invalid token stays on the login page and shows an error', async ({ page }) => {
    let sessionAuthorization: string | undefined
    let roomsRequested = false

    await page.route('**/api/v1/auth/session', async (route) => {
      sessionAuthorization = route.request().headers().authorization
      await route.fulfill({
        status: 401,
        contentType: 'application/json',
        body: JSON.stringify({ error: 'Invalid token' }),
      })
    })

    await page.route('**/api/v1/rooms', async (route) => {
      roomsRequested = true
      await route.fulfill({
        status: 200,
        contentType: 'application/json',
        body: JSON.stringify({ rooms: [], total: 0 }),
      })
    })

    await page.goto('/login')
    await expect(page.getByLabel('Member ID')).toHaveCount(0)
    await page.getByLabel('Token').fill('invalid-token')
    await page.getByRole('button', { name: 'Sign in' }).click()

    await expect(page).toHaveURL(/\/login$/)
    await expect(page.getByText('Login failed. Please check your credentials.')).toBeVisible()
    expect(sessionAuthorization).toBe('Bearer invalid-token')
    expect(roomsRequested).toBe(false)
  })
})
