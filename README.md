# Wisdoverse Nexus

Wisdoverse Nexus is source-available AI-native collaboration infrastructure: a
Rust-first gateway, shared protocol crates, web/mobile apps, SDKs, and deployment
assets for real-time rooms, identity, WebSocket messaging, and AI-assisted
workflows.

The published project documentation is available at
<https://wisdoverse.github.io/Wisdoverse-Nexus/>.

> [!WARNING]
> Wisdoverse Nexus is a pre-1.0 engineering preview for trusted development,
> evaluation, and non-production environments. APIs, SDKs, deployment manifests,
> and crate boundaries may change before the first stable release.

## Running Wisdoverse Nexus

### Requirements

Wisdoverse Nexus works best in repositories where Rust, Node, Expo, Docker, and
documentation checks are reproducible from a fresh checkout.

- Rust stable
- Node.js `>=20.19.0`
- pnpm `>=10.30.0`
- Docker or Docker Compose for containerized local runs

### Option 1. Continue from the spec

Tell your coding agent to inspect the current repository and continue from the
project spec:

> Review Wisdoverse Nexus according to the following spec:
> <https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SPEC.md>

### Option 2. Use the reference implementation

Run the Rust gateway from source:

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

cargo check --workspace
cargo run -p nexis-gateway
```

Validate the gateway from another terminal:

```bash
curl http://localhost:8080/health
# OK
```

You can also run the local container stack:

```bash
docker compose up -d
curl http://localhost:8080/health
```

### Option 3. Verify the workspace

Run the checks that match your change. For broad repository work:

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

---

## Documentation

- Spec: [SPEC.md](SPEC.md)
- Published docs: <https://wisdoverse.github.io/Wisdoverse-Nexus/>
- Quick start: [docs/en/getting-started/quick-start.md](docs/en/getting-started/quick-start.md)
- Development: [docs/en/getting-started/development-guide.md](docs/en/getting-started/development-guide.md)
- Architecture: [docs/en/architecture.md](docs/en/architecture.md)
- Contributing: [CONTRIBUTING.md](CONTRIBUTING.md)
- Security: [SECURITY.md](SECURITY.md)

## License

Wisdoverse Nexus is licensed under the [Wisdoverse Nexus Business Source License
1.1](LICENSE). The license permits personal learning, research, education,
development and testing, internal evaluation, and non-production use without a
commercial license.

Commercial production use, SaaS or hosted service offerings, managed service
offerings, resale, sublicensing, and use to build competing products require a
separate commercial license from Wisdoverse.

Each published version automatically converts to the Apache License 2.0 four
years after that version is first made publicly available by Wisdoverse.
