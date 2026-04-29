import { httpClient } from '../httpClient'

export interface Room {
  id: string
  name: string
  topic?: string
  createdAt?: string
}

export const roomsApi = {
  list: () => httpClient.get<Room[]>('/rooms'),
  get: (id: string) => httpClient.get<Room>(`/rooms/${id}`),
}
