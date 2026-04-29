# ADR-007: Phase 6 CI Quality Gates for Performance and Coverage

## Status

Accepted

## Context

Phase 6 increased delivery volume and introduced broader collaboration features.
To keep quality measurable, CI needed stronger automatic quality gates for:
- Performance regressions in benchmarks
- Coverage visibility beyond pass/fail threshold checks

## Decision

Strengthen CI/CD quality controls by:
- Enforcing benchmark regression alerts at `120%` (fail on >20% regression)
- Publishing coverage summaries and coverage artifacts alongside threshold enforcement

## Consequences

### Positive
- Regressions are detected and blocked earlier in PR flow
- Coverage trends are easier to inspect from CI artifacts and job summaries
- Quality expectations are explicit and auditable

### Negative
- More CI failures require active triage from contributors
- Benchmark variability can produce noisy failures on unstable workloads

### Mitigation
- Keep benchmark scenarios deterministic where possible
- Re-baseline intentionally when validated performance trade-offs are accepted
