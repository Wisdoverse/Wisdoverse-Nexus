import { create } from 'zustand'
import { useAuthStore } from '../../entities/session'
import type { Message } from '../../shared/api/endpoints/messages'
import { httpClient } from '../../shared/api/httpClient'
import { wsClient } from '../../shared/ws/wsClient'
import type {
  ConnectionState,
  MessageDeliveryStatus,
  MessageStatusPayload,
  ReadReceiptPayload,
  WebSocketMessage,
} from '../../shared/ws/types'

export interface RealtimeMessage extends Message {
  deliveryStatus: MessageDeliveryStatus
  clientId?: string
}

interface QueuedMessage {
  clientId: string
  roomId: string
  text: string
}

interface MessagesState {
  messages: RealtimeMessage[]
  loading: boolean
  error: string | null
  unreadCount: number
  connectionState: ConnectionState
  activeRoomId: string | null
  offlineQueue: QueuedMessage[]
  setMessages: (messages: Message[]) => void
  addMessage: (message: Message) => void
  fetchMessages: (roomId: string) => Promise<void>
  sendMessage: (roomId: string, text: string) => Promise<void>
  connect: (roomId: string) => void
  disconnect: () => void
  setConnectionState: (state: ConnectionState) => void
  flushOfflineQueue: () => Promise<void>
  handleRealtimeEvent: (event: WebSocketMessage) => void
  markAllRead: () => void
}

function toRealtimeMessage(message: Message, deliveryStatus: MessageDeliveryStatus = 'delivered'): RealtimeMessage {
  return { ...message, deliveryStatus }
}

function createLocalMessage(roomId: string, sender: string, text: string): RealtimeMessage {
  const clientId = `local-${Date.now()}-${Math.random().toString(36).slice(2, 8)}`
  return {
    id: clientId,
    clientId,
    roomId,
    sender,
    text,
    timestamp: new Date().toISOString(),
    deliveryStatus: 'sending',
  }
}

const flushingKeys = new Set<string>()

function confirmMessage(messages: RealtimeMessage[], localId: string, saved: Message): RealtimeMessage[] {
  const confirmed = toRealtimeMessage(saved)
  if (!messages.some((message) => message.id === localId)) {
    return messages.map((message) => message.id === saved.id ? confirmed : message)
  }
  return messages.flatMap((message) => message.id === localId ? [confirmed]
    : message.id === saved.id ? [] : [message])
}

