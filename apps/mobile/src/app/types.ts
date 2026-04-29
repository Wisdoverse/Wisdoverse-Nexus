export interface User {
  id: string
  email: string
  name?: string
}

export type SessionStatus = 'anonymous' | 'authenticated' | 'refreshing' | 'expired'

export interface Session {
  token: string
  memberId: string
  tenantId?: string
  user?: User
  expiresAt?: number
  refreshExpiresAt?: number
  roles?: string[]
  permissions?: string[]
}
