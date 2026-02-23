# API Reference

Nexis provides REST and WebSocket APIs for integration.

## Base URL

```
https://api.nexis.ai/v1
```

## Authentication

All API requests require a JWT token in the Authorization header:

```
Authorization: Bearer <token>
```

## Endpoints

### Rooms

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /rooms | Create a room |
| GET | /rooms | List rooms |
| GET | /rooms/{id} | Get room details |
| DELETE | /rooms/{id} | Delete a room |

### Messages

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /rooms/{id}/messages | Send a message |
| GET | /rooms/{id}/messages | List messages |
| GET | /messages/{id} | Get message |

### Search

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /search | Semantic search |
| GET | /search/suggestions | Search suggestions |

## WebSocket

Connect to `/ws` for real-time messaging.

### Events

- `message:create` - New message
- `message:update` - Message updated
- `room:join` - User joined room
- `room:leave` - User left room

## Error Codes

| Code | Description |
|------|-------------|
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Not Found |
| 429 | Rate Limited |
| 500 | Internal Error |
