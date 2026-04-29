import axios from 'axios'

import type { Session } from '../../app/types'

const API_BASE_URL = process.env.EXPO_PUBLIC_API_BASE_URL || 'http://localhost:8080/api/v1'

interface RefreshResponse {
  token: string
  expiresAt: number
  refreshExpiresAt?: number
}

export const authApi = {
  async refreshToken(): Promise<RefreshResponse> {
    const response = await axios.post<RefreshResponse>(`${API_BASE_URL}/auth/refresh`, {})
    return response.data
  },

  async login(credentials: { email: string; password: string }): Promise<Session> {
    const response = await axios.post<Session>(`${API_BASE_URL}/auth/login`, credentials)
    return response.data
  },

  async logout(): Promise<void> {
    await axios.post(`${API_BASE_URL}/auth/logout`, {})
  },
}