export const useMessagesStore = create<MessagesState>((set, get) => ({
  messages: [],
  loading: false,
  error: null,
  unreadCount: 0,
  connectionState: 'disconnected',
  activeRoomId: null,
  offlineQueue: [],

  setMessages: (messages) => set({ messages: messages.map((message) => toRealtimeMessage(message, 'read')) }),
  addMessage: (message) =>
    set((state) => ({
      messages: [...state.messages, toRealtimeMessage(message)],
    })),
  setConnectionState: (state) => set({ connectionState: state }),

  fetchMessages: async (roomId: string) => {
    set({ loading: true, error: null })
    try {
      const response = await httpClient.get<Message[]>(`/rooms/${roomId}/messages`)
      if (get().activeRoomId && get().activeRoomId !== roomId) return
      set((state) => ({ messages: [
        ...response.data.map((message) => toRealtimeMessage(message, 'read')),
        ...state.messages.filter((message) => !response.data.some((saved) => saved.id === message.id)),
      ], loading: false, unreadCount: 0 }))
    } catch {
      set({ error: 'Failed to fetch messages', loading: false })
    }
  },

  sendMessage: async (roomId: string, text: string) => {
    const sender = useAuthStore.getState().memberId || 'nexis:human:unknown'
    const localMessage = createLocalMessage(roomId, sender, text)

    set((state) => ({ messages: [...state.messages, localMessage], error: null }))

    if (get().connectionState !== 'connected') {
      set((state) => ({
        messages: state.messages.map((message) =>
          message.id === localMessage.id ? { ...message, deliveryStatus: 'sending' } : message
        ),
        offlineQueue: [...state.offlineQueue, { clientId: localMessage.id, roomId, text }],
      }))
      return
    }

    try {
      const response = await httpClient.post<Message>('/messages', { roomId, text, clientMessageId: localMessage.id })
      set((state) => ({
        messages: confirmMessage(state.messages, localMessage.id, response.data),
      }))
    } catch {
      set((state) => ({
        messages: state.messages.map((message) =>
          message.id === localMessage.id ? { ...message, deliveryStatus: 'failed' } : message
        ),
        error: 'Failed to send message',
        offlineQueue: [...state.offlineQueue, { clientId: localMessage.id, roomId, text }],
      }))
    }
  },

  flushOfflineQueue: async () => {
    const queueSnapshot = [...get().offlineQueue]

    for (const queued of queueSnapshot) {
      if (flushingKeys.has(queued.clientId)) continue
      flushingKeys.add(queued.clientId)
      try {
        const response = await httpClient.post<Message>('/messages', {
          roomId: queued.roomId,
          clientMessageId: queued.clientId,
          text: queued.text,
        })
        set((state) => ({
          messages: confirmMessage(state.messages, queued.clientId, response.data),
          offlineQueue: state.offlineQueue.filter((item) => item.clientId !== queued.clientId),
          error: null,
        }))
      } catch {
        set((state) => ({
          messages: state.messages.map((message) =>
            message.id === queued.clientId ? { ...message, deliveryStatus: 'failed' } : message
          ),
          error: 'Failed to resend queued messages',
        }))
      } finally {
        flushingKeys.delete(queued.clientId)
      }
    }
  },

  handleRealtimeEvent: (event) => {
    if (event.type === 'message') {
      const incoming = event.payload as Message
      if (get().activeRoomId && incoming.roomId !== get().activeRoomId) return
      const currentMemberId = useAuthStore.getState().memberId
      set((state) => {
        const exists = state.messages.some((message) => message.id === incoming.id)
        if (exists) {
          return {}
        }
        const shouldCountUnread = incoming.sender !== currentMemberId
        return {
          messages: [...state.messages, toRealtimeMessage(incoming, shouldCountUnread ? 'delivered' : 'read')],
          unreadCount: shouldCountUnread ? state.unreadCount + 1 : state.unreadCount,
        }
      })
      return
    }

    if (event.type === 'message_status') {
      const payload = event.payload as MessageStatusPayload
      set((state) => ({
        messages: state.messages.map((message) =>
          message.id === payload.messageId ? { ...message, deliveryStatus: payload.status } : message
        ),
      }))
      return
    }

    if (event.type === 'read_receipt') {
      const payload = event.payload as ReadReceiptPayload
      set((state) => ({
        messages: state.messages.map((message) =>
          message.id === payload.messageId ? { ...message, deliveryStatus: 'read' } : message
        ),
      }))
    }
  },

  markAllRead: () => {
    const currentMemberId = useAuthStore.getState().memberId
    set((state) => ({
      messages: state.messages.map((message) =>
        message.sender === currentMemberId ? message : { ...message, deliveryStatus: 'read' }
      ),
      unreadCount: 0,
    }))
  },

  connect: (roomId: string) => {
    set({ activeRoomId: roomId, connectionState: 'connecting', messages: [], offlineQueue: [] })
    wsClient.connect(roomId)
  },

  disconnect: () => {
    wsClient.disconnect()
    set({ connectionState: 'disconnected', activeRoomId: null })
  },
}))

wsClient.setHandlers({
  onMessage: (event) => {
    useMessagesStore.getState().handleRealtimeEvent(event)
  },
  onStateChange: (state) => {
    const previousState = useMessagesStore.getState().connectionState
    useMessagesStore.getState().setConnectionState(state)
    if (state === 'connected' && previousState !== 'connected') {
      const roomId = useMessagesStore.getState().activeRoomId
      if (roomId) void useMessagesStore.getState().fetchMessages(roomId)
      void useMessagesStore.getState().flushOfflineQueue()
    }
  },
})

export type { Message, ConnectionState, MessageDeliveryStatus }
