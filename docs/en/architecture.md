# System Architecture

This document describes the technical architecture of Nexis.

## System Overview

```
┌─────────────────────────────────────────────────────────────────────┐
│                           Client Layer                               │
│   Web UI (React)  │  Mobile  │  CLI  │  SDK (TS/Python)  │  Webhooks │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                        API Gateway (Axum)                            │
│   REST API  │  WebSocket  │  Rate Limiting  │  CORS  │  Auth (JWT)  │
└───────────────────────────────┬─────────────────────────────────────┘
                                │
        ┌───────────────────────┼───────────────────────┐
        ▼                       ▼                       ▼
┌───────────────┐    ┌───────────────┐    ┌───────────────────────┐
│  Connection   │    │   Plugin      │    │   AI Integration       │
│  Pool (100K+) │    │   System      │    │   (nexis-context)      │
│  64 Shards    │    │   5 Official  │    │   Summarization        │
└───────────────┘    └───────────────┘    └───────────────────────┘
        │                       │                       │
        └───────────────────────┼───────────────────────┘
                                ▼
┌─────────────────────────────────────────────────────────────────────┐
│                      Data Layer                                      │
│   PostgreSQL  │  Redis  │  AES-256-GCM Encryption  │  S3 (Files)    │
└─────────────────────────────────────────────────────────────────────┘
```

## Technology Stack

| Layer | Technologies |
|-------|--------------|
| **Backend** | Rust 1.75+, Axum, Tokio, sqlx, tower-http |
| **Frontend** | React 18, TypeScript, Vite, TailwindCSS |
| **WebSocket** | tokio-tungstenite, ShardedConnectionManager |
| **Auth** | JWT (jsonwebtoken), Argon2id |
| **Encryption** | AES-256-GCM, Argon2id KDF |
| **Database** | PostgreSQL 16, Redis 7 |
| **Infrastructure** | Docker, Kubernetes, Helm 3 |
| **Monitoring** | Prometheus, Grafana, OpenTelemetry |
| **CI/CD** | GitHub Actions, TruffleHog OSS |
| **SDKs** | TypeScript, Python |

## Data Flow

### Message Send Flow

```
Client → API Gateway → Auth Middleware → Rate Limiter
    → Room Router → Connection Pool → Broadcast
    → Plugin Hooks → AI Summarizer (optional)
    → WebSocket Push → Clients
```

### WebSocket Connection Flow

```
Client → WS Handshake → First-Message Auth (10s timeout)
    → Connection Pool Add → Room Subscribe
    → Message Loop → Plugin Processing
    → Graceful Disconnect → Pool Remove
```

## Deployment Architecture

### Development (Docker Compose)

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ nexis-      │    │  postgres   │    │   redis     │
│ gateway     │───▶│    :5432    │───▶│    :6379    │
│   :8080     │    └─────────────┘    └─────────────┘
└─────────────┘
       │
       ▼
┌─────────────┐    ┌─────────────┐
│ prometheus  │    │   grafana   │
│   :9090     │───▶│    :3000    │
└─────────────┘    └─────────────┘
```

### Production (Kubernetes)

```
                    ┌─────────────────┐
                    │   Ingress/TLS   │
                    └────────┬────────┘
                             │
         ┌───────────────────┼───────────────────┐
         ▼                   ▼                   ▼
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│ nexis-      │    │ nexis-      │    │ nexis-      │
│ gateway-0   │    │ gateway-1   │    │ gateway-2   │
└──────┬──────┘    └──────┬──────┘    └──────┬──────┘
       │                  │                  │
       └──────────────────┼──────────────────┘
                          │
         ┌────────────────┼────────────────┐
         ▼                ▼                ▼
┌─────────────┐  ┌─────────────┐  ┌─────────────┐
│  PostgreSQL │  │   Redis     │  │  Prometheus │
│  (Primary)  │  │  Cluster    │  │  + Grafana  │
└─────────────┘  └─────────────┘  └─────────────┘
```

## Key Design Decisions

1. **Sharded Connection Pool**: 64 shards reduce lock contention for 100K+ connections
2. **First-Message Auth**: WebSocket auth via first message, not query params (security)
3. **Plugin System**: Extensible via trait-based hooks
4. **AES-256-GCM**: Static data encryption at rest
5. **GDPR Compliance**: Export and right-to-be-forgotten APIs
