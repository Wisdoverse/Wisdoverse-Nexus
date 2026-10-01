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
  get: (id: string) => httpClient.get<Room>(`/rooms/${id}`),
  create: (name: string, topic?: string) => httpClient.post<Room>('/rooms', { name, topic }),
  delete: (id: string) => httpClient.delete(`/rooms/${id}`),
  invite: (roomId: string, memberId: string) =>
    httpClient.post(`/rooms/${roomId}/invite`, { memberId }),
}
