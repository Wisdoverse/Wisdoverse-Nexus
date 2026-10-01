import { getSessionSnapshot, logoutSession } from '../session/sessionAccess'
import type { ConnectionState, WebSocketClientOptions, WebSocketMessage } from './types'

const WS_URL = import.meta.env.VITE_WS_URL || `${window.location.protocol === 'https:' ? 'wss:' : 'ws:'}//${window.location.host}/ws`
const MAX_RECONNECT_DELAY = 30000
const DEFAULT_HEARTBEAT_INTERVAL = 15000
const DEFAULT_HEARTBEAT_TIMEOUT = 7000

export class WebSocketClient {
  private ws: WebSocket | null = null
  private reconnectAttempts = 0
  private readonly maxReconnectAttempts: number
  private readonly reconnectDelay: number
  private readonly heartbeatInterval: number
  private readonly heartbeatTimeout: number
  private onMessage?: (data: WebSocketMessage) => void
  private onStateChange?: (state: ConnectionState) => void
  private intentionalClose = true
  private authenticated = false
  private readonly url: string
  private readonly getToken: () => string | null
  private currentState: ConnectionState = 'disconnected'
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null
  private heartbeatTimer: ReturnType<typeof setInterval> | null = null
  private heartbeatTimeoutTimer: ReturnType<typeof setTimeout> | null = null
  private lastRoomId?: string
  private offlineQueue: string[] = []
  private readonly onOnline = () => {
    if (!this.intentionalClose && this.currentState !== 'connected') {
      this.connect(this.lastRoomId)
    }
  }
  private readonly onOffline = () => {
    this.setState('disconnected')
  }

  constructor(options: WebSocketClientOptions = {}) {
    this.url = options.url ?? WS_URL
    this.getToken = options.getToken ?? (() => getSessionSnapshot().token)
    this.maxReconnectAttempts = options.maxReconnectAttempts ?? 5
    this.reconnectDelay = options.reconnectDelay ?? 1000
    this.heartbeatInterval = options.heartbeatInterval ?? DEFAULT_HEARTBEAT_INTERVAL
    this.heartbeatTimeout = options.heartbeatTimeout ?? DEFAULT_HEARTBEAT_TIMEOUT
    this.onMessage = options.onMessage
    this.onStateChange = options.onStateChange

    if (typeof window !== 'undefined') {
      window.addEventListener('online', this.onOnline)
      window.addEventListener('offline', this.onOffline)
    }
  }

  connect(roomId?: string): void {
    this.intentionalClose = false
    this.lastRoomId = roomId ?? this.lastRoomId
    this.clearReconnectTimer()
    this.clearHeartbeatTimers()
    this.setState(this.reconnectAttempts > 0 ? 'reconnecting' : 'connecting')

    this.authenticated = false
    const previous = this.ws
    this.ws = null
    previous?.close()
    try {
      const socket = new WebSocket(this.url)
      this.ws = socket
      socket.onopen = () => {
        if (this.ws !== socket) return
        socket.send(JSON.stringify({ type: 'auth', token: `Bearer ${this.getToken() ?? ''}` }))
      }
      socket.onmessage = (event) => {
        if (this.ws !== socket) return
        let data
        try { data = JSON.parse(event.data) } catch { return }
        if (data.type === 'auth_success') {
          if (this.lastRoomId) socket.send(JSON.stringify({ type: 'join_room', room_id: this.lastRoomId }))
          else this.markConnected()
        } else if (data.type === 'room_joined' && data.room_id === this.lastRoomId) {
          this.markConnected()
        } else if (data.type === 'auth_error' || data.type === 'auth_required') {
          this.disconnect()
          logoutSession()
        } else if (data.type === 'heartbeat_ack') {
          this.clearHeartbeatTimeout()
        } else if (data.type === 'new_message') {
          this.onMessage?.({ type: 'message', payload: {
            id: data.message_id, roomId: data.room_id, sender: data.sender_id,
            text: data.content, reply_to: data.reply_to, timestamp: new Date(data.timestamp).toISOString(),
          } })
        } else if (data.type === 'error' && !this.authenticated) {
          this.disconnect()
        }
      }
      socket.onclose = () => {
        if (this.ws !== socket) return
        this.authenticated = false
        this.clearHeartbeatTimers()
        this.setState('disconnected')
        if (!this.intentionalClose) this.scheduleReconnect()
      }
      socket.onerror = () => socket.close()
    } catch {
      this.setState('disconnected')
      this.scheduleReconnect()
    }
  }

