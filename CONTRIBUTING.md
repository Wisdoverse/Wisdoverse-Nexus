# Contributing to Wisdoverse Nexus

Thank you for taking the time to contribute. This guide describes the workflow
expected for a public GitHub repository: small pull requests, reproducible
commands, clear issue linkage, and documentation updates when behavior changes.

By participating, you agree to follow the [Code of Conduct](CODE_OF_CONDUCT.md).

## Before You Start

Use GitHub issues for work coordination:

- Search existing issues before opening a new one.
- Use a bug report for reproducible defects.
- Use a feature request for new behavior or public API changes.
- For security vulnerabilities, do not open a public issue. Follow [SECURITY.md](SECURITY.md).

Maintainers may close issues that do not include enough information to reproduce
or evaluate the request.

## Development Requirements

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker or Docker Compose for containerized local runs

Install Node dependencies from the repository root:

```bash
pnpm install --frozen-lockfile --ignore-scripts
```

## Repository Areas

| Area | Main paths | Typical checks |
| --- | --- | --- |
| Rust backend and protocol | `crates/`, `Cargo.toml`, `Cargo.lock` | `cargo fmt`, `cargo clippy`, `cargo check`, `cargo test` |
| Web app | `apps/web/` | `pnpm --filter @wisdoverse/nexus-web build` |
| Mobile app | `apps/mobile/` | Expo compatibility, typecheck, Vitest |
| TypeScript SDK | `sdk/typescript/` | `pnpm --filter @wisdoverse/nexus-sdk build` |
| Python SDK | `sdk/python/` | package-specific tests when changed |
| Documentation | `README*.md`, `SPEC.md`, `docs/` | `pnpm --dir docs docs:build` |
| CI and release | `.github/`, `deploy/`, Dockerfiles | workflow syntax, dry-run-equivalent local checks |

## Branches and Commits

Create topic branches from `main`:

```bash
git checkout main
git pull --ff-only
git checkout -b feat/short-topic
```

Use Conventional Commits where practical:

```text
feat(gateway): add tenant-aware room lookup
fix(mobile): align Expo dependency versions
docs(readme): clarify local verification commands
test(protocol): cover invalid member identifiers
chore(deps): update workspace lockfiles
```

Keep unrelated cleanup out of feature PRs. If a change touches multiple areas,
explain why in the PR description.

## Local Verification

Run the checks that match your change. For broad changes, run the full local
gate:

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

Dependency changes must include the relevant lockfile updates. For the Expo app,
prefer SDK-compatible versions over registry-latest versions; `expo install
--check` is the source of truth for mobile dependency compatibility.

## Documentation Standards

Documentation changes should be accurate for the current repository state.

- Prefer relative links for files inside the repository.
- Link the official documentation site as <https://wisdoverse.com>.
- Do not add unverified customer names, certification claims, or performance
  claims.
- Mark experimental or planned capabilities explicitly.
- Update both English and Chinese public entry points when the change affects
  first-run instructions.
- Keep runtime identifiers such as crate names, environment variables, and
  container names accurate even when public branding changes.

Build the docs before submitting:

```bash
pnpm --dir docs docs:build
```

## Pull Requests

A good pull request includes:

- What changed and why.
- Linked issues, for example `Closes #123`.
- User-facing or API impact.
- Migration notes for breaking changes.
- Verification commands and their results.
- Screenshots or logs when UI, CLI, or deployment behavior changes.

Maintainers may request smaller PRs when a change is hard to review. CI must pass
before merge.

## Review Expectations

Reviews should focus on correctness, security, maintainability, test coverage,
and public API compatibility. Feedback should be specific and actionable.

After approval, maintainers may squash merge to keep the public history clean.

## Release Notes

For user-visible changes, update [CHANGELOG.md](CHANGELOG.md) under
`Unreleased`. Include security fixes, dependency policy changes, breaking
changes, and migration notes.
