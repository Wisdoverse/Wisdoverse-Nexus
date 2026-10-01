# Core collaboration guide (M1)

This guide describes the M1 collaboration flow through the Nexus gateway: an
application-issued JWT establishes a member session, HTTP manages rooms and
message history, and WebSocket carries live room events. M1 supports only the
default feature set: single tenant, in-memory storage, and one process.
Multi-tenant core flows are explicitly disabled with HTTP 503 pending M3
acceptance. Durable persistence also belongs to M3.

## Authenticate with an application-issued JWT

Nexus does not provide email/password login, account registration, token
issuance, or token refresh. Your application authenticates its users and signs
an HS256 JWT with the same runtime `JWT_SECRET` configured for the gateway.
The gateway validates the JWT through `GET /v1/auth/session` and returns
`memberId`, `memberType`, `expiresAt` in milliseconds, and
`refreshSupported: false`. Multi-tenant core flows are disabled in M1 and
return HTTP 503 pending M3 acceptance.
Treat JWTs as secrets: send them only over TLS outside local development, keep
them in a secret manager or process environment, and do not print them.

For a local synthetic token, this Python standard-library snippet creates an
HS256 JWT with a `sub` member ID and short expiry. It reads the runtime secret
from the environment; do not replace that with a committed secret, and never
reuse a development token or key in a real deployment.

```python
import base64
import hashlib
import hmac
import json
import os
import time


def b64url(data: bytes) -> str:
    return base64.urlsafe_b64encode(data).rstrip(b"=").decode("ascii")


secret = os.environ["JWT_SECRET"].encode("utf-8")
header = {"alg": "HS256", "typ": "JWT"}
now = int(time.time())
claims = {
    "sub": "m1-demo-member",
    "exp": now + 900,
    "iat": now,
    "iss": os.environ.get("JWT_ISSUER", "nexis"),
    "aud": os.environ.get("JWT_AUDIENCE", "nexis"),
    "member_type": "human",
}
signing_input = (
    f"{b64url(json.dumps(header, separators=(',', ':')).encode())}."
    f"{b64url(json.dumps(claims, separators=(',', ':')).encode())}"
)
signature = hmac.new(secret, signing_input.encode("ascii"), hashlib.sha256).digest()
token = f"{signing_input}.{b64url(signature)}"
```

The TypeScript SDK verifies the supplied token with `NexisClient.authenticate(token)`;
the Python SDK uses `await client.authenticate(token)`. Both call the session
endpoint before using authenticated APIs. The decoded token itself is not
proof of identity; rely on the gateway's validation result.

## Run the collaboration flow

Start the gateway with its runtime `JWT_SECRET`, create a short-lived local
token as above, then pass it to an SDK. The following TypeScript-shaped flow
shows the order; `room` and `message` are returned gateway models:

```ts
const client = new NexisClient({ baseUrl: "http://localhost:8080" });
const session = await client.authenticate(token);
const room = await client.createRoom({ name: "General", topic: "M1 demo" });
const message = await client.sendMessage(room.id, "Hello", undefined);
const history = await client.getMessages(room.id);

client.connect(room.id);
client.on("new_message", (event) => console.log(event));
// ...when the application session ends:
client.logout();
```

`session` contains member identity and expiry but no refresh capability.
`Room` contains `id`, `name`, optional `topic`, `messages`, and
`member_count`. `Message` contains `id`, `roomId`, `sender`, `text`, and
optional `reply_to`. Python equivalents are `create_room(CreateRoomData(...))`,
`list_rooms(limit=100, offset=0)`, `get_room`,
`send_message(room_id, text, reply_to=None)`, `get_messages`, `delete_room`,
and `invite_member`.

Room access is restricted: only the creator and invited members may read a
room, write messages, or subscribe to its live events. Only the creator may
invite members or delete the room.

HTTP `GET /v1/rooms` returns `{ "rooms": [...], "total": n }`;
`POST /v1/messages` accepts `{ "roomId": "...", "text": "...",
"replyTo": "..." }` with `replyTo` optional. Any legacy `sender` supplied by a
caller is ignored: the gateway derives the sender from the validated JWT `sub`.
`GET /v1/rooms/{id}/messages` returns a `Message[]`. HTTP and WebSocket use
the same collaboration service, and a live event is broadcast only after its
write succeeds.

Message writes support optional idempotency keys: HTTP uses `clientMessageId`
and WebSocket uses `client_message_id`. A key must be non-empty and no longer
than 128 bytes. Keys are scoped by `(sender, room, key)` and retained for 60
minutes, with a maximum of 10,000 active keys. Capacity exhaustion returns HTTP
503. Retrying with the same content returns the original message ID; reusing a
key with different content returns HTTP 409. Omit the key only when the caller
does not need retry deduplication.

## WebSocket events and recovery

Connect to `/ws` without a token or room ID in the URL. The client sends a
flat `snake_case` `auth` event; after `auth_success`, it sends `join_room` and
waits for `room_joined`. A connection is ready only after authentication and
the requested subscription succeed. The client can then send
`send_message` with `room_id`, `content`, and optional `reply_to`. The gateway
broadcasts `new_message` with `room_id`, `message_id`, `sender_id`, `content`,
optional `reply_to`, and millisecond `timestamp`. `heartbeat` receives
`heartbeat_ack`. A committed write also receives `message_accepted` with its
message ID and retry key; a retry acknowledges the original ID without a new broadcast.

On disconnect, clients reconnect, authenticate again, and restore their room
subscriptions. WebSocket delivery does not fill gaps from the offline period;
fetch `GET /v1/rooms/{id}/messages` after reconnect and merge that history into
the client view. The complete history is the recovery window for the lifetime
of the gateway process; restarting the default in-memory process clears it.
Explicit logout/close terminates the connection and stops reconnection.

## Migration from earlier examples

Examples with `login(email, password)`, `register(...)`, `RegisterData`, or
email-oriented users describe a fictional API and must be replaced with
external JWT issuance plus `authenticate(token)`. There is no refresh-token
flow. Update model access from old `content`, `sender_id`, `user`, or
`refresh_token` fields to the M1 member, `text`, and `sender` fields documented
above. Room creation uses `name` and optional `topic`; legacy join/login steps
are not required by this flow. Review callers that relied on a sender field:
the authenticated JWT subject is authoritative.

## M1 acceptance

Prepare the dependencies from a fresh checkout, then run the acceptance command:

```bash
pnpm install --frozen-lockfile --ignore-scripts
python3 -m venv .venv
.venv/bin/pip install -r tests/smoke/requirements.txt
NEXIS_SMOKE_PYTHON=.venv/bin/python bash scripts/m1_smoke.sh
```

The script builds and starts an isolated real gateway, then smoke
the TypeScript SDK, Python SDK, and Web/mobile APIs. It reports results
in `artifacts/m1/summary.json` and gateway output in
`artifacts/m1/gateway.log`; reports and logs must not record tokens or secrets.
Use those artifacts to determine acceptance status. This guide does not claim
that the command has passed.
