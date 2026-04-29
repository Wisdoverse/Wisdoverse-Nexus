# Release Process

Wisdoverse Nexus is pre-1.0. Releases should favor reproducibility,
traceability, and clear rollback instructions over broad compatibility promises.

## Versioning

Use semantic-looking tags:

```text
v0.1.0
v0.1.1
v0.2.0
```

Before `v1.0.0`, minor versions may still include breaking changes. Breaking
changes must be documented in `CHANGELOG.md` and the GitHub release notes.

## Release Artifacts

The current `release.yml` workflow is tag-triggered and is expected to:

| Artifact | Source |
| --- | --- |
| Gateway container image | `docker/Dockerfile.gateway`, pushed to GHCR |
| TypeScript SDK package | `sdk/typescript`, published to npm |
| GitHub release | Generated from the tag and commit history |

The private web app package is not published as a public npm package.

## Pre-Release Checklist

Run or verify the broad local gate:

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo check --workspace
cargo test --workspace
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --dir docs docs:build
```

Also verify:

- `CHANGELOG.md` has an entry for the release.
- `Cargo.lock`, `pnpm-lock.yaml`, and package-local lockfiles are committed when changed.
- The GitHub Actions CI workflow is green on the release commit.
- Security-relevant dependency updates include audit evidence.
- Breaking changes include migration notes.

## Tagging

Create releases from a clean `main` commit:

```bash
git checkout main
git pull --ff-only
git status --short
git tag -a v0.1.0 -m "Release v0.1.0"
git push origin v0.1.0
```

The tag starts the release workflow.

## Post-Release Verification

After the workflow completes:

- Confirm the GitHub Release exists.
- Confirm the GHCR image tag exists.
- Confirm the TypeScript SDK version is visible on npm when publishing is enabled.
- Review generated release notes for sensitive or misleading content.
- Open a follow-up issue for any release gaps found during verification.

## Rollback

For a broken container release, re-deploy the previous known-good image tag in
your environment. For a broken npm package, deprecate the affected version and
publish a patch:

```bash
npm deprecate @wisdoverse/nexus-sdk@0.1.0 "Use 0.1.1 or newer"
```

Security issues should use the private advisory flow documented in
[SECURITY.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/SECURITY.md).

## Related Documents

- [Testing Guide](testing.md)
- [API Versioning](api-versioning.md)
- [Deployment](../operations/deployment.md)
