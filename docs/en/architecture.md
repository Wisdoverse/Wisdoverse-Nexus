# Architecture Overview

Wisdoverse Nexus is a Rust-first collaboration platform organized as a
monorepo. The public architecture is intentionally described from the repository
state, not from future product plans.

## System Shape

```text
Client apps and SDKs
  apps/web
  apps/mobile
  sdk/typescript
  sdk/python
        |
        v
Gateway
  crates/nexis-gateway
  HTTP routes
  WebSocket endpoint
  metrics
  OpenAPI document
        |
        v
Domain crates
  nexis-protocol
  nexis-ai
  nexis-context
  nexis-plugin
  nexis-meeting
  nexis-doc
  nexis-task
  nexis-calendar
  nexis-memory
  nexis-skills
```

## Core Areas

| Area | Current repository paths | Responsibility |
| --- | --- | --- |
| Gateway | `crates/nexis-gateway` | HTTP routing, WebSocket upgrade, metrics, middleware, API docs |
| Protocol | `crates/nexis-protocol` | Shared identity, message, permission, and tenant primitives |
| AI integration | `crates/nexis-ai`, `crates/nexis-mcp` | Provider and MCP integration surfaces |
| Context and memory | `crates/nexis-context`, `crates/nexis-memory`, `crates/nexis-vector` | Context windows, memory structures, vector integration boundaries |
| Collaboration domains | `crates/nexis-meeting`, `crates/nexis-doc`, `crates/nexis-task`, `crates/nexis-calendar` | Meeting, document, task, and calendar domain modules |
| Plugin and skills | `crates/nexis-plugin`, `crates/nexis-skills` | Extensibility and skill execution model |
| Apps | `apps/web`, `apps/mobile` | Web and mobile clients |
| SDKs | `sdk/typescript`, `sdk/python` | Client integration packages |
| Deployment | `docker-compose.yml`, `deploy/`, `docker/` | Local gateway stack and deployment assets |

## Gateway Surface

The gateway currently exposes:

| Endpoint | Purpose |
| --- | --- |
| `GET /health` | Basic liveness response |
| `GET /metrics` | Prometheus metrics |
| `GET /openapi.json` | Gateway OpenAPI document |
| `GET /docs` | Swagger UI for the OpenAPI document |
| `GET /ws` | WebSocket upgrade endpoint |
| `POST /v1/rooms` | Create a room |
| `GET /v1/rooms` | List rooms |
| `GET /v1/rooms/:id` | Read room state |
| `DELETE /v1/rooms/:id` | Delete a room |
| `POST /v1/messages` | Send a message |
| `GET/POST /v1/search` | Search messages when search service is configured |

Collaboration-specific endpoints are registered under `/v1/collaboration/*`.

## WebSocket Authentication

The preferred WebSocket authentication pattern is first-message authentication.
Legacy query-token authentication is still accepted by the gateway with a
deprecation warning. New clients should avoid putting tokens in URLs because URLs
can be logged by infrastructure.

## Local Deployment Model

The root `docker-compose.yml` runs the gateway with local persistent storage:

```bash
docker compose up -d
curl http://localhost:8080/health
```

The development compose file under `deploy/docker-compose.yml` adds PostgreSQL,
Redis, Prometheus, and Grafana for local integration work:

```bash
docker compose -f deploy/docker-compose.yml up -d
```

## Production Guidance

Production operators should treat this repository as deployable source and
complete their own environment-specific hardening:

- TLS termination and HSTS at the edge
- Secret injection through a secrets manager
- Explicit CORS origins
- Database backups and restore testing
- Metrics, logs, and alerting retention
- CPU, memory, and file descriptor limits
- Load tests in the target environment

See [Deployment](operations/deployment.md), [Monitoring](operations/monitoring.md),
and [Security Overview](architecture/security/overview.md).

## Architecture Decisions

Significant design decisions are tracked in [ADRs](architecture/adr/README.md).
