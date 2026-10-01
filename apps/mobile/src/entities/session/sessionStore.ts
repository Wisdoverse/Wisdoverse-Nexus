import { configureSessionAccess } from '../../shared/session/sessionAccess'
import { create } from 'zustand'

import type { Session, SessionStatus, User } from '../../shared/session/types'
import { tokenStorage } from '../../shared/auth/tokenStorage'
import { authApi } from '../../shared/api/authApi'

const REFRESH_THRESHOLD_MS = 60 * 1000

let sessionRevision = 0
let restoration: Promise<void> | null = null
let storageQueue: Promise<unknown> = Promise.resolve()

function withTokenStorage<T>(operation: () => Promise<T>): Promise<T> {
  const next = storageQueue.then(operation)
  storageQueue = next.catch(() => undefined)
  return next
}

function sessionState(session: Session) {
  return {
    token: session.token,
    memberId: session.memberId,
    tenantId: session.tenantId || null,
    user: session.user || null,
    isAuthenticated: true,
    expiresAt: session.expiresAt ?? null,
    refreshExpiresAt: session.refreshExpiresAt ?? null,
    status: 'authenticated' as SessionStatus,
  }
}

interface AuthState {
  token: string | null
  memberId: string | null
  tenantId: string | null
  user: User | null
  isAuthenticated: boolean
  expiresAt: number | null
  refreshExpiresAt: number | null
  status: SessionStatus
  isHydrated: boolean
  restorationError: string | null
  restoreSession: () => Promise<void>
  login: (session: Session) => Promise<void>
  logout: () => Promise<void>
  setTenantId: (tenantId: string) => void
  updateSession: (session: Partial<Session>) => void
  setStatus: (status: SessionStatus) => void
  needsRefresh: () => boolean
}

const initialState = {
  token: null,
  memberId: null,
  tenantId: null,
  user: null,
  isAuthenticated: false,
  expiresAt: null,
  refreshExpiresAt: null,
  status: 'anonymous' as SessionStatus,
}

export const useAuthStore = create<AuthState>((set, get) => ({
  ...initialState,
  isHydrated: false,
  restorationError: null,

  restoreSession: () => {
    if (restoration) return restoration
    const revision = ++sessionRevision
    set({ isHydrated: false, restorationError: null })
    const operation = (async () => {
      try {
        const token = await withTokenStorage(() => tokenStorage.getToken())
        if (revision !== sessionRevision) return
        if (!token) {
          set(initialState)
          return
        }
        const session = await authApi.authenticate(token)
        if (revision === sessionRevision) set(sessionState(session))
      } catch (error) {
        if (revision !== sessionRevision) return
        const status = (error as { response?: { status?: number } })?.response?.status
        if (status === 401 || status === 403) {
          set(initialState)
          try {
            await withTokenStorage(() => tokenStorage.clearToken())
          } catch {
            if (revision === sessionRevision) set({ restorationError: 'Could not clear the saved credential. Please sign in again.' })
          }
        } else {
          set({ ...initialState, restorationError: 'Could not restore your session. Check your connection and retry, or sign in again.' })
        }
      } finally {
        if (revision === sessionRevision) set({ isHydrated: true })
      }
    })()
    restoration = operation.finally(() => {
      if (revision === sessionRevision) restoration = null
    })
    return restoration
  },

  login: async (session) => {
    const revision = ++sessionRevision
    restoration = null
    await withTokenStorage(() => tokenStorage.setToken(session.token))
    if (revision === sessionRevision) {
      set({ ...sessionState(session), isHydrated: true, restorationError: null })
    }
  },

  logout: async () => {
    ++sessionRevision
    restoration = null
    set({ ...initialState, isHydrated: true, restorationError: null })
    await withTokenStorage(() => tokenStorage.clearToken())
  },

  setTenantId: (tenantId) => set({ tenantId }),

  updateSession: (session) => {
    set((state) => ({
      token: session.token ?? state.token,
      memberId: session.memberId ?? state.memberId,
      tenantId: session.tenantId ?? state.tenantId,
      user: session.user ?? state.user,
      expiresAt: session.expiresAt ?? state.expiresAt,
      refreshExpiresAt: session.refreshExpiresAt ?? state.refreshExpiresAt,
    }))
  },

  setStatus: (status) => set({ status }),

  needsRefresh: () => {
    const { expiresAt } = get()
    if (!expiresAt) {
      return false
    }
    return Date.now() >= expiresAt - REFRESH_THRESHOLD_MS
  },
}))

configureSessionAccess({
  getSnapshot: () => {
    const { token, tenantId, isAuthenticated, needsRefresh } = useAuthStore.getState()
    return { token, tenantId, isAuthenticated, needsRefresh }
  },
  updateSession: (session) => useAuthStore.getState().updateSession(session),
  logout: () => {
    void useAuthStore.getState().logout().catch(() => {
      useAuthStore.setState({ restorationError: 'Could not clear the saved credential. Please sign in again.' })
    })
  },
})
