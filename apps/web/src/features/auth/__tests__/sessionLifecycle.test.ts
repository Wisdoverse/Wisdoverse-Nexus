import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useAuthStore, resetAuthStore } from '../authStore'
import { sessionTimers, IDLE_TIMEOUT_MS, EXPIRY_CHECK_INTERVAL_MS } from '../sessionTimers'

const createMockSession = (overrides = {}) => ({
  token: 'test-token',
  memberId: 'member-123',
  tenantId: 'tenant-456',
  expiresAt: Date.now() + 3600000,
  ...overrides,
})

describe('session lifecycle', () => {
  beforeEach(() => {
    vi.useFakeTimers()
    resetAuthStore()
    localStorage.clear()
    sessionStorage.clear()
    sessionTimers.stop()
    sessionTimers.clearLogoutCallback()
  })

  afterEach(() => {
    vi.useRealTimers()
    sessionTimers.stop()
    sessionTimers.clearLogoutCallback()
    vi.restoreAllMocks()
  })

  describe('token expiry detection', () => {
    it('should trigger logout when token expires', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession({ expiresAt: Date.now() + 500 }))

      const logoutSpy = vi.spyOn(useAuthStore.getState(), 'logout')

      sessionTimers.start()

      vi.advanceTimersByTime(EXPIRY_CHECK_INTERVAL_MS + 1000)

      expect(logoutSpy).toHaveBeenCalled()
    })

    it('should not expire when token is still valid', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession({ expiresAt: Date.now() + 3600000 }))

      sessionTimers.start()

      vi.advanceTimersByTime(EXPIRY_CHECK_INTERVAL_MS * 5)

      const state = useAuthStore.getState()
      expect(state.status).toBe('authenticated')
    })
  })

  describe('idle timeout detection', () => {
    it('should trigger logout after idle timeout', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      const logoutSpy = vi.spyOn(useAuthStore.getState(), 'logout')

      sessionTimers.start()

      vi.advanceTimersByTime(IDLE_TIMEOUT_MS + EXPIRY_CHECK_INTERVAL_MS)

      expect(logoutSpy).toHaveBeenCalled()
    })

    it('should not logout when user is active', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      const logoutSpy = vi.spyOn(useAuthStore.getState(), 'logout')

      sessionTimers.start()

      for (let i = 0; i < 10; i++) {
        vi.advanceTimersByTime(IDLE_TIMEOUT_MS / 10)
        sessionTimers.recordActivity()
      }

      expect(logoutSpy).not.toHaveBeenCalled()
    })

    it('should update lastActivityAt on user activity', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      sessionTimers.start()

      const initialActivity = useAuthStore.getState().lastActivityAt
      vi.advanceTimersByTime(1000)

      sessionTimers.recordActivity()

      const newActivity = useAuthStore.getState().lastActivityAt
      expect(newActivity).toBeGreaterThan(initialActivity!)
    })
  })

  describe('session timers lifecycle', () => {
    it('should not start timers when not authenticated', () => {
      sessionTimers.start()

      vi.advanceTimersByTime(IDLE_TIMEOUT_MS * 2)

      const state = useAuthStore.getState()
      expect(state.status).toBe('anonymous')
    })

    it('should stop timers when logout callback is invoked', () => {
      const { login, logout } = useAuthStore.getState()
      login(createMockSession())

      sessionTimers.setLogoutCallback(() => {
        sessionTimers.stop()
      })
      sessionTimers.start()

      expect(sessionTimers.isRunning()).toBe(true)

      logout()
      sessionTimers.stop()

      expect(sessionTimers.isRunning()).toBe(false)
    })

    it('should restart timers on re-login', () => {
      const { login, logout } = useAuthStore.getState()

      login(createMockSession())
      sessionTimers.start()
      expect(sessionTimers.isRunning()).toBe(true)

      logout()
      sessionTimers.stop()
      expect(sessionTimers.isRunning()).toBe(false)

      login(createMockSession())
      sessionTimers.start()
      expect(sessionTimers.isRunning()).toBe(true)
    })
  })

  describe('activity tracking', () => {
    it('should track mousemove as activity', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      sessionTimers.start()
      sessionTimers.setupActivityListeners()

      const initialActivity = useAuthStore.getState().lastActivityAt
      vi.advanceTimersByTime(1000)

      window.dispatchEvent(new Event('mousemove'))

      const newActivity = useAuthStore.getState().lastActivityAt
      expect(newActivity).toBeGreaterThan(initialActivity!)
    })

    it('should track keydown as activity', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      sessionTimers.start()
      sessionTimers.setupActivityListeners()

      const initialActivity = useAuthStore.getState().lastActivityAt
      vi.advanceTimersByTime(1000)

      window.dispatchEvent(new Event('keydown'))

      const newActivity = useAuthStore.getState().lastActivityAt
      expect(newActivity).toBeGreaterThan(initialActivity!)
    })

    it('should track click as activity', () => {
      const { login } = useAuthStore.getState()
      login(createMockSession())

      sessionTimers.start()
      sessionTimers.setupActivityListeners()

      const initialActivity = useAuthStore.getState().lastActivityAt
      vi.advanceTimersByTime(1000)

      window.dispatchEvent(new Event('click'))

      const newActivity = useAuthStore.getState().lastActivityAt
      expect(newActivity).toBeGreaterThan(initialActivity!)
    })
  })
})
