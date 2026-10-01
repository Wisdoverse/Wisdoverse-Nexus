import { expect, it } from 'vitest'
import { authApi } from '../src/shared/api/authApi'
import { roomsApi } from '../src/shared/api/endpoints/rooms'
import { messagesApi } from '../src/shared/api/endpoints/messages'
import { configureSessionAccess } from '../src/shared/session/sessionAccess'
import { WebSocketClient } from '../src/shared/ws/wsClient'

it('authenticates, creates/lists a room, and receives the committed HTTP message through WebSocket', async () => {
  const token = process.env.NEXIS_SMOKE_TOKEN!
  await expect(authApi.authenticate('invalid-token')).rejects.toThrow()
  await expect(authApi.authenticate(process.env.NEXIS_SMOKE_EXPIRED_TOKEN!)).rejects.toThrow()
  const session = await authApi.authenticate(token)
  expect(session.memberId).toBe('m1-owner')
  configureSessionAccess({ getSnapshot: () => ({ token, tenantId: null, isAuthenticated: true, needsRefresh: () => false }), updateSession: () => {}, logout: () => {} })
  const { data: room } = await roomsApi.create('web-smoke', 'synthetic')
  expect((await roomsApi.list()).data.some((item) => item.id === room.id)).toBe(true)
  let connected = false
  const events: any[] = []
  const socket = new WebSocketClient({ url: process.env.VITE_WS_URL!, getToken: () => token,
    onStateChange: (state) => { connected = state === 'connected' }, onMessage: (event) => events.push(event) })
  try {
    socket.connect(room.id)
    await expect.poll(() => connected).toBe(true)
    const { data: message } = await messagesApi.send(room.id, session.memberId, 'web message')
    expect(message.text).toBe('web message')
    await expect.poll(() => events.some((event) => event.payload.id === message.id)).toBe(true)
    expect((await messagesApi.list(room.id)).data.at(-1)?.id).toBe(message.id)
  } finally { socket.dispose() }
})
