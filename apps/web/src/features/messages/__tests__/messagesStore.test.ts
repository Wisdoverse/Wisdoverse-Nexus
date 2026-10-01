import { beforeEach, describe, expect, it, vi } from 'vitest'
import { useMessagesStore } from '../messagesStore'
import { resetAuthStore, useAuthStore } from '../../../entities/session'
import { httpClient } from '../../../shared/api/httpClient'

vi.mock('../../../shared/api/httpClient', () => ({
  httpClient: {
    get: vi.fn(),
    post: vi.fn(),
  },
}))

describe('messagesStore', () => {
  beforeEach(() => {
    resetAuthStore()
    useAuthStore.getState().login({
      token: 'token',
      memberId: 'nexis:human:tester',
      tenantId: 'tenant-1',
    })

    useMessagesStore.getState().disconnect()
    useMessagesStore.setState({
      messages: [],
      loading: false,
      error: null,
      unreadCount: 0,
      connectionState: 'disconnected',
      offlineQueue: [],
    })

    vi.mocked(httpClient.post).mockReset()
    vi.mocked(httpClient.get).mockReset()
  })

  it('queues outgoing messages while offline and flushes them when recovered', async () => {
    vi.mocked(httpClient.post).mockResolvedValueOnce({
      data: {
        id: 'server-1',
        roomId: 'room-1',
        sender: 'nexis:human:tester',
        text: 'hello',
        timestamp: new Date().toISOString(),
      },
      status: 200,
      statusText: 'OK',
      headers: {},
      config: {},
    } as any)

    await useMessagesStore.getState().sendMessage('room-1', 'hello')

    const queued = useMessagesStore.getState().messages[0]
    expect(queued.deliveryStatus).toBe('sending')

    await useMessagesStore.getState().flushOfflineQueue()

    const delivered = useMessagesStore.getState().messages[0]
    expect(delivered.deliveryStatus).toBe('delivered')
    expect(delivered.id).toBe('server-1')
  })

  it('marks message as failed when sending fails online', async () => {
    useMessagesStore.getState().setConnectionState('connected')
    vi.mocked(httpClient.post).mockRejectedValueOnce(new Error('network'))

    await useMessagesStore.getState().sendMessage('room-1', 'oops')

    const failed = useMessagesStore.getState().messages[0]
    expect(failed.deliveryStatus).toBe('failed')
    expect(useMessagesStore.getState().offlineQueue[0].clientId).toBe(failed.id)
  })

  it('flushes each key once and keeps the confirmed message when its websocket echo arrives first', async () => {
    let acknowledge!: (response: any) => void
    vi.mocked(httpClient.post).mockImplementationOnce(() => new Promise((resolve) => { acknowledge = resolve }))
    await useMessagesStore.getState().sendMessage('room-1', 'hello')
    const firstFlush = useMessagesStore.getState().flushOfflineQueue()
    await useMessagesStore.getState().flushOfflineQueue()
    const saved = { id: 'saved', roomId: 'room-1', sender: 'nexis:human:tester', text: 'hello' }
    useMessagesStore.getState().handleRealtimeEvent({ type: 'message', payload: saved })
    acknowledge({ data: saved })
    await firstFlush
    expect(httpClient.post).toHaveBeenCalledTimes(1)
    expect(useMessagesStore.getState().messages).toEqual([{ ...saved, deliveryStatus: 'delivered' }])
    expect(useMessagesStore.getState().offlineQueue).toHaveLength(0)
  })

  it('retains an ambiguous failed write and retries using the same idempotency key', async () => {
    await useMessagesStore.getState().sendMessage('room-1', 'retry')
    const key = useMessagesStore.getState().offlineQueue[0].clientId
    vi.mocked(httpClient.post).mockRejectedValueOnce(new Error('response lost'))
    await useMessagesStore.getState().flushOfflineQueue()
    expect(useMessagesStore.getState().offlineQueue[0].clientId).toBe(key)
    vi.mocked(httpClient.post).mockResolvedValueOnce({ data: { id: 'saved', roomId: 'room-1', sender: 'tester', text: 'retry' } } as any)
    await useMessagesStore.getState().flushOfflineQueue()
    expect(vi.mocked(httpClient.post).mock.calls.map((call) => call[1])).toEqual([
      { roomId: 'room-1', text: 'retry', clientMessageId: key },
      { roomId: 'room-1', text: 'retry', clientMessageId: key },
    ])
    expect(useMessagesStore.getState().messages[0].id).toBe('saved')
  })

  it('tracks unread count and marks messages as read', () => {
    useMessagesStore.getState().handleRealtimeEvent({
      type: 'message',
      payload: {
        id: 'msg-1',
        roomId: 'room-1',
        sender: 'nexis:human:peer',
        text: 'ping',
        timestamp: new Date().toISOString(),
      },
    })

    expect(useMessagesStore.getState().unreadCount).toBe(1)

    useMessagesStore.getState().markAllRead()
    expect(useMessagesStore.getState().unreadCount).toBe(0)
    expect(useMessagesStore.getState().messages[0].deliveryStatus).toBe('read')
  })
})
