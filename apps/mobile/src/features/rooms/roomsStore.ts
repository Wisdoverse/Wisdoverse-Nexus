import { create } from 'zustand'

import { roomsApi, type Room } from '../../shared/api/endpoints/rooms'

interface RoomsState {
  rooms: Room[]
  loading: boolean
  error: string | null
  fetchRooms: () => Promise<void>
}

export const useRoomsStore = create<RoomsState>((set) => ({
  rooms: [],
  loading: false,
  error: null,

  fetchRooms: async () => {
    set({ loading: true, error: null })
    try {
      const response = await roomsApi.list()
      set({ rooms: response.data, loading: false })
    } catch {
      set({ error: 'Failed to fetch rooms', loading: false })
    }
  },
}))
