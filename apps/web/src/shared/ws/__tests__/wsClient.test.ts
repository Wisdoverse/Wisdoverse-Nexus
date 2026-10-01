import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { WebSocketClient } from '../wsClient'
import type { ConnectionState } from '../types'

class MockWebSocket {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  static instances: MockWebSocket[] = []

  readonly url: string
  readyState = MockWebSocket.CONNECTING
  sent: string[] = []

  onopen: (() => void) | null = null
  onmessage: ((event: { data: string }) => void) | null = null
  onclose: (() => void) | null = null
  onerror: (() => void) | null = null

  constructor(url: string) {
    this.url = url
    MockWebSocket.instances.push(this)
  }

  send(data: string): void {
    this.sent.push(data)
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.()
  }

  triggerOpen(): void {
    this.readyState = MockWebSocket.OPEN
    this.onopen?.()
  }

  triggerMessage(data: string): void {
    this.onmessage?.({ data })
  }

  triggerClose(): void {
    this.readyState = MockWebSocket.CLOSED
    this.onclose?.()
  }
}

describe('WebSocketClient', () => {
  const originalWebSocket = globalThis.WebSocket

  beforeEach(() => {
    vi.useFakeTimers()
    MockWebSocket.instances = []
    Object.defineProperty(globalThis, 'WebSocket', {
      value: MockWebSocket,
      configurable: true,
      writable: true,
    })
  })

  afterEach(() => {
    vi.useRealTimers()
    Object.defineProperty(globalThis, 'WebSocket', {
      value: originalWebSocket,
      configurable: true,
      writable: true,
    })
  })

  it('tracks connection state and reconnects with exponential backoff', () => {
    const states: ConnectionState[] = []
    const client = new WebSocketClient({
      onStateChange: (state) => states.push(state),
      reconnectDelay: 100,
      maxReconnectAttempts: 2,
    })

    client.connect('room-1')
    expect(states).toEqual(['connecting'])

    const firstSocket = MockWebSocket.instances[0]
    firstSocket.triggerOpen()
    expect(states.at(-1)).toBe('connecting')
    firstSocket.triggerMessage(JSON.stringify({ type: 'auth_success' }))
    firstSocket.triggerMessage(JSON.stringify({ type: 'room_joined', room_id: 'room-1' }))
    expect(states.at(-1)).toBe('connected')

    firstSocket.triggerClose()
    expect(states.at(-2)).toBe('disconnected')
    expect(states.at(-1)).toBe('reconnecting')

    vi.advanceTimersByTime(100)
    expect(MockWebSocket.instances).toHaveLength(2)
  })

  it('sends heartbeats and reconnects when heartbeat times out', () => {
    const states: ConnectionState[] = []
    const client = new WebSocketClient({
      onStateChange: (state) => states.push(state),
      heartbeatInterval: 1000,
      heartbeatTimeout: 500,
      reconnectDelay: 50,
    })

    client.connect()
    const socket = MockWebSocket.instances[0]
    socket.triggerOpen()
    socket.triggerMessage(JSON.stringify({ type: 'auth_success' }))

    vi.advanceTimersByTime(1000)
    expect(socket.sent).toHaveLength(2)

    vi.advanceTimersByTime(500)
    expect(states).toContain('reconnecting')
  })

  it('keeps tokens out of URLs and stops reconnecting after auth rejection', () => {
    const client = new WebSocketClient({ getToken: () => 'sensitive-token' })
    client.connect('room-1')
    const socket = MockWebSocket.instances[0]
    expect(socket.url).not.toContain('sensitive-token')
    socket.triggerOpen()
    expect(JSON.parse(socket.sent[0])).toEqual({ type: 'auth', token: 'Bearer sensitive-token' })
    socket.triggerMessage(JSON.stringify({ type: 'auth_error', code: 'TOKEN_EXPIRED' }))
    vi.advanceTimersByTime(60000)
    expect(MockWebSocket.instances).toHaveLength(1)
    expect(client.state).toBe('disconnected')
    client.dispose()
  })

  it('queues outbound messages while disconnected and flushes on connect', () => {
    const client = new WebSocketClient()
    client.connect('room-2')

    const sentNow = client.send({ type: 'message', payload: { text: 'hello' } })
    expect(sentNow).toBe(false)
    expect(client.queuedCount).toBe(1)

    const socket = MockWebSocket.instances[0]
    socket.triggerOpen()
    expect(client.queuedCount).toBe(1)
    socket.triggerMessage(JSON.stringify({ type: 'auth_success' }))
    socket.triggerMessage(JSON.stringify({ type: 'room_joined', room_id: 'room-2' }))

    expect(client.queuedCount).toBe(0)
    expect(socket.sent).toHaveLength(3)
  })
})
