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
| Web build | `apps/web` | TypeScript and Vite production build |
| SDK build | `sdk/typescript` | TypeScript SDK compilation |
| Docs build | `docs` | VitePress build with strict dead-link checking |

## Full Local Gate

Run this for broad changes:

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
```

## Rust Commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
```

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
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
```

## Mobile Commands

The mobile app is Expo-managed. Dependency upgrades must pass Expo compatibility
checks.

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## Documentation Commands

```bash
pnpm --dir docs docs:build
```

The VitePress build checks internal links. Do not re-enable dead-link ignoring
to hide broken documentation.

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
