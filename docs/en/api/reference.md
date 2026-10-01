# API Reference

Wisdoverse Nexus provides REST and WebSocket APIs for integration.

## Base URL

```
http://localhost:8080
```

This is the local development gateway. For a self-hosted deployment, use your
configured URL. Public examples should use `https://api.example.com` as a
placeholder and omit private deployment addresses.

All v1 endpoints are prefixed with `/v1`.

## Authentication

All `/v1/*` API requests require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

Unauthenticated requests to protected endpoints return `401 Unauthorized`.
The supported M1 mode is one process with default features and in-memory,
single-tenant storage. Tenant-scoped credentials are rejected. See the
[core collaboration guide](../guides/core-collaboration.md) for external JWT signing,
retry limits, and migration. The served `/openapi.json` is the schema source.

### Verify session

`GET /v1/auth/session` returns the verified identity:

```json
{"memberId":"demo-member","memberType":"human","expiresAt":1893456000000,"refreshSupported":false}
```

The gateway does not issue or refresh tokens. Use externally issued HS256 JWTs.

## Endpoints

### Health Check

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | /health | Health check | No |

**Response:** `200 OK` - Plain text `OK`

### Rooms API

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | /v1/rooms | List rooms | Yes |
| POST | /v1/rooms | Create a room | Yes |
| GET | /v1/rooms/{id} | Get room details | Yes |
| GET | /v1/rooms/{id}/messages | Ordered message history | Yes |
| DELETE | /v1/rooms/{id} | Delete a room | Yes |
| POST | /v1/rooms/{id}/invite | Invite member | Yes |

Only the creator and invited members may read, send, or subscribe. Only the
creator may invite members or delete a room; unauthorized access returns 403.
Listing includes only accessible rooms; pagination and `total` apply to that set.

#### List Rooms

Query parameters:
- `limit` (optional, default: 100, max: 1000) - Max rooms to return
- `offset` (optional, default: 0) - Pagination offset

Response:
```json
{
  "rooms": [
    {
      "id": "room_abc123",
      "name": "general",
      "topic": "Team chat",
      "member_count": 5
    }
  ],
  "total": 42
}
```

#### Create Room

Request:
```json
{
  "name": "general",
  "topic": "Team chat"
}
```

Response: `201 Created`
```json
{
  "id": "room_abc123",
  "name": "general"
}
```

#### Get Room

Response:
```json
{
  "id": "room_abc123",
  "name": "general",
  "topic": "Team chat",
  "messages": [
    {
      "id": "msg_xyz",
      "roomId": "room_abc123",
      "sender": "alice",
      "text": "Hello!"
    }
  ]
}
```

#### Delete Room

Response: `204 No Content` (empty body)

### Messages API

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| POST | /v1/messages | Send a message | Yes |

#### Send Message

Request:
```json
{
  "roomId": "room_abc123",
  "clientMessageId": "demo-write-1",
  "text": "Hello, world!",
  "replyTo": null
}
```

Response: `201 Created`
```json
{
  "id": "msg_xyz",
  "roomId": "room_abc123",
  "sender": "demo-member",
  "text": "Hello, world!"
}
```

The sender comes from the JWT subject; a legacy `sender` input is ignored.
Replies must reference a message in the same room. Repeating a
`clientMessageId` with the same text and reply returns the original ID. Changed
content returns 409; keys are retained for 60 minutes within one process.
`GET /v1/rooms/{id}/messages` returns an ordered array with the same message shape.

### Search API

Search requires a configured search service and is outside the M1 qualification profile.

| Method | Endpoint | Description | Auth |
|--------|----------|-------------|------|
| GET | /v1/search | Semantic search | Yes |
| POST | /v1/search | Semantic search | Yes |

#### Search Messages

Query parameters:
- `q` (required) - Search query string
- `limit` (optional, default: 10) - Max results
- `min_score` (optional) - Minimum relevance score
- `room_id` (optional, UUID) - Filter by room

Response:
```json
{
  "query": "project updates",
  "results": [
    {
      "id": "550e8400-e29b-41d4-a716-446655440000",
      "score": 0.95,
      "content": "Here are the latest project updates...",
      "room_id": "550e8400-e29b-41d4-a716-446655440001"
    }
  ],
  "total": 5
}
```

## WebSocket API

Connect to `/ws`, send `auth` as the first message within 10 seconds, then
`join_room` after `auth_success`. Await `room_joined` before sending messages.
Tokens are never required in the URL. Live `new_message` events share IDs,
senders, content, and reply references with HTTP history. WebSocket writes
receive `message_accepted`, including on an idempotent retry.

See the [WebSocket contract](websocket.md) for flat JSON event shapes and recovery.

## Error Handling

Room/message application errors return the following JSON shape. Authentication
failures and Axum request extraction can return plain-text bodies; parse the
HTTP status first and handle either representation. WebSocket errors use the
event shapes in the [WebSocket contract](websocket.md).

```json
{
  "error": "Human-readable error message",
  "code": "ERROR_CODE"
}
```

### Error Codes

| Code | HTTP Status | Description |
|------|-------------|-------------|
| BAD_REQUEST | 400 | Invalid request parameters |
| — | 401 | Missing, invalid or expired authentication; body may be plain text |
| FORBIDDEN | 403 | Insufficient permissions |
| NOT_FOUND | 404 | Resource not found |
| CONFLICT | 409 | Retry key reused with different content |
| UNSUPPORTED_MODE | 503 | Multi-tenant core flow is outside M1 support |
| SERVICE_UNAVAILABLE | 503 | Service temporarily unavailable |
| INTERNAL_ERROR | 500 | Internal server error |
| INVALID_QUERY | 400 | Invalid search query |
| SEARCH_UNAVAILABLE | 503 | Search service not configured |

## Rate Limiting

API endpoints are protected by a write gate to prevent overload. When at capacity, endpoints return `503 Service Unavailable` with code `SERVICE_UNAVAILABLE`.
