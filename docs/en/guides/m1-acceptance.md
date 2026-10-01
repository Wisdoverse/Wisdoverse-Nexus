# M1 implementation and acceptance evidence

Status as of 2026-10-01: **implementation and automated checks passed; merged at `dc40ae6bb53f9ad7b88b91122113c24ecbc88646`; maintainer acceptance ownership remains to be recorded** in [PR #110](https://github.com/Wisdoverse/Wisdoverse-Nexus/pull/110).
Scope: RD-001–RD-004 / QG-01. The supported profile is one gateway process,
default features, single tenant, and in-memory room/message state.

## Initial revision and public evidence

| Evidence field | Recorded value |
| --- | --- |
| Base revision | `e47f55bd70d413098fc6de9ed19fe285232b980e` |
| Tested PR head | `e98e9c1c4d769dcc2a142d36b89ff239e2332815` |
| CI checkout recorded by live runner | `9d1e8a4a52352b3b01f3b565fd991b74976f59df` (GitHub-generated PR merge revision) |
| CI | [36851322165](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851322165): passed, including live gateway and browser paths |
| Security | [36851322042](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851322042): passed |
| Coverage | [CI coverage job](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851322165/job/110334386732): 67.50% workspace lines (9,052/13,410), above the 60% floor; [coverage workflow](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851322090) also passed |
| Benchmark | [36851322075](https://github.com/Wisdoverse/Wisdoverse-Nexus/actions/runs/36851322075): passed existing benchmark job |
| Contract artifact | `m1-live-gateway-contracts`, ID `11156225724`, in the linked CI run; retained for 30 days |
| Human DRI / acceptance reviewer | Assignment and acceptance remain to be recorded by the maintainer |

## Review completion and architecture gates

The follow-up integrates dependency commit `d915f56d38205dce16230d153b45e9bbda724b5a`
from [PR #111](https://github.com/Wisdoverse/Wisdoverse-Nexus/pull/111), preserving
both changelog sets and using checkout v7 in all M1 CI jobs. The final review
revision and its public workflow links are recorded in PR #110; the table above
preserves the initial implementation's historical evidence.

Review corrections require production JWT configuration before binding a
listener, restore and validate mobile saved sessions before protected navigation,
and enforce frontend public APIs/layer direction. Mobile session and Web room
state live in entities; navigation contracts live in shared. The architecture
gate checks 98 frontend modules and four gateway domain modules, with four
positive/negative checker cases. Its static scope does not prove full architecture
or independent service deployment.

The single-instance deployment gate verifies Helm lint/render, existing Secret
and image digest references, non-root execution, resource budgets and all three
health probes. Missing Secret names, multiple replicas and HPA must be rejected.
`Recreate` upgrades preserve the one-process scope but lose in-memory state and
have downtime. Isolated process tests verify rejected missing/blank production
keys and HTTP health probes with HTTPS redirect; a server test verifies that
graceful HTTP shutdown waits for an accepted response. Cluster operation,
WebSocket draining and durable recovery remain unqualified.

Local follow-up checks passed: 391 Rust tests including documentation tests
(one stress test remains ignored), strict workspace/all-target Clippy, 14 mobile
tests and typecheck, 56 Web tests and build, SDK build, 21 live-gateway scenario
groups, architecture/deployment checks and the VitePress build. Fresh public CI
for each runtime revision remains required and is linked from the PR. The live
runner records its checkout and working-tree diff digest; a successful earlier
revision does not qualify later behavior changes.

The head revision identifies the proposed changes; the runner records the
actual checkout tested by GitHub. Later documentation commits do not change
this historical evidence. A behavior change requires a fresh report for its
own revision. Workspace coverage and benchmark success do not demonstrate the
planned 80% changed-line target or the QP-1 load/soak and operational objectives.

## Work-package traceability

The IDs and responsibilities follow the [execution standard](roadmap-execution.md).

| Work item | Implemented outcome | Executable evidence |
| --- | --- | --- |
| RD-001: protocol/gateway contracts | Verified external JWT session, room membership/administration, shared HTTP/WS contracts and served OpenAPI schemas; rejected and expired credentials and unsupported tenant scope | HTTP/WS negative cases and served-schema assertions in `tests/smoke/m1_gateway.py` |
| RD-002: client/SDK workflow | Web, mobile, TypeScript and Python authenticate, create/list a room, send/receive and recover history through the same gateway | `apps/web/smoke/collaboration.test.ts`, `apps/mobile/smoke/collaboration.test.ts`, `sdk/typescript/tests/live_gateway.cjs`, Python SDK checks in the live runner |
| RD-003: message safety/recovery | Stable sender/message/reply references, ordered writes/history, scoped one-hour retry deduplication, conflict responses and reconnect recovery | Concurrent HTTP and WS retry/ack/history cases; application tests for membership, failed writes, ledger capacity/expiry and cross-room replies |
| RD-004: live CI | Fresh-checkout isolated gateway, synthetic credentials/data, bounded waits, cleanup and sanitized retained results | `scripts/m1_smoke.sh`, `M1 Live Gateway Contracts` CI job, JSON summary and logs |

## Initial observed results and profile

Local qualification used Linux x86_64, Rust 1.98.1, Node 24.19.0,
pnpm 10.30.3 and Python 3.12.14. The live runner builds a locked development
binary and starts an ephemeral loopback gateway with synthetic users/rooms.
CI toolchain selection is defined in the workflow: Ubuntu runner, pinned Rust 1.98.1,
Node 24, Python 3.12 and tracked dependency locks.

| Check | Observed result | Interpretation |
| --- | --- | --- |
| Real gateway | 21/21 scenario groups passed locally and in public CI | Four clients; expiry/access denials; concurrent retries; actual SDK reconnect; acknowledgements and ordered history |
| Browser user paths | 2/2 Chromium tests passed in public CI | Accepted/rejected token login and room-list rendering; synthetic HTTP fixtures |
| Web | 56 unit tests and production build passed | Client/store regression and compilation checks |
| Mobile | 6 unit tests and typecheck passed | Adapter/store checks; native-device UI is outside this result |
| Rust | Workspace tests, strict all-target Clippy and formatting passed | Gateway failure/concurrency regressions included; experimental multi-tenant/persistence feature combination compiles |
| SDK/docs | TypeScript SDK and VitePress builds passed | Published contracts/examples and internal links remain buildable |
| Expo | Online compatibility gate passed in public CI | Local proxy blocked online metadata; offline metadata was not counted as a pass |

The local combined preview of M1 with [dependency PR #111](https://github.com/Wisdoverse/Wisdoverse-Nexus/pull/111)
(head `98c8ff6172ba58bdb90db6dde1b5172300e7da52`) passed the same 21 live groups,
56 Web tests and mobile typecheck. Its recorded local merge revision is
`ad0b1a98bee51e43f101bae429011c1fbf569ebd`; this is local integration evidence,
separate from the public per-PR checks.

## Reproduce and retain evidence

Use the [fresh-checkout setup](core-collaboration.md#m1-acceptance), then:

```bash
NEXIS_SMOKE_PYTHON=<prepared-python> bash scripts/m1_smoke.sh
cargo fmt --all -- --check
cargo check --locked --workspace
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets -- -D warnings
pnpm --filter @wisdoverse/nexus-web exec vitest run
pnpm --filter @wisdoverse/nexus-web build
pnpm --filter @wisdoverse/nexus-mobile typecheck
pnpm --filter @wisdoverse/nexus-mobile test
pnpm --dir docs docs:build
```

For browser checks, use the [testing guide](testing.md#web-browser-paths).
`artifacts/m1/summary.json` records the actual checkout, working-tree fingerprint,
configuration, sample count and each result. Download the linked run's artifact
before its 30-day retention expires; retain the original ZIP/summary and checksum
with release evidence when acceptance or publication is recorded. Synthetic
credentials and private deployment addresses must remain absent from public logs.

## Limits and remaining acceptance

History recovery lasts for the current gateway process; restarting clears the
store. Retry keys last 60 minutes, up to 10,000 active keys, and do not survive
restart. Multi-tenant core routes fail closed with 503; tenant-scoped tokens are
rejected in the supported single-tenant mode. Broader tenancy, durable recovery,
load/soak and rollout qualification belong to later milestone evidence.

[ADR-008](../architecture/adr/008-fsd-ddd-cloud-native-services.md) requires FSD,
DDD and cloud-native service evolution. These tests do not certify every import
boundary, independent service deployment, native-device UX or production scaling.
Before marking M1 complete, record the responsible human reviewer and acceptance
against the pinned evidence, then merge and apply the relevant release gates.
M2–M5 remain planned.
