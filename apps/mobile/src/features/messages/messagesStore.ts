import { create } from 'zustand'

import { messagesApi, type Message } from '../../shared/api/endpoints/messages'
import { useAuthStore } from '../auth/authStore'

interface MessagesState {
  messages: Message[]
  loading: boolean
  sending: boolean
  error: string | null
  fetchMessages: (roomId: string) => Promise<void>
  sendMessage: (roomId: string, text: string) => Promise<void>
}

export const useMessagesStore = create<MessagesState>((set) => ({
  messages: [],
  loading: false,
  sending: false,
  error: null,

  fetchMessages: async (roomId) => {
    set({ loading: true, error: null })
    try {
      const response = await messagesApi.list(roomId)
      set({ messages: response.data, loading: false })
    } catch {
      set({ error: 'Failed to fetch messages', loading: false })
    }
  },

  sendMessage: async (roomId, text) => {
    const sender = useAuthStore.getState().memberId || 'nexis:human:unknown'
    set({ sending: true, error: null })

    try {
      const response = await messagesApi.send(roomId, sender, text)
      set((state) => ({
        sending: false,
        messages: [...state.messages, response.data],
      }))
    } catch {
      set({ error: 'Failed to send message', sending: false })
    }
  },
}))
