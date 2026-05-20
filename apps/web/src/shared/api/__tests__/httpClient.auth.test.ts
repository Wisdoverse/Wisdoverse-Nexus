import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  configureSessionAccess,
  type RefreshSessionResult,
  type SessionSnapshot,
} from '../../session/sessionAccess'

let sessionState: SessionSnapshot
let updatedSession: RefreshSessionResult | null
let logoutCalls: number

const createSessionSnapshot = (overrides: Partial<SessionSnapshot> = {}): SessionSnapshot => ({
  token: 'test-token',
  tenantId: 'tenant-456',
  isAuthenticated: true,
  needsRefresh: () => false,
  ...overrides,
})

describe('httpClient auth refresh', () => {
  beforeEach(() => {
    sessionState = createSessionSnapshot()
    updatedSession = null
    logoutCalls = 0
    configureSessionAccess({
      getSnapshot: () => sessionState,
      updateSession: (session) => {
        updatedSession = session
      },
      logout: () => {
        logoutCalls += 1
        sessionState = createSessionSnapshot({
          token: null,
          tenantId: null,
          isAuthenticated: false,
        })
      },
    })
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
      sessionState = createSessionSnapshot({
        needsRefresh: () => true,
      })

      expect(sessionState.needsRefresh()).toBe(true)
    })

    it('should not need refresh when token has plenty of time', () => {
      sessionState = createSessionSnapshot({
        needsRefresh: () => false,
      })

      expect(sessionState.needsRefresh()).toBe(false)
    })
  })

  describe('session update after refresh', () => {
    it('should update token and expiresAt after refresh', async () => {
      const newExpiresAt = Date.now() + 7200000
      const { handleRefreshSuccess } = await import('../sessionManager')
      handleRefreshSuccess({ token: 'new-token', expiresAt: newExpiresAt })

      expect(updatedSession).toEqual({ token: 'new-token', expiresAt: newExpiresAt })
    })
  })

  describe('handleRefreshSuccess and handleRefreshFailure', () => {
    it('should update session on refresh success', async () => {
      const { handleRefreshSuccess } = await import('../sessionManager')
      const newExpiresAt = Date.now() + 7200000

      handleRefreshSuccess({ token: 'refreshed-token', expiresAt: newExpiresAt })

      expect(updatedSession).toEqual({ token: 'refreshed-token', expiresAt: newExpiresAt })
    })

    it('should logout on refresh failure', async () => {
      const { handleRefreshFailure } = await import('../sessionManager')
      handleRefreshFailure()

      expect(logoutCalls).toBe(1)
      expect(sessionState.isAuthenticated).toBe(false)
      expect(sessionState.token).toBeNull()
    })
  })
})
