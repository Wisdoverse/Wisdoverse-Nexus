# Development Guide

This guide sets up a local development environment that matches the GitHub CI
model for the source-available repository.

## Prerequisites

- Rust 1.98.1 with `rustfmt` and `clippy` (qualified repository toolchain)
- Node.js `24.x`
- pnpm `>=10.30.0`
- Docker or Docker Compose when testing containerized services

```bash
rustup toolchain install 1.98.1 --profile minimal --component rustfmt --component clippy
corepack enable
```

The repository's `rust-toolchain.toml` automatically selects Rust 1.98.1,
including in CI. Compiler upgrades require separate qualification.

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

## Architecture requirements

The [repository instructions](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/AGENTS.md)
and [ADR-008](../architecture/adr/008-fsd-ddd-cloud-native-services.md) require:

- FSD for Web/mobile: downward imports through slice public APIs; domain state
  in entities, user actions in features, and reusable transport/config/UI in shared.
- DDD for backends: framework-free domains, application use cases, and explicit
  infrastructure/interface adapters with bounded-context contracts.
- Cloud-native microservices: service-owned data, independent artifacts,
  versioned API/event contracts, bounded failure handling, non-root containers,
  injected secrets, probes/resource limits, graceful shutdown, telemetry and
  evidenced rollout/recovery/rollback.

Record implementation gaps and migration scope in the PR. M1 currently qualifies
one single-tenant, in-memory process; these requirements guide service evolution.

## Local Docker

The root `docker-compose.yml` starts the gateway and mounts a data volume.
M1 room/message state uses in-memory storage: a volume does not establish
restart durability. Persistence/recovery acceptance belongs to M3.

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
