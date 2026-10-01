import axios from 'axios'
import type { Session } from './endpoints/auth'
const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'
export const authApi = {
  async authenticate(token: string): Promise<Session> {
    const { data } = await axios.get<Omit<Session, 'token'>>(`${API_BASE_URL}/auth/session`, {
      headers: { Authorization: `Bearer ${token}` }, timeout: 30000,
    })
    return { ...data, token }
  },
}
