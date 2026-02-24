import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useAuthStore, resetAuthStore } from '../../../features/auth/authStore'

const createMockSession = (overrides = {}) => ({
  token: 'test-token',
  memberId: 'member-123',
  tenantId: 'tenant-456',
  expiresAt: Date.now() + 3600000,
  ...overrides,
})

describe('httpClient auth refresh', () => {
  beforeEach(() => {
    resetAuthStore()
    localStorage.clear()
    sessionStorage.clear()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('sessionManager', () => {
    it('should create single-flight refresh lock', async () => {
      const { sessionManager } = await import('../sessionManager')

      let refreshCalls = 0
      const refreshFn = async () => {
        refreshCalls++
        await new Promise((r) => setTimeout(r, 50))
        return { token: 'new-token', expiresAt: Date.now() + 7200000 }
      }

      const results = await Promise.all([
        sessionManager.executeWithRefreshLock(refreshFn),
        sessionManager.executeWithRefreshLock(refreshFn),
        sessionManager.executeWithRefreshLock(refreshFn),
      ])

      expect(refreshCalls).toBe(1)
      expect(results[0]).toEqual(results[1])
      expect(results[1]).toEqual(results[2])
    })

    it('should clear refresh lock after completion', async () => {
      const { sessionManager } = await import('../sessionManager')

      const refreshFn = async () => ({ token: 'new-token', expiresAt: 1 })

      await sessionManager.executeWithRefreshLock(refreshFn)

      expect(sessionManager.isRefreshing()).toBe(false)

      const secondRefreshFn = async () => ({ token: 'another-token', expiresAt: 2 })
      await sessionManager.executeWithRefreshLock(secondRefreshFn)

      expect(sessionManager.isRefreshing()).toBe(false)
    })

    it('should propagate refresh errors', async () => {
      const { sessionManager } = await import('../sessionManager')

      const refreshFn = async () => {
        throw new Error('Refresh failed')
      }

      await expect(sessionManager.executeWithRefreshLock(refreshFn)).rejects.toThrow('Refresh failed')
    })
  })

  describe('401 error classification', () => {
    it('should identify token_expired error as recoverable', async () => {
      const { isRecoverable401 } = await import('../sessionManager')

      expect(isRecoverable401({ code: 'token_expired' })).toBe(true)
      expect(isRecoverable401({ code: 'TOKEN_EXPIRED' })).toBe(true)
    })

    it('should identify token_revoked error as non-recoverable', async () => {
      const { isRecoverable401 } = await import('../sessionManager')

      expect(isRecoverable401({ code: 'token_revoked' })).toBe(false)
      expect(isRecoverable401({ code: 'token_invalid' })).toBe(false)
      expect(isRecoverable401({ code: 'session_expired' })).toBe(false)
    })

    it('should handle missing error code', async () => {
      const { isRecoverable401 } = await import('../sessionManager')

      expect(isRecoverable401({})).toBe(false)
      expect(isRecoverable401(null)).toBe(false)
      expect(isRecoverable401(undefined)).toBe(false)
    })
  })

  describe('token needs refresh detection', () => {
    it('should detect token needs refresh when expiring soon', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession({ expiresAt: Date.now() + 30000 }))

      const { needsRefresh } = useAuthStore.getState()
      expect(needsRefresh()).toBe(true)
    })

    it('should not need refresh when token has plenty of time', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession({ expiresAt: Date.now() + 3600000 }))

      const { needsRefresh } = useAuthStore.getState()
      expect(needsRefresh()).toBe(false)
    })
  })

  describe('session update after refresh', () => {
    it('should update token and expiresAt after refresh', async () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      const newExpiresAt = Date.now() + 7200000
      const { updateSession } = useAuthStore.getState()
      updateSession({ token: 'new-token', expiresAt: newExpiresAt })

      const state = useAuthStore.getState()
      expect(state.token).toBe('new-token')
      expect(state.expiresAt).toBe(newExpiresAt)
    })
  })

  describe('handleRefreshSuccess and handleRefreshFailure', () => {
    it('should update session on refresh success', async () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      const { handleRefreshSuccess } = await import('../sessionManager')
      const newExpiresAt = Date.now() + 7200000

      handleRefreshSuccess({ token: 'refreshed-token', expiresAt: newExpiresAt })

      const state = useAuthStore.getState()
      expect(state.token).toBe('refreshed-token')
      expect(state.expiresAt).toBe(newExpiresAt)
    })

    it('should logout on refresh failure', async () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      const { handleRefreshFailure } = await import('../sessionManager')
      handleRefreshFailure()

      const state = useAuthStore.getState()
      expect(state.isAuthenticated).toBe(false)
      expect(state.token).toBeNull()
    })
  })
})
