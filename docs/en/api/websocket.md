# WebSocket API

The gateway exposes `/ws` for flat JSON events. M1 supports default features,
one process, single-tenant in-memory storage. Multi-tenant core endpoints fail
with HTTP 503 pending M3. See the [collaboration guide](../guides/core-collaboration.md)
for JWT issuance, HTTP contracts, and client migration.

## Authenticate and subscribe

1. Connect to `ws://localhost:8080/ws` for local development; use `wss://` over TLS elsewhere.
2. Send `auth` within 10 seconds. A raw token or `Bearer <token>` is accepted.
3. Await `auth_success`, send `join_room`, then await `room_joined`.
4. Send or receive messages only in authorized, joined rooms.

```json
{"type":"auth","token":"Bearer <application-issued-jwt>"}
```

```json
{"type":"auth_success","member_id":"demo-member","member_type":"human"}
```

The HTTP session endpoint and WebSocket validate the same signing key, issuer,
audience, expiry, and member identity. Invalid or expired credentials receive
`auth_error` and the connection closes. A non-auth operation before authentication
receives `auth_required`; timeout receives `auth_error` with `AUTH_TIMEOUT`.
An established connection also closes when its JWT expires. There is no refresh
endpoint; obtain a new token from the issuing application.

Query-token authentication remains deprecated. Clients and SDKs use first-message
authentication to avoid exposing credentials in URLs.

## Client events

| Type | Fields | Effect |
| --- | --- | --- |
| `auth` | `token` | Establish a verified session |
| `join_room` | `room_id` | Subscribe after checking creator/member access |
| `leave_room` | `room_id` | Stop this subscription |
| `heartbeat` | optional `timestamp` | Receive a flat `heartbeat_ack` with the same timestamp |
| `send_message` | `room_id`, `content`, optional `reply_to`, optional `client_message_id` | Commit a message as the authenticated member |

```json
{"type":"join_room","room_id":"room_abc123"}
```

```json
{"type":"send_message","room_id":"room_abc123","content":"Hello!","client_message_id":"write-1"}
```

Only creators and invited members may join or write. Access is checked again on
writes and event delivery. Replies must reference an existing message in the same
room. A `client_message_id` is scoped by sender and room: identical retries within
60 minutes return the original ID; different content for that key is rejected.
Keys must be 1–128 bytes; the process retains at most 10,000 active keys and refuses
new keyed writes at capacity. These rules match HTTP `clientMessageId`.

## Server events

| Type | Fields | Meaning |
| --- | --- | --- |
| `auth_success` | `member_id`, `member_type` | Verified identity |
| `auth_error` | `message`, `code` | `AUTH_FAILED`, `TOKEN_EXPIRED`, or `AUTH_TIMEOUT`; terminal |
| `auth_required` | `message` | Authentication must precede operations |
| `room_joined` / `room_left` | `room_id` | Subscription acknowledgement |
| `heartbeat_ack` | `timestamp` | Echoed timestamp, or null |
| `message_accepted` | `room_id`, `message_id`, `client_message_id` | Write or retry accepted; key may be null |
| `new_message` | `room_id`, `message_id`, `sender_id`, `content`, `reply_to`, `timestamp` | Newly committed room message; timestamp is milliseconds |
| `error` | `message`, `code` | Explicit parse, authorization, write, or recovery failure |

```json
{"type":"message_accepted","room_id":"room_abc123","message_id":"msg_xyz","client_message_id":"write-1"}
```

```json
{"type":"new_message","room_id":"room_abc123","message_id":"msg_xyz","sender_id":"demo-member","content":"Hello!","reply_to":null,"timestamp":1893456000000}
```

A successful new write produces one room broadcast. Every successful keyed retry
receives `message_accepted` with the original identity without another broadcast.
HTTP writes use the same service and broadcast after the repository accepts them.
Clients must merge HTTP acknowledgements, broadcasts, and history by message ID.

## Recovery and limits

Reconnect, authenticate again, subscribe, and fetch `GET /v1/rooms/{id}/messages`.
Web/mobile stores merge history on reconnect. SDK applications explicitly fetch
history after restoring their subscription. WebSocket events are live delivery;
HTTP history fills offline gaps for the lifetime of the process. Process restart
clears the default store and retry ledger; durable recovery is an M3 requirement.

The shared event channel retains 1,024 events; each socket has a bounded 256-frame
outgoing queue. A lagging subscription receives `SYNC_REQUIRED` when it can be
delivered and closes; an overloaded outgoing queue closes the socket. Recover
through history. These disconnects never remove accepted messages from the store.
A new connection for the same member replaces the previous socket. Explicit
logout/close cancels SDK/client reconnect attempts.

Malformed frames receive `PARSE_ERROR`, unauthorized access receives `FORBIDDEN`,
and failed writes return their application error. Message acknowledgement means
accepted by the supported in-memory store, not durable across crashes. See the
[M1 acceptance report](../guides/m1-acceptance.md) for tested scenarios and limits.
