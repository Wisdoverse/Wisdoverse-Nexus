import { httpClient } from '../httpClient'

export interface Room {
  id: string
  name: string
  topic?: string
  createdAt?: string
}

export const roomsApi = {
  list: async () => {
    const response = await httpClient.get<{ rooms: Room[]; total: number }>('/rooms')
    return { ...response, data: response.data.rooms }
  },
  create: (name: string, topic?: string) => httpClient.post<Room>('/rooms', { name, topic }),
  get: (id: string) => httpClient.get<Room>(`/rooms/${id}`),
}
