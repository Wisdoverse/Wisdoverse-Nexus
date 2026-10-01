import axios from 'axios'
import type { Session } from '../session/types'
const API_BASE_URL = process.env.EXPO_PUBLIC_API_BASE_URL || 'http://localhost:8080/v1'
export const authApi = {
  async authenticate(token: string): Promise<Session> {
    const { data } = await axios.get<Omit<Session, 'token'>>(`${API_BASE_URL}/auth/session`, {
      headers: { Authorization: `Bearer ${token}` }, timeout: 30000,
    })
    return { ...data, token }
  },
}
