# Contributing

Wisdoverse Nexus is developed as a public GitHub project. The contribution
process is intentionally conservative: small pull requests, reproducible local
commands, accurate documentation, and no public disclosure of vulnerabilities.

For the canonical repository policy, see
[CONTRIBUTING.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/CONTRIBUTING.md).

## Development Requirements

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker or Docker Compose for local services

## First-Time Setup

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

pnpm install --frozen-lockfile --ignore-scripts
cargo check --workspace
```

Run the gateway:

```bash
cargo run -p nexis-gateway
```

Run the web app:

```bash
pnpm --filter @wisdoverse/nexus-web dev
```

Run mobile checks:

```bash
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
```

## Project Structure

| Path | Purpose |
| --- | --- |
| `crates/nexis-protocol` | Identity, message, permission, and tenant primitives |
| `crates/nexis-gateway` | HTTP/WebSocket gateway and request handling |
| `crates/nexis-ai` | AI provider and MCP integration surfaces |
| `crates/nexis-context` | Context window and summarization primitives |
| `crates/nexis-plugin` | Plugin execution model |
| `crates/nexis-meeting`, `nexis-doc`, `nexis-task`, `nexis-calendar` | Collaboration domain crates |
| `apps/web` | React + Vite web application |
| `apps/mobile` | Expo / React Native mobile application |
| `sdk/typescript`, `sdk/python` | Client SDKs |
| `docs` | VitePress documentation site |
| `deploy` | Local and deployment manifests |

## Local Quality Gate

Run targeted checks for narrow changes. For broad changes, run:

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

## Pull Request Standard

Before opening a pull request:

- Keep the scope focused and explain the user impact.
- Link related issues.
- Add or update tests for behavior changes.
- Update documentation when commands, configuration, public APIs, or deployment behavior changes.
- Include the verification commands you ran.
- Call out breaking changes explicitly.

CI must pass before merge. Maintainers may ask for smaller PRs when a diff is
hard to review.

## Documentation Standard

Documentation must describe the current repository, not future intent.

- Do not add unverified benchmark, certification, customer, or compatibility claims.
- Keep English and Chinese entry points aligned when first-run behavior changes.
- Use <https://wisdoverse.com> for the official public documentation site.
- Keep repository links under `https://github.com/Wisdoverse/Wisdoverse-Nexus`.
- Keep runtime names such as `nexis-gateway`, `NEXIS_*`, crate names, and Docker service names accurate.

## Security Reports

Do not report vulnerabilities in public issues or pull requests. Use the private
GitHub Security Advisory flow described in
[SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md).
