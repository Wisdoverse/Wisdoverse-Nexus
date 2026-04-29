import { create } from 'zustand'

import type { Session, SessionStatus, User } from '../../app/types'
import { tokenStorage } from '../../shared/auth/tokenStorage'

const REFRESH_THRESHOLD_MS = 60 * 1000

interface AuthState {
  token: string | null
  memberId: string | null
  tenantId: string | null
  user: User | null
  isAuthenticated: boolean
  expiresAt: number | null
  refreshExpiresAt: number | null
  status: SessionStatus
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

  login: async (session) => {
    set({
      token: session.token,
      memberId: session.memberId,
      tenantId: session.tenantId || null,
      user: session.user || null,
      isAuthenticated: true,
      expiresAt: session.expiresAt ?? null,
      refreshExpiresAt: session.refreshExpiresAt ?? null,
      status: 'authenticated',
    })
    await tokenStorage.setToken(session.token)
  },

  logout: async () => {
    set(initialState)
    await tokenStorage.clearToken()
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
