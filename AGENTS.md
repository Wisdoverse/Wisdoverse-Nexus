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

## Testing

Rust integration tests in `tests/`; crate-local tests for crate-specific behavior. Name by behavior: `rejects_invalid_tenant_token`. Mobile unit tests: Vitest in `__tests__/`. Web e2e: `apps/web/e2e/tests/`.

## Env

Copy `.env.example` or `deploy/.env.example`. Required: `JWT_SECRET`, `NEXIS_DEFAULT_PROVIDER` (default `mock`), `NEXIS_DATABASE_PATH`, `RUST_LOG`. Never commit secrets — pre-commit runs `gitleaks`, `detect-secrets`, fmt, clippy.

## Gotchas

- `plugins/*` excluded from cargo workspace — build individually: `cargo build --manifest-path plugins/<name>/Cargo.toml`.
- Mobile uses Vitest (not Jest) on Expo 55 + RN 0.83 + React 19.
- Clippy bar strict (pedantic + nursery + cargo). Run clippy before push.
- pnpm overrides pin `esbuild`, `postcss`, `vite`, `xcode>uuid` — don't bump blindly.

## Commits & PRs

Conventional Commits, scoped: `feat(gateway): add tenant-aware room lookup`, `fix(mobile): align Expo dependencies`. PRs: explain change, link issues, include verification commands + results, add screenshots/logs for UI/CLI/deploy. Update `CHANGELOG.md` Unreleased for user-visible changes.

## Security

Vulnerabilities: see `SECURITY.md`. No secrets in commits.
