import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('zustand', () => ({
  create: (creator: (set: (partial: unknown) => void, get: () => unknown) => unknown) => {
    let state: Record<string, unknown> = {}
    const setState = (partial: unknown) => {
      if (typeof partial === 'function') {
        const updater = partial as (s: Record<string, unknown>) => Record<string, unknown>
        state = { ...state, ...updater(state) }
      } else {
        state = { ...state, ...(partial as Record<string, unknown>) }
      }
    }
    const getState = () => state
    state = creator(setState, getState) as Record<string, unknown>
    return { getState }
  },
}))

vi.mock('../../../shared/auth/tokenStorage', () => ({
  tokenStorage: {
    setToken: vi.fn(),
    clearToken: vi.fn(),
  },
}))

import { useAuthStore } from '../authStore'
import { tokenStorage } from '../../../shared/auth/tokenStorage'

describe('authStore', () => {
  beforeEach(async () => {
    await useAuthStore.getState().logout()
    vi.clearAllMocks()
  })

  it('persists token on login', async () => {
    await useAuthStore.getState().login({
      token: 'tkn',
      memberId: 'm1',
      tenantId: 'tenant-1',
    })

    expect(useAuthStore.getState().isAuthenticated).toBe(true)
    expect(tokenStorage.setToken).toHaveBeenCalledWith('tkn')
  })

  it('clears token on logout', async () => {
    await useAuthStore.getState().logout()
    expect(tokenStorage.clearToken).toHaveBeenCalledTimes(1)
  })
})
