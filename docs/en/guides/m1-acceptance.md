# M1 implementation and acceptance evidence

Status: **implementation complete; public CI and reviewer acceptance pending**.
Scope: RD-001–RD-004 / QG-01, based on main `e47f55b` on 2026-10-01.
This is engineering-preview qualification, not a production release, performance
certification, or confirmation of the planned 80% changed-line coverage target.

## Implemented behavior

| Work item | Implementation | Evidence |
| --- | --- | --- |
| RD-001: authentication | Verified external JWT session; invalid/expired/issuer/audience/empty-subject rejection; token expiry closes sockets; tenant-scoped tokens rejected in single-tenant mode | Real HTTP/WS negative cases, auth timeout and expiry cases |
| RD-002: rooms/messages | Creator/member permissions, creator-only administration, shared HTTP/WS writes, stable sender/message/reply identity, ordered history, one-hour scoped retry keys | Cross-room denials, invitations, concurrent retry and conflict cases; failed-write and ledger-capacity unit cases |
| RD-003: contracts/clients | Served OpenAPI response schemas; actual Web/mobile HTTP and WS adapters; Python/TypeScript SDK authentication, events, retry IDs, and reconnect | Served-schema assertions and all four client suites against the same real gateway |
| RD-004: live CI | Isolated synthetic gateway process, deterministic credentials, lifecycle cleanup, bounded waits, sanitized evidence retention | `scripts/m1_smoke.sh`; `M1 Live Gateway Contracts` CI job; JSON summary and client/gateway logs |

## Observed local results

Validated on Linux x86_64, Rust 1.98.1, Node 24.19.0, pnpm 10.30.3,
Python 3.12.14. The live profile is one ephemeral loopback gateway, default
features, single tenant, in-memory storage, synthetic users and rooms.

- Live gateway runner: **21 of 21 scenario groups passed**, including both SDKs,
  Web/mobile adapters, actual forced reconnect, permission denials, expiry,
  concurrent retry deduplication, acknowledgements, and history recovery.
- Rust workspace tests and strict workspace/all-target Clippy passed. Gateway
  tests were rerun after adding concurrency, failed-write, membership, and retry
  capacity/expiry regression cases; the multi-tenant/persistence feature combination compiles.
- Web: **56 unit tests passed** and production build passed. Mobile: **6 unit
  tests passed** and typecheck passed. TypeScript SDK and documentation builds passed.
- Expo's offline metadata check reports up-to-date, but cannot establish online
  compatibility. The online service is blocked by the execution environment proxy;
  the unchanged compatibility gate must run in public CI.

## Reproduce and retain evidence

Follow the [fresh-checkout commands](core-collaboration.md#m1-acceptance), or run
`NEXIS_SMOKE_PYTHON=<prepared-python> bash scripts/m1_smoke.sh` after installing
workspace dependencies. The script builds the locked gateway and SDK, launches
the real gateway, and runs all client suites. No production credentials or
external services are needed.

`artifacts/m1/summary.json` records the tested commit, working-tree fingerprint,
configuration, sample count, and each scenario result. CI uploads this summary
and sanitized logs for 30 days, including on failure. Generated artifacts are
ignored locally; link the public CI run and download before retention expires.

Also run `cargo fmt --all -- --check`, `cargo check --locked --workspace`,
`cargo test --locked --workspace`, and
`cargo clippy --locked --workspace --all-targets -- -D warnings`, plus the
Web/mobile/SDK/docs commands in the repository guidelines.

## Acceptance limits and remaining gates

The recovery window is the current process lifetime for history and 60 minutes
for retry keys, with 10,000 active keys. Explicitly unsupported multi-tenant core
flows fail closed with 503. Persistent crash/restart safety, performance/load
objectives, broader deployments, and AI workflows belong to later milestones.
Native-device and browser UI interaction were not exercised by the transport
suites; the report validates the adapters and unit/build checks.

Before marking M1 complete, attach the public PR and green CI run for its final
commit, assign the responsible reviewer, and record acceptance. No reviewer or
release approval is implied by these local results. Keep M2–M5 planned.