  private markConnected(): void {
    this.authenticated = true
    this.reconnectAttempts = 0
    this.setState('connected')
    this.startHeartbeat()
    this.flushOfflineQueue()
  }

  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.warn('Max reconnect attempts reached')
      return
    }

    this.reconnectAttempts++
    this.setState('reconnecting')
    const delay = Math.min(this.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1), MAX_RECONNECT_DELAY)

    this.reconnectTimer = setTimeout(() => {
      if (!this.intentionalClose) {
        this.connect(this.lastRoomId)
      }
    }, delay)
  }

  disconnect(): void {
    this.intentionalClose = true
    this.authenticated = false
    this.offlineQueue = []
    this.reconnectAttempts = 0
    this.clearReconnectTimer()
    this.clearHeartbeatTimers()
    const socket = this.ws
    this.ws = null
    socket?.close()
    this.setState('disconnected')
  }

  send(data: unknown): boolean {
    if (this.authenticated && this.ws?.readyState === WebSocket.OPEN) {
      this.ws.send(JSON.stringify(data))
      return true
    }
    if (this.offlineQueue.length >= 1000) throw new Error('Offline queue is full')
    this.offlineQueue.push(JSON.stringify(data))
    return false
  }

  dispose(): void {
    this.disconnect()
    if (typeof window !== 'undefined') {
      window.removeEventListener('online', this.onOnline)
      window.removeEventListener('offline', this.onOffline)
    }
  }

  get state(): ConnectionState {
    return this.currentState
  }

  get queuedCount(): number {
    return this.offlineQueue.length
  }

  setHandlers(handlers: Pick<WebSocketClientOptions, 'onMessage' | 'onStateChange'>): void {
    if (handlers.onMessage) {
      this.onMessage = handlers.onMessage
    }
    if (handlers.onStateChange) {
      this.onStateChange = handlers.onStateChange
    }
  }

  private setState(next: ConnectionState): void {
    this.currentState = next
    this.onStateChange?.(next)
  }

  private flushOfflineQueue(): void {
    if (!this.authenticated || !this.ws || this.ws.readyState !== WebSocket.OPEN || this.offlineQueue.length === 0) {
      return
    }
    while (this.offlineQueue.length > 0) {
      const payload = this.offlineQueue.shift()
      if (payload) {
        this.ws.send(payload)
      }
    }
  }

  private startHeartbeat(): void {
    this.clearHeartbeatTimers()
    this.heartbeatTimer = setInterval(() => {
      if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
        return
      }
      this.ws.send(JSON.stringify({ type: 'heartbeat', timestamp: Date.now() }))
      this.heartbeatTimeoutTimer = setTimeout(() => {
        if (this.ws && this.ws.readyState === WebSocket.OPEN) {
          this.ws.close()
        }
      }, this.heartbeatTimeout)
    }, this.heartbeatInterval)
  }

  private clearHeartbeatTimeout(): void {
    if (this.heartbeatTimeoutTimer) {
      clearTimeout(this.heartbeatTimeoutTimer)
      this.heartbeatTimeoutTimer = null
    }
  }

  private clearHeartbeatTimers(): void {
    if (this.heartbeatTimer) {
      clearInterval(this.heartbeatTimer)
      this.heartbeatTimer = null
    }
    this.clearHeartbeatTimeout()
  }

  private clearReconnectTimer(): void {
    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }
}

export const wsClient = new WebSocketClient()
