import { beforeEach, describe, expect, it, vi } from 'vitest'

vi.mock('../../../shared/auth/tokenStorage', () => ({
  tokenStorage: { getToken: vi.fn(), setToken: vi.fn(), clearToken: vi.fn() },
}))
vi.mock('../../../shared/api/authApi', () => ({ authApi: { authenticate: vi.fn() } }))

import { useAuthStore } from '..'
import { tokenStorage } from '../../../shared/auth/tokenStorage'
import { authApi } from '../../../shared/api/authApi'

const session = { token: 'synthetic-token', memberId: 'synthetic-member', expiresAt: Date.now() + 60000 }

describe('mobile session lifecycle', () => {
  beforeEach(async () => {
    vi.resetAllMocks()
    await useAuthStore.getState().logout()
    vi.clearAllMocks()
  })

  it('validates a saved credential before restoring identity and the auth gate', async () => {
    vi.mocked(tokenStorage.getToken).mockResolvedValue(session.token)
    vi.mocked(authApi.authenticate).mockResolvedValue(session)
    const pending = useAuthStore.getState().restoreSession()
    expect(useAuthStore.getState().isHydrated).toBe(false)
    expect(useAuthStore.getState().isAuthenticated).toBe(false)
    await pending
    expect(authApi.authenticate).toHaveBeenCalledWith(session.token)
    expect(useAuthStore.getState()).toMatchObject({ ...session, isAuthenticated: true, isHydrated: true })
  })

  it('finishes startup anonymously when no credential was saved', async () => {
    vi.mocked(tokenStorage.getToken).mockResolvedValue(null)
    await useAuthStore.getState().restoreSession()
    expect(authApi.authenticate).not.toHaveBeenCalled()
    expect(useAuthStore.getState()).toMatchObject({ isHydrated: true, isAuthenticated: false })
  })

  it.each([401, 403])('discards a saved credential rejected with %i', async (status) => {
    vi.mocked(tokenStorage.getToken).mockResolvedValue(session.token)
    vi.mocked(authApi.authenticate).mockRejectedValue({ response: { status } })
    await useAuthStore.getState().restoreSession()
    expect(tokenStorage.clearToken).toHaveBeenCalledOnce()
    expect(useAuthStore.getState()).toMatchObject({ isHydrated: true, isAuthenticated: false, token: null })
  })

  it('preserves the saved credential on a network failure and lets the user retry', async () => {
    vi.mocked(tokenStorage.getToken).mockResolvedValue(session.token)
    vi.mocked(authApi.authenticate).mockRejectedValueOnce(new Error('offline')).mockResolvedValueOnce(session)
    await useAuthStore.getState().restoreSession()
    expect(tokenStorage.clearToken).not.toHaveBeenCalled()
    expect(useAuthStore.getState()).toMatchObject({ isAuthenticated: false, isHydrated: true })
    expect(useAuthStore.getState().restorationError).toContain('retry')
    await useAuthStore.getState().restoreSession()
    expect(useAuthStore.getState()).toMatchObject({ isAuthenticated: true, restorationError: null })
  })

  it('coalesces concurrent restoration requests', async () => {
    vi.mocked(tokenStorage.getToken).mockResolvedValue(session.token)
    vi.mocked(authApi.authenticate).mockResolvedValue(session)
    const first = useAuthStore.getState().restoreSession()
    const second = useAuthStore.getState().restoreSession()
    expect(first).toBe(second)
    await first
    expect(authApi.authenticate).toHaveBeenCalledOnce()
  })

  it('ignores a restoration response arriving after logout', async () => {
    let resolve!: (value: typeof session) => void
    vi.mocked(tokenStorage.getToken).mockResolvedValue(session.token)
    vi.mocked(authApi.authenticate).mockImplementation(() => new Promise((done) => { resolve = done }))
    const pending = useAuthStore.getState().restoreSession()
    await vi.waitFor(() => expect(authApi.authenticate).toHaveBeenCalledOnce())
    await useAuthStore.getState().logout()
    resolve(session)
    await pending
    expect(useAuthStore.getState()).toMatchObject({ isAuthenticated: false, isHydrated: true, token: null })
  })

  it('persists a login before opening the authenticated gate', async () => {
    let resolve!: () => void
    vi.mocked(tokenStorage.setToken).mockImplementation(() => new Promise((done) => { resolve = done }))
    const pending = useAuthStore.getState().login(session)
    await vi.waitFor(() => expect(tokenStorage.setToken).toHaveBeenCalledWith(session.token))
    expect(useAuthStore.getState().isAuthenticated).toBe(false)
    resolve()
    await pending
    expect(useAuthStore.getState().isAuthenticated).toBe(true)
  })

  it('does not authenticate when secure storage fails', async () => {
    vi.mocked(tokenStorage.setToken).mockRejectedValue(new Error('storage unavailable'))
    await expect(useAuthStore.getState().login(session)).rejects.toThrow('storage unavailable')
    expect(useAuthStore.getState().isAuthenticated).toBe(false)
  })

  it('clears the credential on logout', async () => {
    await useAuthStore.getState().login(session)
    await useAuthStore.getState().logout()
    expect(tokenStorage.clearToken).toHaveBeenCalledOnce()
    expect(useAuthStore.getState().isAuthenticated).toBe(false)
  })
})
