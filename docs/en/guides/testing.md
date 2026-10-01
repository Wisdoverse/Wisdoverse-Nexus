# Testing Guide

This guide documents the tests and checks that exist in the source-available
repository. Treat it as the source of truth for local verification before a pull
request.

## Test Layers

| Layer | Paths | Purpose |
| --- | --- | --- |
| Rust unit tests | `crates/*/src/**` | Module-level behavior |
| Rust integration tests | `crates/*/tests/**`, `tests/**` | Gateway, protocol, and cross-crate behavior |
| Rust benchmarks | `crates/*/benches/**` | Performance investigation, not merge gates |
| Mobile tests | `apps/mobile/src/**/__tests__/**` | Store and app behavior covered by Vitest |
| Web unit tests | `apps/web/src/**/__tests__/**` | Stores and WebSocket client behavior |
| Live gateway | `tests/smoke/`, app smoke suites, SDK live tests | Real HTTP/WS contracts, negative cases, reconnect and retry behavior |
| Browser paths | `apps/web/e2e/tests/**` | Chromium token login and room-list rendering with synthetic HTTP fixtures |
| Web build | `apps/web` | TypeScript and Vite production build |
| Architecture checks | `scripts/check_architecture.mjs`, `scripts/check_architecture.test.mjs` | Static FSD import direction/public API checks and gateway domain dependency checks |
| Deployment profile | `scripts/check_m1_deployment.py` | Render-only Helm/Kubernetes controls, external Secret/image references and rejected unsafe M1 configurations |
| SDK build | `sdk/typescript` | TypeScript SDK compilation |
| Docs build | `docs` | VitePress build with strict dead-link checking |

## Full Local Gate

Run this for broad changes:

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo check --locked --workspace
cargo test --locked --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm test:architecture
pnpm check:architecture
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-web exec vitest run
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
```

## Rust Commands

```bash
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo check --locked --workspace
cargo test --locked --workspace
```

The Linux/Docker deployment gate installs `PyYAML==6.0.3` and runs
`python3 scripts/check_m1_deployment.py`. It uses a digest-pinned Helm tool
container with no container network and synthetic references. It validates
single-instance/Recreate upgrades, image digest and Secret injection, probes,
non-root execution and resource budgets. Missing Secrets, multiple replicas
and HPA must fail chart rendering. This gate creates no cluster resources and
does not qualify cluster runtime, persistence or scaling. Isolated gateway tests
also verify production configuration, HTTP health probes with HTTPS redirect,
and draining an accepted HTTP response at shutdown.

Focused examples:

```bash
cargo test -p nexis-gateway --test api_integration
cargo test -p nexis-gateway --test boundary_conditions
cargo test -p nexis-protocol
```

The gateway integration tests build requests against `build_routes()` and use
test JWT secrets. Keep examples aligned with the current `/v1/*` route contract.

## Node Workspace Commands

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web exec vitest run
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
```

The architecture checker inspects relative TypeScript imports in Web/mobile
source modules and transport/storage/runtime dependencies in gateway domain
modules. It excludes tests and declaration files. It is a static boundary check,
not proof of complete FSD/DDD compliance or runtime behavior. CI runs both the
checker behavior tests and the repository check in its Node job.

## Mobile Commands

The mobile app is Expo-managed. Dependency upgrades must pass Expo compatibility
checks.

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## Live gateway contracts

Follow the [M1 setup](core-collaboration.md#m1-acceptance) and run:

```bash
NEXIS_SMOKE_PYTHON=<prepared-python> bash scripts/m1_smoke.sh
```

The script builds a locked gateway and SDK and launches an isolated process.
The runner exercises actual Web/mobile adapters and both SDKs with synthetic
credentials, validates served OpenAPI responses, and checks access, expiry,
retry deduplication and recovery. Retain `artifacts/m1/summary.json` and sanitized
logs with the tested revision. The [M1 report](m1-acceptance.md) separates these
results from broader deployment, coverage and performance targets.

## Web browser paths

From the repository root:

```bash
npm ci --ignore-scripts --prefix apps/web/e2e
(cd apps/web/e2e && npx playwright install --with-deps chromium)
(cd apps/web/e2e && npx playwright test)
```

The Playwright config starts the Web dev server. These two paths use synthetic
HTTP fixtures for accepted/rejected token login and the room-list envelope;
real transport is verified by the separate gateway suite. If local browser or
Expo metadata downloads are blocked, report that restriction and use the linked
public CI result; offline metadata is insufficient compatibility evidence.

## Documentation Commands

```bash
pnpm --dir docs docs:build
```

The VitePress build checks internal links. Do not re-enable dead-link ignoring
to hide broken documentation.

## Authentication configuration

Set a nonblank `JWT_SECRET` for production (`NEXIS_ENV=production`); the gateway
fails startup when it is missing or blank. Development can generate an
ephemeral secret, which cannot validate externally issued tokens unless the
matching secret is configured. Keep secrets out of source, logs, and test
artifacts; use synthetic credentials for local and CI verification.

Mobile startup restores a saved token only after validating it through
`/auth/session`. A rejected token is removed; a network failure leaves the
session available for retry. The loading gate prevents protected screens from
appearing before restoration finishes, and logout cannot be overwritten by a
late restoration response.

## Audits

```bash
pnpm audit --audit-level moderate
npm --prefix apps/mobile audit --audit-level=moderate
npm --prefix docs audit --audit-level=moderate
npm --prefix apps/web/e2e audit --audit-level=moderate
```

Rust advisory checks require `cargo-audit`:

```bash
cargo audit
```

## Benchmarks

Benchmarks are for investigation and capacity planning. Include hardware,
commit SHA, and exact commands when sharing results.

```bash
cargo bench --workspace
cargo bench -p nexis-gateway --bench websocket_connections
cargo bench -p nexis-gateway --bench message_throughput
cargo bench -p nexis-gateway --bench routing
```

## Pull Request Evidence

Include the commands you ran and the result in the PR description. If a command
is skipped, explain why it was not relevant to the change.
