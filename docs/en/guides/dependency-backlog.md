# Dependency backlog disposition

This page records how the 16 open dependency update PRs were handled while
aligning updates with the versions and runtime constraints already on `main`.
All 16 source PRs are closed: five because `main` already contains a higher
compatible version, and eleven replaced by the compatible grouped update in
[PR #111](https://github.com/Wisdoverse/Wisdoverse-Nexus/pull/111).
The recorded runtime head `98c8ff6172ba58bdb90db6dde1b5172300e7da52` passed
[CI](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851793249),
[security](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851793412),
[coverage](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851793298), and
[benchmark](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851793262)
on 2026-10-01. Human review and merge remain pending. Later documentation edits
do not change this historical evidence; behavior changes require fresh checks.

| Source PR | Disposition |
| --- | --- |
| #107, PostCSS for web | Closed: the web workspace uses the higher PostCSS 8.5.28 floor. |
| #106, PostCSS for docs | Included in the grouped update with the PostCSS 8.5.28 floor. |
| #105, web dependency group | Included with compatible web updates. Existing higher axios 1.20, Vite 8.3.1, Vitest 4.1.11, and PostCSS 8.5.28 are retained. |
| #103, `quinn-proto` | Closed: `main` already has a higher version. |
| #102, 21 Rust dependencies | Included in the grouped Rust update. |
| #101, axios for web | Closed: `main` already has a higher axios version. |
| #99, Rust Docker image | Included; the Rust builder image is updated to `1.97-slim-bookworm` to match the Debian 12 runtime. |
| #98, Rust Docker image in `/docker` | Included with the Docker image update. |
| #97, `cmov` | Included at 0.5.4. |
| #94, mobile dependency group | Included with Expo 55-compatible navigation, Zustand, Node types, and the `@expo/dom-webview` peer. The proposed React Native 0.86 and React upgrade are deferred because they do not fit the current Expo 55 compatibility matrix. |
| #91, GitHub Actions group | Included with eight action updates. |
| #90, Vite | Closed: `main` already has Vite 8.3.1, above the proposed 8.0.16. |
| #83, axios for mobile | Closed: `main` already has axios 1.20, above the proposed 1.16.0. |
| #80, TypeScript SDK group | Included for the Node type update; the higher axios and `ws` versions already on `main` are retained. |
| #69, docs dependency group | Included for Vue and `@vitejs/plugin-vue`. |
| #66, Playwright | Included with the `@playwright/test` 1.60.0 floor. |

The source PRs were superseded on 2026-10-01. PR #111 contains the reviewable
replacement and tracks final CI and acceptance; source closure does not mean
that the grouped update is already merged.

## Reproduce the verification

```bash
pnpm install --frozen-lockfile --ignore-scripts
cargo clippy --locked --workspace --all-targets -- -D warnings
cargo test --locked --workspace
pnpm --filter @wisdoverse/nexus-web exec vitest run
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-sdk build
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --filter @wisdoverse/nexus-mobile exec expo install --check
pnpm --dir docs docs:build
pnpm audit --audit-level moderate
npm --prefix apps/mobile audit --audit-level=moderate
npm --prefix docs audit --audit-level=moderate
npm --prefix apps/web/e2e audit --audit-level=moderate
```

Record the current commit, toolchain and actual outputs when rerunning. Lockfile
installation and audits validate package resolution; native-device UX and browser
execution need their own tests. Preserve Expo 55 compatibility until an explicit
Expo/React Native migration is qualified.

## Local verification evidence

The following local checks completed successfully:

- Rust workspace: 380 tests passed; strict all-target Clippy passed.
- TypeScript SDK, web app, and docs builds passed.
- Mobile typecheck passed; all 6 mobile tests passed.
- All 53 web tests passed.
- Frozen pnpm installation passed; Playwright discovered its browser test successfully.
- `pnpm audit --audit-level moderate` and the npm audits for mobile, docs, and
  web E2E each reported zero vulnerabilities.

The Expo online compatibility check could not reach its remote metadata
through the environment proxy. Its offline-mode output said dependency
validation was unreliable, so that output is not counted as a compatibility
pass. The online compatibility check passed in the linked public CI run.

Docker image changes use the Rust 1.97 slim Bookworm builder to match the
Debian 12 runtime, retain `cargo --locked`, and include the root health-check
`curl` dependency fix. The gateway image built successfully with Rust 1.97 on Bookworm,
using a temporary CA trust mount for this environment. A non-root container
started successfully and returned HTTP 200 / `OK` from `/health`.

## Integration and remaining gates

A local preview combining PR #111 with M1 head
`e98e9c1c4d769dcc2a142d36b89ff239e2332815` passed 21 real-gateway scenario
groups, 56 Web tests and mobile typecheck. The recorded local merge revision is
`ad0b1a98bee51e43f101bae429011c1fbf569ebd`. Integrating both branches requires
retaining both CHANGELOG entry sets and keeping new CI checkout steps on v7.

React Native 0.86 remains deferred pending an Expo migration with compatibility
and native-device checks. Source PR closure records disposition, not acceptance
or publication. Human review, merge and release/artifact verification remain
separate gates.

## Compiler qualification

CI now selects Rust 1.98.1 explicitly, matching the compiler used for local
qualification. A later floating-stable M1 run selected Rust 1.99 and rejected
`async_trait`-generated `must_use` attributes under the strict lint gate.
Compiler adoption requires fresh qualification; the workspace lint gate stays
strict. Docker builders retain the separately validated Rust 1.97/Bookworm
profile. New workflow links for this configuration revision are recorded in PR #111.
