import { useAuthStore } from '../../features/auth/authStore'

interface RefreshResult {
  token: string
  expiresAt: number
  refreshExpiresAt?: number
}

type RefreshFn = () => Promise<RefreshResult>

let refreshPromise: Promise<RefreshResult> | null = null

export const sessionManager = {
  async executeWithRefreshLock(refreshFn: RefreshFn): Promise<RefreshResult> {
    if (refreshPromise) {
      return refreshPromise
    }

    refreshPromise = refreshFn()

    try {
      const result = await refreshPromise
      return result
    } finally {
      refreshPromise = null
    }
  },

  clearRefreshLock(): void {
    refreshPromise = null
  },

  isRefreshing(): boolean {
    return refreshPromise !== null
  },
}

export function isRecoverable401(errorData: unknown): boolean {
  if (!errorData || typeof errorData !== 'object') {
    return false
  }

  const data = errorData as Record<string, unknown>
  const code = data?.code

  if (typeof code !== 'string') {
    return false
  }

  return code.toLowerCase() === 'token_expired'
}

export function handleRefreshSuccess(result: RefreshResult): void {
  const { updateSession } = useAuthStore.getState()
  updateSession({
    token: result.token,
    expiresAt: result.expiresAt,
    refreshExpiresAt: result.refreshExpiresAt,
  })
}

export function handleRefreshFailure(): void {
  const { logout } = useAuthStore.getState()
  logout()
  sessionManager.clearRefreshLock()
}
