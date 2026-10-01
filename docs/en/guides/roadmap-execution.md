# Roadmap Execution Standard

This guide makes the [roadmap](../roadmap.md) actionable through scoped work,
accountability, measurable gates, and reproducible evidence. It applies to the
planned M1–M5 capabilities within the project's documented evaluation scope.
The [Chinese guide](../../zh-CN/guides/roadmap-execution.md) tracks the same rules.

## Baseline and Proposed Gates

The targets below are **proposed Nexus targets awaiting qualification**.
Observed values, owners, and completion evidence remain unassigned until
recorded in a work item. Public engineering references inform the process;
these numbers are project choices and are not a company certification or SLA.

| Area | Current repository check | Planned addition |
| --- | --- | --- |
| Build and correctness | Rust format/Clippy/workspace tests, mobile checks, Web/SDK/docs builds, M1 live-gateway and browser jobs | Extend qualified contracts and profiles as supported behavior grows |
| Coverage | Workspace line coverage floor is 60% in CI | Retain that floor; add at least 80% coverage of changed executable lines in auth, tenant, tool/approval, and persistence paths, plus mandatory behavior tests (RD-004, RD-008, RD-012) |
| Security | Dependency/advisory, license, secret, CodeQL, and container checks | Permission-revocation, context leakage, prompt-injection, and side-effect approval tests (RD-005, RD-007, RD-010, RD-015) |
| Performance | Benchmark entry points and generic gateway metrics exist | Versioned end-to-end load profile, SLI instrumentation, budgets, and regression gates (RD-008, RD-012) |
| Release | Tag-triggered container/SDK workflows, container SBOM/provenance configuration | Verify published artifacts, qualification evidence, upgrade/recovery, and staged enablement (RD-011, RD-012, RD-016, RD-020) |

New gates become enforced only after their measurement and CI work lands.
Coverage does not replace observable behavior checks. Confirm every required
scenario has an executable test target in CI; a test file's presence is
insufficient evidence of execution. Preserve the current checks while adding
new ones. Record reviewed advisory exceptions with a responsible role, reason,
mitigation, and expiry or upstream re-evaluation condition.

## Execution Packages

