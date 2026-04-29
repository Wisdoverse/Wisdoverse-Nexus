# WebSocket API

## Overview

The Wisdoverse Nexus Gateway provides a real-time WebSocket endpoint at `/ws` for bidirectional communication. All messages are JSON-encoded.

## Authentication

Wisdoverse Nexus uses a **first-message authentication** pattern to keep tokens out of URLs and logs.

### Recommended: First-Message Auth

```text
Client                                          Server
  |                                                |
  |--- WebSocket Connect (no token) ------------->|
  |                                                |
  |--- {"type":"auth","token":"Bearer xxx"} ------>|
  |                                                |
  |<-- {"type":"auth_success","member_id":"...",   |
  |     "member_type":"..."} ----------------------|
  |                                                |
  |--- (other messages) ------------------------->|
```

1. Connect to `ws://host:8080/ws` (no token in URL)
2. Send an auth message as the **first message** within 10 seconds
3. Receive `auth_success` or `auth_error`
4. Only after successful auth can you send other messages

### Authentication Timeout

Connections must authenticate within **10 seconds** or they will be closed with an `auth_timeout` error.

### Legacy: Query Parameter Auth (Deprecated)

> ⚠️ **Deprecated**: Passing tokens via query parameter (`?token=Bearer xxx`) still works but is not recommended. Tokens in URLs may appear in server logs, proxy logs, and browser history.

The server logs a deprecation warning when this method is used. Please migrate to first-message auth.

## Message Types

### Client → Server Messages

| Type | Description |
|------|-------------|
| `auth` | Authenticate the connection (must be first message) |
| `heartbeat` | Ping/keepalive |
| `join_room` | Join a chat room |
| `leave_room` | Leave a chat room |
| `send_message` | Send a message to a room |

#### `auth`

```json
{
  "type": "auth",
  "token": "Bearer <jwt_token>"
}
```

#### `heartbeat`

```json
{
  "type": "heartbeat",
  "timestamp": 1700000000
}
```

#### `join_room`

```json
{
  "type": "join_room",
  "room_id": "room_abc123"
}
```

#### `leave_room`

```json
{
  "type": "leave_room",
  "room_id": "room_abc123"
}
```

#### `send_message`

```json
{
  "type": "send_message",
  "room_id": "room_abc123",
  "content": "Hello!",
  "reply_to": "msg_optional_id"
}
```

### Server → Client Messages

| Type | Description |
|------|-------------|
| `auth_success` | Authentication successful |
| `auth_error` | Authentication failed |
| `auth_required` | Sent when a non-auth message is received before auth |
| `heartbeat_ack` | Response to heartbeat |
| `room_joined` | Room join confirmed |
| `room_left` | Room leave confirmed |
| `new_message` | New message received in a joined room |
| `error` | General error |

#### `auth_success`

```json
{
  "type": "auth_success",
  "member_id": "nexis:human:alice@example.com",
  "member_type": "human"
}
```

#### `auth_error`

```json
{
  "type": "auth_error",
  "message": "Token has expired",
  "code": "TOKEN_EXPIRED"
}
```

Error codes: `AUTH_FAILED`, `TOKEN_EXPIRED`, `AUTH_TIMEOUT`

#### `auth_required`

```json
{
  "type": "auth_required",
  "message": "Authentication required. Please send an 'auth' message first."
}
```

#### `error`

```json
{
  "type": "error",
  "message": "Invalid message format: ...",
  "code": "PARSE_ERROR"
}
```

## Connection Lifecycle

1. **Connect** — Open WebSocket to `/ws`
2. **Authenticate** — Send `auth` message within 10 seconds
3. **Interact** — Join rooms, send/receive messages
4. **Disconnect** — Client or server closes connection

If authentication is not completed within 10 seconds, the server sends an `auth_timeout` error and closes the connection.

## Example (JavaScript)

```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // First message must be auth
  ws.send(JSON.stringify({
    type: 'auth',
    token: `Bearer ${jwtToken}`
  }));
};

ws.onmessage = (event) => {
  const msg = JSON.parse(event.data);

  switch (msg.type) {
    case 'auth_success':
      console.log('Authenticated as', msg.member_id);
      // Now join rooms, send messages, etc.
      ws.send(JSON.stringify({ type: 'join_room', room_id: 'room_general' }));
      break;

    case 'auth_error':
      console.error('Auth failed:', msg.message, msg.code);
      ws.close();
      break;

    case 'new_message':
      console.log(`[${msg.room_id}] ${msg.sender_id}: ${msg.content}`);
      break;

    case 'heartbeat_ack':
      // Pong received
      break;
  }
};
```
