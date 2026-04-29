# Wisdoverse Nexus Spec

Wisdoverse Nexus is source-available AI-native collaboration infrastructure. The
repository is organized as a Rust-first monorepo with web, mobile, SDK,
documentation, and deployment surfaces that should remain reproducible from a
fresh checkout.

## Product Boundary

Wisdoverse Nexus provides:

- a Rust gateway for HTTP, WebSocket, identity, tenant, observability, and AI
  collaboration entry points;
- shared Rust protocol and domain crates under `crates/`;
- React and Expo applications under `apps/`;
- TypeScript and Python SDKs under `sdk/`;
- VitePress documentation under `docs/`;
- Docker Compose, Helm, Prometheus, and Grafana deployment assets under
  `deploy/` and `ops/`.

Public documentation and metadata should use `Wisdoverse Nexus` and
<https://wisdoverse.com>. Runtime identifiers should stay accurate to the
implementation, including `nexis-gateway`, `NEXIS_*`, crate names, Docker service
names, and SDK package names.

## Agent Handoff

When continuing this repository, start by checking the real workspace state:

```bash
git status --short
rg --files | sort
```

Use English documentation as the primary source of truth. Chinese documentation
may lag and should be updated when first-run instructions, license terms, safety
guidance, or user-visible product names change.

Do not replace Expo-managed mobile dependencies with registry-latest versions
without checking the active Expo SDK compatibility matrix. Do not rename runtime
or service identifiers only for branding consistency.

## Fresh Checkout Path

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

From another terminal:

```bash
curl http://localhost:8080/health
# OK
```

Container path:

```bash
docker compose up -d
curl http://localhost:8080/health
```

## Validation

Run checks that match the edited surface. For broad repository changes:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
git diff --check
```

## License Model

The project is source-available under the Wisdoverse Nexus Business Source
License 1.1. Non-production use is permitted without a commercial license.
Commercial production use, SaaS or hosted service offerings, managed service
offerings, resale, sublicensing, and use to build competing products require a
separate commercial license from Wisdoverse.

Each published version automatically converts to the Apache License 2.0 four
years after that version is first made publicly available by Wisdoverse.