RD-001–RD-004 implementation is **merged**, with passed automated evidence in
[PR #110](https://github.com/Wisdoverse/Wisdoverse-Nexus/pull/110) and the
[M1 report](m1-acceptance.md). RD-005–RD-008 have implementation and local synthetic evidence; real-provider/reviewer acceptance is pending. RD-009–RD-020 remain **planned**. Human DRI and
acceptance-reviewer assignments remain unrecorded; responsibility names below
are proposed roles, not completed governance assignments. A maintainer assigns one publicly identified directly responsible
individual (DRI) and an appropriate reviewer before a package becomes Ready.
These are local planning IDs, not existing GitHub issue numbers. Split packages
into small, independently reviewable PRs and link them from the work item.

| ID | Milestone | Primary responsibility | Dependency | Required exit evidence |
| --- | --- | --- | --- | --- |
| RD-001 | M1 | Protocol / Gateway | None | Auth, room membership, HTTP/WS error and payload contracts; supported-mode matrix |
| RD-002 | M1 | Clients / SDKs | RD-001 | One room/message workflow exercised by Web, mobile, TypeScript, and Python |
| RD-003 | M1 | Gateway | RD-001 | Message IDs, replay window, ordering, retry/idempotency, and reconnect failure tests |
| RD-004 | M1 | Test infrastructure | RD-002, RD-003 | Fresh-checkout live-gateway CI with discoverable test targets and contract fixtures |
| RD-005 | M2 | AI / Security | RD-004 | Context selection and revocation tests that reject cross-room/tenant access |
| RD-006 | M2 | AI / Clients | RD-005 | Named agent run with progress, streaming, sources, cancel, and terminal states |
| RD-007 | M2 | Tools / Security | RD-005, RD-006 | One read-only MCP tool, allowlist, authorization at execution time, and failure cases |
| RD-008 | M2 | AI evaluation | RD-006, RD-007 | Versioned evaluation set, provider smoke evidence, hard run budgets, and usage reporting |
| RD-009 | M3 | Storage | RD-004, RD-006 | Versioned persistence schema and crash/restart tests for acknowledged state |
| RD-010 | M3 | Security / Storage | RD-005, RD-009 | Tenant isolation and retention/export/deletion across enabled stores and indexes |
| RD-011 | M3 | Platform | RD-009, RD-010 | Backup/restore and upgrade/migration rehearsal with measured recovery times |
| RD-012 | M3 | Reliability | RD-008, RD-010, RD-011 | Load/soak report, resource limits, SLI dashboard/alerts, incident and recovery runbooks |
| RD-020 | M3 | Release engineering | RD-011, RD-012 | Published container/SDK fresh-install checks, digest/provenance verification, and release gate report |
| RD-013 | M4 | Collaboration / AI | RD-008, RD-012 | Drafts and retrieval with source references and shared permission checks |
| RD-014 | M4 | Documents | RD-012 | Two-member CRDT convergence, conflict, unauthorized-edit, and durable revision tests |
| RD-015 | M4 | Tasks / Security | RD-013, RD-014 | Human-approved task/document writes, rejection, revocation, and idempotency evidence |
| RD-016 | M4 | Clients / Test infrastructure | RD-013, RD-014, RD-015 | Complete room-to-draft-to-approved-task-to-room-status scenario and pilot report |
| RD-017 | M5 | Extensions | RD-008, RD-016 | Versioned MCP/skills/plugin contracts, capability scopes, credential/failure isolation |
| RD-018 | M5 | Agent interoperability | RD-017 | A2A version decision and identity/discovery/task/cancel/artifact mapping |
| RD-019 | M5 | Interoperability testing | RD-018 | One external implementation passing supported transport/auth/timeout conformance cases |

The critical path to the first product evaluation is RD-001 → RD-002/RD-003
→ RD-004 → RD-005 → RD-006/RD-007 → RD-008. Storage design can start earlier;
expanded deployment and M4 remain gated by M3 evidence. Scope and effort
estimates are recorded after dependencies and capacity are understood. Delivery
dates require an assigned owner and reviewed estimate.

## Work Item Lifecycle

Use the [roadmap work item template](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/.github/ISSUE_TEMPLATE/roadmap_work_item.md)
to record Planned → Ready → In progress → In review → Validated → Released.
A Blocked item records its dependency or failing gate and next action. Validated
means the specified acceptance checks passed on an identified commit; Released
also requires verified published artifacts and the applicable rollout checks.

Before Ready, record:

- The user/operator outcome, in-scope behavior, non-goals, dependency IDs, DRI,
  reviewer, and affected DDD/FSD boundaries.
- Contract examples and design decisions for identity, schema/protocol changes,
  data flow, permissions, compatibility, and migration where relevant.
- A reproducible verification plan with positive, negative, failure, and recovery
  cases; exact profile, target, measurement window, and expected result.
- Resource/token/cost budgets, operator risks, and rollback or disable strategy
  for changes that affect running services or durable state.

Before Validated, record:

- The reviewed PR and commit, green applicable required checks, and independently
  reproducible behavior evidence. Scale verification to the change: documentation
  uses build/link checks; behavior changes need appropriate regression tests.
- Updated contracts, client/SDK examples, changelog, and migration notes.
- Operational signals, failure handling, and rehearsal evidence appropriate to
  the affected runtime/data path.
- Observed gate values and unresolved risks. Target changes require a versioned
  rationale and fresh evaluation; retain the earlier failure evidence.

## Qualification Profile QP-1

QP-1 targets the first supported single-node evaluation path. Before running,
pin the commit, release build, feature flags, persistence backend, dataset,
client versions, commands, and host/storage/network details. Use a dedicated
gateway allocation of **2 vCPU and 4 GiB RAM**, documenting separate database
resources if used. Profiles for other hardware or multi-node operation require
their own targets and evidence.

Use **100 WebSocket clients across 10 rooms**, **50 messages/second aggregate**
with **1 KiB payloads**, sent through authenticated HTTP message writes, plus
**20 HTTP room/message reads/second**: **70 HTTP requests/second total
(50 writes / 20 reads)**. Measure client-observed latency on the
declared local network; separate external model/tool latency. Run a 5-minute
warm-up followed by 30 minutes of measurement, repeated three times. Each run
must meet the gate. Publish sample counts, errors, p50/p95/p99, peak RSS, CPU,
and configuration. Report success-response latency and failed/timed-out
requests together; failures count against QG-03. Overload beyond this profile must fail safely with bounded
queues and explicit backpressure.

| Gate | Proposed target | Required evidence / stage |
| --- | --- | --- |
| QG-01: Contract correctness | 100% of mandatory positive/negative scenarios pass | Live-gateway client/SDK report; M1 onward |
| QG-02: Transport latency | HTTP p95 ≤200 ms and p99 ≤500 ms; WS send-to-recipient p95 ≤500 ms | QP-1 client-side samples, including failure counts; M3 |
| QG-03: Gateway success | ≥99.9% of eligible HTTP requests return the expected successful response within 2 seconds | QP-1 report and SLI definition below; M3 |
| QG-04: State safety | Zero lost acknowledged writes after process crash/restart; zero duplicate effects for the same retry key within the declared window | Fault-injection report and durable-ack contract; M3/M4 |
| QG-05: Resource stability | Gateway peak RSS ≤2 GiB; final-six-hour median RSS ≤110% of first-six-hour median in a 24-hour bounded-working-set soak after warm-up | QP-1 limits plus declared retention/working set, memory series, and queue limits; M3 |
| QG-06: Recovery | Backup RPO ≤24 hours and restore RTO ≤30 minutes | Simulated storage-loss restore of 10 rooms/10,000 messages and enabled artifacts, through client smoke verification; M3/M4 |
| QG-07: Authorization | Zero cross-room/tenant disclosures, unauthorized tool calls, or unapproved side effects in mandatory cases | Negative, revocation, and adversarial scenario results; M2 onward |

QG-04 concerns writes acknowledged before a process crash with the durable
storage intact. QG-06 concerns storage loss and restoration from the scheduled
backup; record the backup's consistency boundary and actual artifact counts.
One successful drill or soak does not establish a longer-term availability claim.

### SLI and Error Budget Policy

Gateway SLI = good eligible requests / all eligible requests. Eligible requests
are valid authenticated calls to supported room/message endpoints within the
declared profile. A good request returns its expected success response within
2 seconds. Count server errors, dependency failures, timeouts, and unexpected
throttling as bad events. Report expected authorization denials and malformed
requests separately, with their own safety assertions. Report external AI/tool
success and latency separately, including provider failures and cancellations.

The proposed operational objective is **99.9% over a rolling 30-day window**;
the allowed error budget is **0.1% of eligible requests**. Instrument the
denominator and failure categories in RD-012 before enforcing this policy.
A 30-minute load run or 24-hour soak is qualification evidence for those windows.
The 30-day objective requires actual operator telemetry over that window.

Candidate burn-rate alerts compare bad-event fraction with 0.001: fast alert
at ≥14.4× in both 1-hour and 5-minute windows; slow alert at ≥6× in both 6-hour
and 30-minute windows. Require at least 100 eligible events in each alert
window; report insufficient data otherwise and retain synthetic health checks.
Data loss and permission violations stop rollout regardless of traffic volume.
An exhausted operational error budget prioritizes reliability fixes and pauses
feature rollout until the owner records recovery and a new assessment.

## AI Quality and Product Evaluation

RD-008 versions **50 synthetic cases** with expected behavior and a rubric:

| Case category | Count | Expected result |
| --- | --- | --- |
| Grounded normal room/document questions | 25 | Relevant answer, correct source references, no unsupported action claim |
| Cross-room/tenant access and revoked permissions | 10 | Denial without disclosure or side effect |
| Prompt injection in messages or tool output | 5 | Policy and tool scope preserved |
| Provider/tool timeout, retry, and cancellation | 5 | Correct terminal state, bounded retries, no further execution after cancellation is confirmed |
| Budget exhaustion and required approval | 5 | Hard stop at the limit; writes require the recorded human approval |

At M2, grounded cases use room messages and write requests must be denied by
the read-only scope. At M4, extend the versioned fixtures to authorized
documents/transcripts and approved-write/revocation cases, retaining the
category counts and rerunning the report.

Pin model/provider configuration, prompt/context versions, tool schema, fixture
commit, and scoring rubric. Run three repetitions: at least 90% of normal-case
answers must pass the rubric overall, counting missing/failed normal answers
as failures; every safety/failure case must meet its
specified safe outcome in every repetition. Record disagreements and reviewer
judgments. A permitted failure is observable and safe, not silently successful.
CI uses deterministic providers; real-provider qualification is opt-in with
runtime secrets and separately reported costs and data egress.

Proposed initial per-run limits are **60 seconds**, **4 tool calls**, and
**4,096 output tokens**; record provider/tool timeouts and tenant concurrency
limits before Ready. Server-side enforcement must stop subsequent work when
limits are exhausted or cancellation is confirmed. Record token usage and the
model's dated pricing assumptions; set the monetary budget before a paid pilot.
Any integration needing other limits requires a reviewed profile and rerun.

RD-016 additionally evaluates at least **50 scoped workflow attempts across
five opt-in evaluators**, aiming for **≥90% completed normal workflows**.
Report started, completed, rejected, cancelled, and failed counts together;
include provider failures in the denominator. Publish anonymized outcomes and
limitations rather than individual identities or private content. Check the
critical Web/mobile journey's keyboard/focus behavior, accessible labels and
status announcements, and documented browser/device compatibility.

## Release Gates and Evidence

| Stage | Entry evidence | Expansion / stop condition |
| --- | --- | --- |
| Trusted alpha | M1 and M2 acceptance, QG-01/QG-07, fixed provider/tool scope, and a tested agent disable path | Start with a synthetic room, then one opt-in workspace; any policy violation disables affected agent/tool execution |
| Evaluation beta | M3, one M4 workflow, QP-1/AI/pilot reports, recovery rehearsal, and operator runbooks | Expand within the qualified capacity; observe each staged enablement for at least 24 hours; gate failures pause expansion |
| 1.0 candidate | All supported-path acceptance, compatibility/migration policy, artifact verification, unresolved-risk disposition, and readiness review | Release only the evidenced support matrix; unsupported M5/federation/media paths retain experimental status |

Use staged feature enablement suitable for the qualified single-node path.
Before each expansion, check metrics, outstanding incidents, budgets, and data
integrity. On failure, disable the feature or restore the known-good artifact;
test schema compatibility or restore before using an older binary. Rehearse
rollback on a copy of the qualified dataset and record measured recovery.

The evidence package must include commit/tag, image/package digest and version,
supported configuration, gate IDs, targets and observed values, sample sizes,
CI/test URLs, reviewer/DRI, exception disposition, and rollout/recovery results.
Verify a fresh installation of the published image and SDK. A configured
SBOM/provenance workflow must produce verifiable artifacts before being marked
complete. Keep signed provenance, artifact identity, and source commit linked.

Record gates in a machine-readable report, for example:

```json
{
  "schema_version": 1,
  "work_item": "RD-012",
  "status": "planned",
  "commit_sha": null,
  "profile": "QP-1",
  "gate": "QG-02",
  "target": {"http_p95_ms": 200, "http_p99_ms": 500},
  "observed": null,
  "sample_count": null,
  "evidence_url": null,
  "dri": null,
  "reviewer": null
}
```

Null evidence fields prevent a planned report from being treated as passed.
Scope the report to the tested commit, environment, and configuration.

## Risks and Review Cadence

| Risk | First mitigation / owner role |
| --- | --- |
| HTTP/WS/client/SDK contract drift | Versioned fixtures and live-gateway CI; Protocol / Test infrastructure |
| AI context leakage or unauthorized side effects | Context ACLs, execution-time authorization, approval and disable controls; AI / Security |
| Crash loss, schema migration, or stale indexes | Durable-ack contract, restore rehearsal, lifecycle checks; Storage / Platform |
| Provider outages, latency, or cost escalation | Hard budgets, cancellation, explicit failure states, qualified provider matrix; AI evaluation |
| Scope growth and unsupported deployment claims | Keep the critical path and qualification profiles explicit; roadmap DRI |

Review active work and blockers weekly when implementation starts. Review
targets, pilot feedback, and capacity at each milestone exit; update both
languages. Record changes through scoped PRs. Incident reviews should identify
causes, detection/recovery gaps, and owned follow-ups using redacted evidence.

## Public Engineering References

Reviewed on 2026-10-01:

- [Google code-review standard](https://github.com/google/eng-practices/blob/deaff2f9ff28501df657f37a73ed72afa57cb2e0/review/reviewer/standard.md): engineering facts, maintainability, and continuous improvement.
- [Google small changes guidance](https://github.com/google/eng-practices/blob/fde5fd40e259bb703dec55f17729abe99c17bf73/review/developer/small-cls.md): focused, reviewable changes and easier rollback.
- [Google SLO Generator](https://github.com/google/slo-generator/blob/47f54921314bf7d5197712e728405fc769724b3e/README.md): explicit SLI, SLO, error-budget, and burn-rate definitions. Nexus can implement the measurements with its existing telemetry stack.
