import axios from 'axios'
import type { Session } from '../../app/types'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

interface RefreshResponse {
  token: string
  expiresAt: number
  refreshExpiresAt?: number
}

export const authApi = {
  async refreshToken(): Promise<RefreshResponse> {
    const response = await axios.post<RefreshResponse>(`${API_BASE_URL}/auth/refresh`, {}, { withCredentials: true })
    return response.data
  },

  async login(credentials: { email: string; password: string }): Promise<Session> {
    const response = await axios.post<Session>(`${API_BASE_URL}/auth/login`, credentials)
    return response.data
  },

  async logout(): Promise<void> {
    await axios.post(`${API_BASE_URL}/auth/logout`, {}, { withCredentials: true })
  },
}
