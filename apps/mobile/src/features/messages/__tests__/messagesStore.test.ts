import { beforeEach, describe, expect, it, vi } from 'vitest'
import { AxiosHeaders, type AxiosResponse } from 'axios'

vi.mock('zustand', () => ({
  create: (creator: (set: (partial: unknown) => void, get: () => unknown) => unknown) => {
    let state: Record<string, unknown> = {}
    const setState = (partial: unknown) => {
      if (typeof partial === 'function') {
        const updater = partial as (s: Record<string, unknown>) => Record<string, unknown>
        state = { ...state, ...updater(state) }
      } else {
        state = { ...state, ...(partial as Record<string, unknown>) }
      }
    }
    const getState = () => state
    state = creator(setState, getState) as Record<string, unknown>
    return { getState }
  },
}))

vi.mock('../../auth/authStore', () => ({
  useAuthStore: {
    getState: () => ({ memberId: 'member-42' }),
  },
}))

vi.mock('../../../shared/api/endpoints/messages', () => ({
  messagesApi: {
    list: vi.fn(),
    send: vi.fn(),
  },
}))

import { useMessagesStore } from '../messagesStore'
import { messagesApi } from '../../../shared/api/endpoints/messages'

function axiosResponse<T>(data: T): AxiosResponse<T> {
  return {
    data,
    status: 200,
    statusText: 'OK',
    headers: {},
    config: { headers: new AxiosHeaders() },
  }
}

describe('messagesStore', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    useMessagesStore.getState().messages = []
    useMessagesStore.getState().error = null
  })

  it('fetches messages into store', async () => {
    vi.mocked(messagesApi.list).mockResolvedValueOnce(
      axiosResponse([
        {
          id: 'msg-1',
          roomId: 'room-1',
          sender: 'member-42',
          text: 'hello',
          timestamp: new Date().toISOString(),
        },
      ]),
    )

    await useMessagesStore.getState().fetchMessages('room-1')

    expect(messagesApi.list).toHaveBeenCalledWith('room-1')
    expect(useMessagesStore.getState().messages).toHaveLength(1)
    expect(useMessagesStore.getState().messages[0].text).toBe('hello')
  })

  it('appends sent message on success', async () => {
    vi.mocked(messagesApi.send).mockResolvedValueOnce(
      axiosResponse({
        id: 'msg-2',
        roomId: 'room-1',
        sender: 'member-42',
        text: 'new message',
        timestamp: new Date().toISOString(),
      }),
    )

    await useMessagesStore.getState().sendMessage('room-1', 'new message')

    expect(messagesApi.send).toHaveBeenCalledWith('room-1', 'member-42', 'new message')
    expect(useMessagesStore.getState().messages).toHaveLength(1)
    expect(useMessagesStore.getState().messages[0].text).toBe('new message')
  })
})
