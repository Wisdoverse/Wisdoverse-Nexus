import axios, { type AxiosError, type AxiosInstance } from 'axios'
import { getSessionSnapshot, logoutSession } from '../session/sessionAccess'

const API_BASE_URL = import.meta.env.VITE_API_BASE_URL || '/api/v1'

class HttpClient {
  private readonly instance: AxiosInstance
  constructor() {
    this.instance = axios.create({ baseURL: API_BASE_URL, timeout: 30000 })
    this.instance.interceptors.request.use((config) => {
      const { token } = getSessionSnapshot()
      if (token) config.headers.Authorization = `Bearer ${token}`
      return config
    })
    this.instance.interceptors.response.use((response) => response, (error: AxiosError) => {
      if (error.response?.status === 401) logoutSession()
      return Promise.reject(error)
    })
  }
  get<T>(url: string, params?: Record<string, unknown>) { return this.instance.get<T>(url, { params }) }
  post<T>(url: string, data?: unknown) { return this.instance.post<T>(url, data) }
  put<T>(url: string, data?: unknown) { return this.instance.put<T>(url, data) }
  patch<T>(url: string, data?: unknown) { return this.instance.patch<T>(url, data) }
  delete<T>(url: string) { return this.instance.delete<T>(url) }
}
export const httpClient = new HttpClient()
