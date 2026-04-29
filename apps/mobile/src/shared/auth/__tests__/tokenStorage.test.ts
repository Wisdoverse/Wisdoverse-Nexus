import { describe, expect, it, vi, beforeEach } from 'vitest'
import * as SecureStore from 'expo-secure-store'
import { tokenStorage } from '../tokenStorage'

describe('tokenStorage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('stores and reads token', async () => {
    vi.mocked(SecureStore.setItemAsync).mockResolvedValueOnce()
    vi.mocked(SecureStore.getItemAsync).mockResolvedValueOnce('abc')

    await tokenStorage.setToken('abc')
    const token = await tokenStorage.getToken()

    expect(SecureStore.setItemAsync).toHaveBeenCalledTimes(1)
    expect(token).toBe('abc')
  })

  it('clears token', async () => {
    vi.mocked(SecureStore.deleteItemAsync).mockResolvedValueOnce()

    await tokenStorage.clearToken()

    expect(SecureStore.deleteItemAsync).toHaveBeenCalledTimes(1)
  })
})
