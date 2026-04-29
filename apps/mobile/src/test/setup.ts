import { vi } from 'vitest'

const secureStore = {
  getItemAsync: vi.fn(),
  setItemAsync: vi.fn(),
  deleteItemAsync: vi.fn(),
}

vi.mock('expo-secure-store', () => secureStore)
