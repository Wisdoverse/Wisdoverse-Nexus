# Dependency backlog disposition

This page records how the 16 open dependency update PRs were handled while
aligning updates with the versions and runtime constraints already on `main`.
Five single-package PRs were closed because `main` already contains a higher
compatible version. The remaining updates are collected in a new grouped PR;
its public CI and final review remain pending.

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

The grouped dependency PR link and its review status will be added after the
integration owner supplies them. Public CI status is still pending.

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
pass. Public CI remains to be checked on the grouped PR.

Docker image changes use the Rust 1.97 slim Bookworm builder to match the
Debian 12 runtime, retain `cargo --locked`, and include the root health-check
`curl` dependency fix. The gateway image built successfully with Rust 1.97 on Bookworm,
using a temporary CA trust mount for this environment. A non-root container
started successfully and returned HTTP 200 / `OK` from `/health`.
