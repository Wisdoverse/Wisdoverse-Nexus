import axios, { AxiosError, AxiosInstance, InternalAxiosRequestConfig } from 'axios'

import { useAuthStore } from '../../features/auth/authStore'
import { handleRefreshFailure, handleRefreshSuccess, isRecoverable401, sessionManager } from './sessionManager'

const API_BASE_URL = process.env.EXPO_PUBLIC_API_BASE_URL || 'http://localhost:8080/api/v1'

class HttpClient {
  private instance: AxiosInstance

  constructor() {
    this.instance = axios.create({
      baseURL: API_BASE_URL,
      timeout: 30000,
      headers: {
        'Content-Type': 'application/json',
      },
    })

    this.setupInterceptors()
  }

  private setupInterceptors(): void {
    this.instance.interceptors.request.use(
      async (config: InternalAxiosRequestConfig) => {
        const { token, tenantId, needsRefresh, isAuthenticated } = useAuthStore.getState()

        if (isAuthenticated && needsRefresh() && !config.url?.includes('/auth/')) {
          const refreshResult = await sessionManager.executeWithRefreshLock(async () => {
            const response = await axios.post<{ token: string; expiresAt: number; refreshExpiresAt?: number }>(
              `${API_BASE_URL}/auth/refresh`,
              {}
            )
            return response.data
          })

          handleRefreshSuccess(refreshResult)
          config.headers.Authorization = `Bearer ${refreshResult.token}`
        } else if (token) {
          config.headers.Authorization = `Bearer ${token}`
        }

        if (tenantId) {
          config.headers['X-Tenant-ID'] = tenantId
        }

        return config
      },
      (error) => Promise.reject(error)
    )

    this.instance.interceptors.response.use(
      (response) => response,
      async (error: AxiosError<{ code?: string }>) => {
        const originalRequest = error.config as InternalAxiosRequestConfig & { _retry?: boolean }

        if (error.response?.status === 401 && originalRequest && !originalRequest._retry) {
          if (isRecoverable401(error.response.data)) {
            originalRequest._retry = true

            try {
              const refreshResult = await sessionManager.executeWithRefreshLock(async () => {
                const response = await axios.post<{ token: string; expiresAt: number; refreshExpiresAt?: number }>(
                  `${API_BASE_URL}/auth/refresh`,
                  {}
                )
                return response.data
              })

              handleRefreshSuccess(refreshResult)
              originalRequest.headers.Authorization = `Bearer ${refreshResult.token}`
              return this.instance(originalRequest)
            } catch (refreshError) {
              handleRefreshFailure()
              return Promise.reject(refreshError)
            }
          }

          await useAuthStore.getState().logout()
        }

        return Promise.reject(error)
      }
    )
  }

  get<T>(url: string, params?: Record<string, unknown>) {
    return this.instance.get<T>(url, { params })
  }

  post<T>(url: string, data?: unknown) {
    return this.instance.post<T>(url, data)
  }

  put<T>(url: string, data?: unknown) {
    return this.instance.put<T>(url, data)
  }

  patch<T>(url: string, data?: unknown) {
    return this.instance.patch<T>(url, data)
  }

  delete<T>(url: string) {
    return this.instance.delete<T>(url)
  }
}

export const httpClient = new HttpClient()
