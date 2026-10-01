import { create } from 'zustand'

import { messagesApi, type Message } from '../../shared/api/endpoints/messages'
import { wsClient } from '../../shared/ws/wsClient'
import type { ConnectionState, WebSocketMessage } from '../../shared/ws/types'
import { useAuthStore } from '../auth/authStore'

interface MessagesState {
  activeRoomId: string | null
  connectionState: ConnectionState
  connect: (roomId: string) => void
  disconnect: () => void
  handleRealtimeEvent: (event: WebSocketMessage) => void
  messages: Message[]
  loading: boolean
  sending: boolean
  error: string | null
  fetchMessages: (roomId: string) => Promise<void>
  sendMessage: (roomId: string, text: string) => Promise<void>
}

export const useMessagesStore = create<MessagesState>((set, get) => ({
  activeRoomId: null,
  connectionState: 'disconnected',
  connect: (roomId) => { set({ activeRoomId: roomId, messages: [] }); wsClient.connect(roomId) },
  disconnect: () => { wsClient.disconnect(); set({ activeRoomId: null, connectionState: 'disconnected' }) },
  handleRealtimeEvent: (event) => {
    if (event.type !== 'message') return
    const message = event.payload as Message
    if (message.roomId !== get().activeRoomId) return
    set((state) => ({ messages: state.messages.some((item) => item.id === message.id)
      ? state.messages : [...state.messages, message] }))
  },
  messages: [],
  loading: false,
  sending: false,
  error: null,

  fetchMessages: async (roomId) => {
    set({ loading: true, error: null })
    try {
      const response = await messagesApi.list(roomId)
      if (get().activeRoomId && get().activeRoomId !== roomId) return
      set((state) => ({ messages: [...response.data,
        ...state.messages.filter((message) => !response.data.some((saved) => saved.id === message.id))], loading: false }))
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
        messages: state.messages.some((message) => message.id === response.data.id)
          ? state.messages : [...state.messages, response.data],
      }))
    } catch {
      set({ error: 'Failed to send message', sending: false })
    }
  },
}))

wsClient.setHandlers({
  onMessage: (event) => useMessagesStore.getState().handleRealtimeEvent(event),
  onStateChange: (connectionState) => {
    useMessagesStore.setState({ connectionState })
    const roomId = useMessagesStore.getState().activeRoomId
    if (connectionState === 'connected' && roomId) void useMessagesStore.getState().fetchMessages(roomId)
  },
})
