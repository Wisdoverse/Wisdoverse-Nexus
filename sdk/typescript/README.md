# TypeScript SDK

The `@wisdoverse/nexus-sdk` package provides a Node.js client for the Nexus
HTTP and WebSocket APIs. The documented workflow is validated with Node.js 24.x.

## Install and build

From the repository root:

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-sdk build
```

## Connect to a gateway

The gateway validates a JWT issued by your application. It does not provide
password login, registration, token issuance, or token refresh. Supply the
externally issued token through your application's secret/configuration system;
do not put a production token in source code.

```ts
import { NexisClient } from '@wisdoverse/nexus-sdk';

const token = process.env.NEXUS_TOKEN;
if (!token) throw new Error('Set NEXUS_TOKEN to an externally issued JWT');

const client = new NexisClient({ baseUrl: 'http://localhost:8080' });

async function main(token: string) {
  const session = await client.authenticate(token);
  console.log(`Authenticated ${session.memberId} (${session.memberType})`);

  const room = await client.createRoom({ name: 'engineering', topic: 'Build notes' });
  const rooms = await client.listRooms({ limit: 50, offset: 0 });
  console.log(`Created ${room.id}; accessible rooms: ${rooms.length}`);

  const message = await client.sendMessage(room.id, 'First update');
  console.log(`Sent ${message.id}`);

  // Use one stable key for retries of this logical write. Reuse it only with
  // the same room, text, and reply target. The default key is newly generated.
  const retryKey = 'engineering-update-2026-10-01';
  const retried = await client.sendMessage(room.id, 'Retryable update', undefined, retryKey);
  console.log(`Accepted ${retried.id}`);

  const connection = client.connect(room.id);
  connection.onConnectionChange((state, attempt) => {
    console.log('WebSocket connection:', state, attempt ?? '');
    if (state === 'connected') {
      // Reconnect restores authentication and the room subscription, but does
      // not replay missed messages. Fetch history and merge by message ID.
      void client.getMessages(room.id).then((history) => {
        console.log(`Recovered ${history.length} messages`);
      }).catch((error: unknown) => {
        console.error(error instanceof Error ? error.message : 'History request failed');
      });
    }
  });
  client.on('new_message', (event) => {
    if (event.type === 'new_message' && event.room_id === room.id) console.log('Live message ID:', event.message_id);
  });

  // At application shutdown:
  // client.logout();
}

void main(token).catch((error: unknown) => {
  console.error('Nexus request failed:', error instanceof Error ? error.message : 'Unknown error');
  client.logout();
  process.exitCode = 1;
});
```

The client also provides `getRoom(roomId)`, `getMessages(roomId)`,
`deleteRoom(roomId)`, `inviteMember(roomId, memberId)`, `disconnect()`, and
`logout()`. `logout()` closes the socket and removes the active bearer token.

`connect(roomId)` returns a `WebSocketManager`. Listen to
`onConnectionChange` for connection state (`connecting`, `connected`,
`reconnecting`, or `disconnected`); use `client.on('new_message', handler)` for
message events. A successful `connected` state means authentication and the
requested room subscription completed. On reconnect, authenticate and subscribe
again, then fetch history with `getMessages` and merge it by message ID to recover
messages missed while offline.

The M1 preview uses the default single-tenant, in-memory, single-process mode.
Room and message history lasts only for that gateway process; restart does not
preserve it. Durable persistence is a later milestone.

## Migration from the earlier placeholder API

Replace fictional `login(email, password)` and `register(...)` calls with
`authenticate(externallyIssuedJwt)`. The gateway verifies the token and returns
the authenticated member identity; it does not manage passwords or issue or
refresh credentials. Create rooms with `createRoom({ name, topic? })`, list them
with `listRooms(options?)`, and use `sendMessage`, `getMessages`, and
`connect(roomId)` for collaboration. The HTTP list endpoint includes a total,
while this SDK method returns the `Room[]` from its `rooms` field.

See the [collaboration guide](../../docs/en/guides/core-collaboration.md) for
permission rules, retry limits, supported modes and migration details.
