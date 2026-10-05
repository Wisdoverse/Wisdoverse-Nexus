# Development guide

Run commands from the repository root unless a command specifies another directory.

## Setup and paths

- Use the Rust toolchain in [rust-toolchain.toml](../rust-toolchain.toml).
- Use the Node and pnpm versions in [package.json](../package.json).
- Read workspace membership and exclusions from [Cargo.toml](../Cargo.toml).
- Read package dependencies and scripts from the relevant manifest.
- Install Node dependencies with `pnpm install --frozen-lockfile --ignore-scripts`.
- Review dependency overrides and Expo compatibility before version changes.
- Include the affected lockfiles with dependency changes.

| Area | Paths |
| --- | --- |
| Rust services and libraries | `crates/nexis-*` |
| Web and mobile clients | `apps/web`, `apps/mobile` |
| Client SDKs | `sdk/typescript`, `sdk/python` |
| Tests | `tests`, crate test modules, colocated client tests |
| Separate plugin builds | `plugins/*` |
| Contracts and examples | `protocol`, `SPEC.md`, `examples` |
| Documentation | `docs`, `README.md`, `CONTRIBUTING.md` |
| Deployment and operations | `deploy`, `docker`, `k8s`, `ops`, `scripts` |

## Code and architecture

- Apply Rust formatting from [rustfmt.toml](../rustfmt.toml).
- Keep `unsafe_code = "forbid"` and the configured workspace lint levels.
- Fix lint findings, or give a specific reason for a scoped allowance.
- Use the `nexis-*` convention for crate names.
- Keep TypeScript strict checks enabled.
- Use PascalCase for React components and camelCase for functions and variables.
- Keep component CSS modules with their components.
- Apply [ADR-008](../docs/en/architecture/adr/008-fsd-ddd-cloud-native-services.md) to architecture changes.
- Keep frontend imports downward through explicit slice public APIs.
- Do not import sideways between frontend features.
- Keep domain code independent of transport, storage, and external providers.
- Update the relevant ADR, contracts, and developer or deployment documentation when architecture changes.
- Record implementation gaps and migration plans.
- Do not treat static architecture checks as proof of runtime or production readiness.

## Configuration and public material

Use [.env.example](../.env.example) or [deploy/.env.example](../deploy/.env.example) for the selected profile.
Set `JWT_SECRET` explicitly for externally issued tokens.
The gateway requires a nonblank `JWT_SECRET` when `NEXIS_ENV=production`.
Development can use an ephemeral signing key.
Externally issued tokens require a matching key.
The `NEXIS_JWT_SECRET` name in the deployment example does not replace `JWT_SECRET`.

Configure `NEXIS_DEFAULT_PROVIDER` and `RUST_LOG` for the selected profile.
The default provider is `mock`.
The gateway does not currently read `NEXIS_DATABASE_PATH`.
Do not use that variable as evidence of durable room or message storage.

Use the [public privacy checklist](../CONTRIBUTING.md#public-material-and-privacy) before publication.
Read [.pre-commit-config.yaml](../.pre-commit-config.yaml) for configured hooks.
Hook configuration does not prove that local hooks are installed or that checks passed.

## Verification

Run the checks for each changed area.
Before a push, run the strict Clippy command below.
For behavior changes, add regression tests for the affected behavior.
Name tests by behavior, such as `rejects_invalid_tenant_token`.
Do not add implementation-copying tests for documentation or formatting changes.

| Changed area | Commands |
| --- | --- |
| Rust | `cargo fmt --all -- --check`; `cargo check --locked --workspace`; `cargo test --locked --workspace` |
| Strict lint gate | `cargo clippy --locked --workspace --all-targets -- -D warnings` |
| Frontend or gateway boundaries | `pnpm test:architecture`; `pnpm check:architecture` |
| TypeScript SDK | `pnpm --filter @wisdoverse/nexus-sdk build` |
| Web | `pnpm --filter @wisdoverse/nexus-web build`; `pnpm --filter @wisdoverse/nexus-web exec vitest run` |
| Mobile | `pnpm --filter @wisdoverse/nexus-mobile typecheck`; `pnpm --filter @wisdoverse/nexus-mobile test` |
| Mobile dependencies | `pnpm --filter @wisdoverse/nexus-mobile exec expo install --check` |
| Documentation | `pnpm --dir docs docs:build`; `git diff --check` |
| M1 deployment assets | `python3 scripts/check_m1_deployment.py` |
| Separate plugin | `cargo build --manifest-path plugins/<name>/Cargo.toml` |

Replace `<name>` with the changed plugin directory.
Workspace Cargo commands do not include the excluded plugins.
Mobile tests use Vitest.
Web browser tests are in `apps/web/e2e/tests`.
Use the affected package's documented checks for Python SDK changes.

The deployment check requires Docker and PyYAML 6.0.3.
It checks rendered configuration, not cluster behavior.
For gateway or client contract changes, use the live acceptance commands:

```bash
python3 -m pip install -r tests/smoke/m2_requirements.txt
bash scripts/m1_smoke.sh
bash scripts/m2_smoke.sh
```

Read [M1 acceptance](../docs/en/guides/m1-acceptance.md) and the [room agent guide](../docs/en/guides/room-agent.md) for prerequisites and scope.
Record real-provider and human-pilot evidence separately from synthetic acceptance results.

For coverage checks, read `scripts/coverage.sh` and [.github/workflows/ci.yml](../.github/workflows/ci.yml).
The local script requires `grcov` and defaults to 80%; CI currently requires 60%.
For legacy E2E checks, pass `BASE_URL` and `TOKEN` as the first and second arguments to `scripts/e2e_test.sh`.
The script defaults to port `8081`; the local Compose gateway uses port `8080`.

For local development, use `pnpm --filter @wisdoverse/nexus-web dev` or `docker compose up`.
Use the selected profile's documented configuration before startup.

## Review evidence

- Include actual commands and results in the PR.
- For UI, CLI, or deployment changes, include relevant screenshots or redacted logs.
- State which checks are blocked, skipped, or incomplete.
- Keep planned targets separate from measured results.
- Require passing CI before merge, as specified in [CONTRIBUTING.md](../CONTRIBUTING.md).
