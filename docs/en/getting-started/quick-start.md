# Quick Start

This page verifies a local Wisdoverse Nexus checkout with commands that match
the current source-available repository.

## Prerequisites

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker or Docker Compose if you want to run the containerized gateway

## Clone the Repository

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus
```

## Option 1: Run from Source

```bash
cargo check --workspace
cargo run -p nexis-gateway
```

In another terminal:

```bash
curl http://localhost:8080/health
# OK
```

The gateway also exposes:

- `GET /openapi.json` for the OpenAPI document
- `GET /docs` for Swagger UI
- `GET /metrics` for Prometheus metrics

## Option 2: Run with Docker Compose

```bash
docker compose up -d
docker compose ps
curl http://localhost:8080/health
```

Stop the stack:

```bash
docker compose down
```

## Verify the Node Workspace

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## Build Documentation Locally

```bash
pnpm --dir docs docs:build
pnpm --dir docs docs:dev
```

The development server prints a local VitePress URL.

## Next Steps

- [Development Guide](development-guide.md)
- [Architecture Overview](../architecture.md)
- [API Reference](../api/reference.md)
- [Contributing](../development/contributing.md)
