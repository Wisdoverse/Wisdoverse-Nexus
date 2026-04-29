# Development Guide

This guide sets up a local development environment that matches the GitHub CI
model for the source-available repository.

## Prerequisites

- Rust stable with `rustfmt` and `clippy`
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker or Docker Compose when testing containerized services

```bash
rustup component add rustfmt clippy
corepack enable
```

## Clone and Install

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

pnpm install --frozen-lockfile --ignore-scripts
cargo check --workspace
```

## Common Commands

### Rust workspace

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
cargo run -p nexis-gateway
cargo run -p nexis-cli -- --help
```

### Web app

```bash
pnpm --filter @wisdoverse/nexus-web dev
pnpm --filter @wisdoverse/nexus-web build
```

### Mobile app

The mobile app is managed by Expo. Keep dependencies aligned with the active
Expo SDK instead of blindly taking registry-latest versions.

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

### SDKs and documentation

```bash
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
```

## Local Docker

The root `docker-compose.yml` runs the gateway with local persistent storage:

```bash
docker compose up -d
docker compose ps
curl http://localhost:8080/health
```

Development infrastructure lives under `deploy/`:

```bash
docker compose -f deploy/docker-compose.yml up -d
```

## Repository Map

| Path | Description |
| --- | --- |
| `crates/nexis-protocol` | Shared protocol and identity primitives |
| `crates/nexis-gateway` | HTTP/WebSocket gateway |
| `crates/nexis-ai` | AI provider and MCP integration |
| `crates/nexis-context` | Context management |
| `crates/nexis-plugin` | Plugin runtime model |
| `crates/nexis-meeting`, `nexis-doc`, `nexis-task`, `nexis-calendar` | Collaboration domains |
| `apps/web` | React + Vite app |
| `apps/mobile` | Expo / React Native app |
| `sdk/typescript`, `sdk/python` | Client SDKs |
| `docs` | VitePress documentation site |
| `deploy` | Compose, Helm, Prometheus, and Grafana assets |

## Troubleshooting

### `cc` linker not found

Install a system C toolchain:

```bash
# Debian / Ubuntu
sudo apt install build-essential

# macOS
xcode-select --install
```

### Expo dependency mismatch

Run the compatibility check and install the versions Expo recommends:

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
```

### Port 8080 already in use

Stop the conflicting process or override the service port in your local compose
configuration before starting the gateway.

## Next Steps

- [Contributing](../development/contributing.md)
- [Architecture overview](../architecture.md)
- [API reference](../api/reference.md)
