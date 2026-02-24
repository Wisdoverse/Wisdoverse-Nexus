import { useAuthStore } from './authStore'

export const IDLE_TIMEOUT_MS = 30 * 60 * 1000
export const EXPIRY_CHECK_INTERVAL_MS = 10 * 1000

type LogoutCallback = () => void

interface SessionTimers {
  start: () => void
  stop: () => void
  isRunning: () => boolean
  recordActivity: () => void
  setupActivityListeners: () => void
  cleanupActivityListeners: () => void
  setLogoutCallback: (callback: LogoutCallback | null) => void
  clearLogoutCallback: () => void
}

let expiryCheckInterval: ReturnType<typeof setInterval> | null = null
let idleCheckInterval: ReturnType<typeof setInterval> | null = null
let activityListenersSetup = false
let logoutCallback: LogoutCallback | null = null

const activityEvents = ['mousemove', 'keydown', 'click', 'touchstart', 'scroll']

function handleActivity(): void {
  sessionTimers.recordActivity()
}

function checkTokenExpiry(): void {
  const state = useAuthStore.getState()
  if (!state.isAuthenticated || !state.expiresAt) return

  if (Date.now() >= state.expiresAt) {
    sessionTimers.stop()
    useAuthStore.getState().logout()
    logoutCallback?.()
  }
}

function checkIdleTimeout(): void {
  const state = useAuthStore.getState()
  if (!state.isAuthenticated || !state.lastActivityAt) return

  const idleTime = Date.now() - state.lastActivityAt
  if (idleTime >= IDLE_TIMEOUT_MS) {
    sessionTimers.stop()
    useAuthStore.getState().logout()
    logoutCallback?.()
  }
}

export const sessionTimers: SessionTimers = {
  start(): void {
    const state = useAuthStore.getState()
    if (!state.isAuthenticated) return

    if (expiryCheckInterval) {
      clearInterval(expiryCheckInterval)
    }
    if (idleCheckInterval) {
      clearInterval(idleCheckInterval)
    }

    expiryCheckInterval = setInterval(checkTokenExpiry, EXPIRY_CHECK_INTERVAL_MS)
    idleCheckInterval = setInterval(checkIdleTimeout, EXPIRY_CHECK_INTERVAL_MS)

    checkTokenExpiry()
    checkIdleTimeout()
  },

  stop(): void {
    if (expiryCheckInterval) {
      clearInterval(expiryCheckInterval)
      expiryCheckInterval = null
    }
    if (idleCheckInterval) {
      clearInterval(idleCheckInterval)
      idleCheckInterval = null
    }
    sessionTimers.cleanupActivityListeners()
  },

  isRunning(): boolean {
    return expiryCheckInterval !== null || idleCheckInterval !== null
  },

  recordActivity(): void {
    const { isAuthenticated, updateActivity } = useAuthStore.getState()
    if (isAuthenticated) {
      updateActivity()
    }
  },

  setupActivityListeners(): void {
    if (activityListenersSetup) return

    activityEvents.forEach((event) => {
      window.addEventListener(event, handleActivity, { passive: true })
    })
    activityListenersSetup = true
  },

  cleanupActivityListeners(): void {
    if (!activityListenersSetup) return

    activityEvents.forEach((event) => {
      window.removeEventListener(event, handleActivity)
    })
    activityListenersSetup = false
  },

  setLogoutCallback(callback: LogoutCallback | null): void {
    logoutCallback = callback
  },

  clearLogoutCallback(): void {
    logoutCallback = null
  },
}
