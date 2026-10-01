export interface SessionSnapshot {
  token: string | null
  tenantId: string | null
  isAuthenticated: boolean
  needsRefresh: () => boolean
}

export interface RefreshSessionResult {
  token: string
  expiresAt: number
  refreshExpiresAt?: number
}

interface SessionAccess {
  getSnapshot: () => SessionSnapshot
  updateSession: (session: RefreshSessionResult) => void
  logout: () => void
}

const anonymousSession: SessionSnapshot = {
  token: null,
  tenantId: null,
  isAuthenticated: false,
  needsRefresh: () => false,
}

let access: SessionAccess = {
  getSnapshot: () => anonymousSession,
  updateSession: () => {},
  logout: () => {},
}

export function configureSessionAccess(nextAccess: SessionAccess): void {
  access = nextAccess
}

export function getSessionSnapshot(): SessionSnapshot {
  return access.getSnapshot()
}

export function updateSession(session: RefreshSessionResult): void {
  access.updateSession(session)
}

export function logoutSession(): void {
  access.logout()
}
