import * as SecureStore from 'expo-secure-store'

const ACCESS_TOKEN_KEY = 'nexis.access_token'

export const tokenStorage = {
  async getToken(): Promise<string | null> {
    return SecureStore.getItemAsync(ACCESS_TOKEN_KEY)
  },

  async setToken(token: string): Promise<void> {
    await SecureStore.setItemAsync(ACCESS_TOKEN_KEY, token)
  },

  async clearToken(): Promise<void> {
    await SecureStore.deleteItemAsync(ACCESS_TOKEN_KEY)
  },
}
