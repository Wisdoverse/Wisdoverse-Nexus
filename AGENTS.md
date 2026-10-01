# Repository Guidelines

Wisdoverse Nexus: Rust + TypeScript monorepo. Node 24.x, pnpm `>=10.30.0`, Rust edition 2021.

## Layout

- `crates/nexis-*` — 16 Rust workspace crates: `protocol`, `gateway`, `ai`, `mcp`, `vector`, `context`, `federation`, `memory`, `skills`, `meeting`, `doc`, `task`, `calendar`, `plugin`, `billing`, `cli`.
- `apps/web` — Vite + React 19 + Tailwind 4. `apps/mobile` — Expo 55 + RN 0.83 + Vitest.
- `sdk/typescript`, `sdk/python` — client SDKs.
- `tests/` — Rust integration, security, stress, e2e.
- `plugins/` — example plugins, **excluded from cargo workspace** (skipped by `cargo *--workspace`; build per-plugin).
- `examples/`, `docs/`, `deploy/`, `docker/`, `k8s/`, `ops/`, `scripts/`, `protocol/`, `SPEC.md`.

## Commands

```bash
pnpm install --frozen-lockfile --ignore-scripts

# Rust
cargo check --workspace
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all -- --check

# Web / SDK / docs
pnpm check:architecture
pnpm test:architecture
python3 scripts/check_m1_deployment.py          # Docker + PyYAML 6.0.3; render-only checks
pnpm --filter @wisdoverse/nexus-web dev          # local dev server
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build

# Mobile
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check

# Local stack + scripts
docker compose up                                 # gateway on :8080
scripts/coverage.sh
scripts/e2e_test.sh
```

## Style

Rust: edition 2021, 4-space indent, Unix newlines, 100-col, `unsafe_code = "forbid"`. Workspace lints set `clippy::pedantic`, `nursery`, `cargo` to warn — fix or `#[allow]` with reason.
Crate names: `nexis-*` kebab/snake.
TypeScript: strict tsconfig, PascalCase React components, camelCase functions/vars, colocated CSS modules where used.

## Architecture

Required architecture: frontend follows Feature-Sliced Design (FSD), backend follows Domain-Driven Design (DDD), and service design follows cloud-native microservices. Apply these requirements to Web, mobile, backend services, and new deployment work.

Run `pnpm test:architecture` for checker behavior and `pnpm check:architecture` for current frontend import boundaries and gateway domain dependency checks. These static checks cover Web/mobile source modules and gateway domain modules; they do not prove complete architectural compliance, runtime behavior, independent service deployment, persistence, or production scaling.

Frontend must follow Feature-Sliced Design (FSD). Organize application code by layers in dependency order: `app` -> `pages` -> `widgets` -> `features` -> `entities` -> `shared`. A layer may import only from lower layers, never from a higher layer or sideways through another feature. Keep public APIs explicit through local `index.ts` barrels; avoid deep cross-slice imports. Place reusable UI, API clients, config, and primitives in `shared`; domain objects and stores in `entities`; user actions in `features`; composed surfaces in `widgets`; route-level screens in `pages`; providers, routing, and app bootstrapping in `app`.

Backend must follow Domain-Driven Design (DDD). Keep domain models, value objects, aggregates, domain errors, and domain services free of transport, database, and framework concerns. Put use-case orchestration in application services; keep Axum handlers, SQL/storage adapters, external providers, queues, and observability in infrastructure/interface layers. Crate boundaries should preserve bounded contexts (`nexis-*`), and cross-context communication should use explicit contracts/events rather than shared mutable internals. Do not leak database row models or HTTP DTOs into domain APIs.

Cloud-native microservices must have explicit bounded-context ownership, versioned API/event contracts, and independently buildable and deployable service artifacts. Each service owns its data and migrations; access another service's data through its published contracts. Keep durable state outside replaceable service instances in qualified deployment profiles. Define timeouts, bounded queues, retry/backoff and idempotency at service boundaries.

Container and Kubernetes changes must use reproducible locked builds, compatible base/runtime images, non-root execution, external configuration and secret injection, resource requests/limits, startup/readiness/liveness probes, graceful shutdown and request draining. Provide structured logs, metrics and propagated trace/correlation IDs; redact credentials and private content. Document rollout, rollback and schema compatibility for affected services.

Architecture changes must update the relevant ADR, contracts and developer/deployment documentation, with verification evidence in the PR. Document current implementation gaps and migration plans explicitly. The M1 single-process, in-memory preview remains a documented evaluation profile; cloud-native requirements do not establish production, persistence or horizontal-scaling support without acceptance evidence.

## Testing

Rust integration tests in `tests/`; crate-local tests for crate-specific behavior. Name by behavior: `rejects_invalid_tenant_token`. Mobile unit tests: Vitest in `__tests__/`. Web e2e: `apps/web/e2e/tests/`.

## Env

Copy `.env.example` or `deploy/.env.example`. Configure `JWT_SECRET`, `NEXIS_DEFAULT_PROVIDER` (default `mock`), `NEXIS_DATABASE_PATH`, and `RUST_LOG`. When `NEXIS_ENV=production`, the gateway refuses to start if `JWT_SECRET` is missing or blank. Development may use an ephemeral random secret; externally issued tokens require an explicitly configured matching secret. Never commit secrets — pre-commit runs `gitleaks`, `detect-secrets`, fmt, clippy.

## Gotchas

- `plugins/*` excluded from cargo workspace — build individually: `cargo build --manifest-path plugins/<name>/Cargo.toml`.
- Mobile uses Vitest (not Jest) on Expo 55 + RN 0.83 + React 19.
- Clippy bar strict (pedantic + nursery + cargo). Run clippy before push.
- pnpm overrides pin `esbuild`, `postcss`, `vite`, `xcode>uuid` — don't bump blindly.

## Commits & PRs

Conventional Commits, scoped: `feat(gateway): add tenant-aware room lookup`, `fix(mobile): align Expo dependencies`. PRs: explain change, link issues, include verification commands + results, add screenshots/logs for UI/CLI/deploy. Update `CHANGELOG.md` Unreleased for user-visible changes.

## Security

Vulnerabilities: see `SECURITY.md`. No secrets in commits.
