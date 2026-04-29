# Wisdoverse Nexus Optimization Backlog

## P0 (Blockers)

### P0-001
- **Title:** Restore build disk headroom on development/CI runners
- **Priority:** P0
- **Description:** Ensure Rust builds no longer fail with `No space left on device` by clearing stale build artifacts and documenting a repeatable cleanup step for constrained environments.
- **Acceptance Criteria:**
  - Running `df -h /tmp` shows at least 2 GB free before build.
  - `cargo clean` (or documented equivalent) is added to a reproducible maintenance step in project docs or CI notes.
  - `cargo check --workspace` completes without `os error 28`.
- **Dependencies:** None
- **Estimated effort:** S
- **Assigned to:** 

### P0-002
- **Title:** Move Cargo target output off constrained tmpfs
- **Priority:** P0
- **Description:** Configure build output directory to a larger mount so compile/test/lint runs do not exhaust `/tmp`.
- **Acceptance Criteria:**
  - `CARGO_TARGET_DIR` is set in the relevant shell/CI workflow.
  - A full build run writes artifacts to the configured non-`/tmp` location.
  - `cargo test --workspace` and `cargo clippy --workspace` run without disk exhaustion errors.
- **Dependencies:** P0-001
- **Estimated effort:** S
- **Assigned to:** 

### P0-003
- **Title:** Add `ContextError::WindowFull` and wire overflow fail path
- **Priority:** P0
- **Description:** Add missing `WindowFull` error variant to `ContextError` and ensure overflow behavior using `OverflowStrategy::Fail` returns this variant.
- **Acceptance Criteria:**
  - `crates/nexis-context/src/error.rs` defines `WindowFull` with user-facing display text.
  - `crates/nexis-context/src/manager.rs` compiles with `Err(ContextError::WindowFull)`.
  - `cargo check -p nexis-context` succeeds.
- **Dependencies:** P0-002
- **Estimated effort:** S
- **Assigned to:** 

### P0-004
- **Title:** Add `ContextError::SummarizationNotAvailable` for no-op summarizer
- **Priority:** P0
- **Description:** Add missing `SummarizationNotAvailable` variant to `ContextError` and keep `NoOpSummarizer` returning this explicit error.
- **Acceptance Criteria:**
  - `crates/nexis-context/src/error.rs` defines `SummarizationNotAvailable` with display text.
  - `crates/nexis-context/src/summarizer.rs` compiles with `ContextError::SummarizationNotAvailable`.
  - `cargo check -p nexis-context` succeeds.
- **Dependencies:** P0-002
- **Estimated effort:** S
- **Assigned to:** 

## P1 (Lint Hygiene)

### P1-001
- **Title:** Remove unused `async_trait` import from plugin manager
- **Priority:** P1
- **Description:** Remove dead import in `crates/nexis-plugin/src/manager.rs` to eliminate lint noise.
- **Acceptance Criteria:**
  - `use async_trait::async_trait;` is removed from the file.
  - `cargo clippy -p nexis-plugin -- -D warnings` has no unused import warning.
- **Dependencies:** P0-003, P0-004
- **Estimated effort:** S
- **Assigned to:** 

### P1-002
- **Title:** Eliminate unused `plugin` variable in unregister path
- **Priority:** P1
- **Description:** Refactor unregister branch to avoid binding an unused variable while preserving behavior.
- **Acceptance Criteria:**
  - `crates/nexis-plugin/src/manager.rs` no longer emits unused variable warning for `plugin`.
  - `cargo clippy -p nexis-plugin -- -D warnings` passes for this warning class.
- **Dependencies:** P0-003, P0-004
- **Estimated effort:** S
- **Assigned to:** 

## P2 (Test Coverage)

### P2-001
- **Title:** Add regression test for `NoOpSummarizer` unavailable error
- **Priority:** P2
- **Description:** Add a targeted regression test proving `NoOpSummarizer` returns `ContextError::SummarizationNotAvailable` exactly.
- **Acceptance Criteria:**
  - Test exists in `crates/nexis-context/src/summarizer.rs` test module.
  - Test asserts exact error variant equality.
  - `cargo test -p nexis-context` passes.
- **Dependencies:** P0-004
- **Estimated effort:** S
- **Assigned to:** 

### P2-002
- **Title:** Add regression test for context window overflow fail mode
- **Priority:** P2
- **Description:** Add regression test proving overflow with `OverflowStrategy::Fail` returns `ContextError::WindowFull`.
- **Acceptance Criteria:**
  - Test exists in `crates/nexis-context/src/manager.rs` test module.
  - Test asserts exact error variant equality.
  - `cargo test -p nexis-context` passes.
- **Dependencies:** P0-003
- **Estimated effort:** S
- **Assigned to:** 

### P2-003
- **Title:** Add `nexis-memory` tests for context window eviction and boundaries
- **Priority:** P2
- **Description:** Create `crates/nexis-memory/tests/context_manager.rs` covering `ContextWindow::push` eviction order and `max_entries` constraints.
- **Acceptance Criteria:**
  - Tests cover eviction ordering and max boundary behavior.
  - Both empty and full-window cases are asserted.
  - `cargo test -p nexis-memory` passes.
- **Dependencies:** P1-001, P1-002
- **Estimated effort:** M
- **Assigned to:** 

### P2-004
- **Title:** Add `nexis-memory` tests for recent-load semantics
- **Priority:** P2
- **Description:** Add `load_recent_with_limit` tests validating recency ordering and empty store handling.
- **Acceptance Criteria:**
  - Test file `crates/nexis-memory/tests/memory_entry.rs` or equivalent includes these scenarios.
  - Ordering assertions verify newest-first/expected recency contract.
  - `cargo test -p nexis-memory` passes.
- **Dependencies:** P2-003
- **Estimated effort:** M
- **Assigned to:** 

### P2-005
- **Title:** Add `MemoryEntry::new` initialization tests
- **Priority:** P2
- **Description:** Validate ID/timestamp initialization and metadata defaults in `MemoryEntry::new`.
- **Acceptance Criteria:**
  - Tests verify non-empty ID, initialized timestamps, and expected default metadata.
  - Failing assertions reproduce on intentionally broken defaults.
  - `cargo test -p nexis-memory` passes.
- **Dependencies:** P2-003
- **Estimated effort:** S
- **Assigned to:** 

### P2-006
- **Title:** Add embedding batch failure short-circuit tests
- **Priority:** P2
- **Description:** Add tests for `EmbeddingService::embed_batch` to ensure processing stops on first error and reports it.
- **Acceptance Criteria:**
  - Test exists in `crates/nexis-memory/tests/embedding_service.rs`.
  - Test verifies no further embeddings are attempted after first failure.
  - `cargo test -p nexis-memory` passes.
- **Dependencies:** P2-003
- **Estimated effort:** M
- **Assigned to:** 

### P2-007
- **Title:** Add plugin manager registration/unregistration branch tests
- **Priority:** P2
- **Description:** Add tests for duplicate registration rejection and both unregister outcomes (not-found/success).
- **Acceptance Criteria:**
  - Tests exist in `crates/nexis-plugin/tests/plugin_manager.rs`.
  - Duplicate, not-found, and successful remove branches are asserted.
  - `cargo test -p nexis-plugin` passes.
- **Dependencies:** P1-001, P1-002
- **Estimated effort:** M
- **Assigned to:** 

### P2-008
- **Title:** Add plugin dispatch behavior tests for resilience
- **Priority:** P2
- **Description:** Validate dispatch returns first `Some(Response)` and continues despite plugin hook errors where specified.
- **Acceptance Criteria:**
  - Tests exist in `crates/nexis-plugin/tests/dispatch_semantics.rs`.
  - Assertions cover first-response selection and error isolation behavior.
  - `cargo test -p nexis-plugin` passes.
- **Dependencies:** P2-007
- **Estimated effort:** M
- **Assigned to:** 

### P2-009
- **Title:** Add in-memory skill registry CRUD and duplicate tests
- **Priority:** P2
- **Description:** Add tests for `register/get/list/remove` happy paths and duplicate rejection in `nexis-skills`.
- **Acceptance Criteria:**
  - Tests exist in `crates/nexis-skills/tests/registry.rs`.
  - CRUD operations and duplicate path are covered with explicit assertions.
  - `cargo test -p nexis-skills` passes.
- **Dependencies:** P1-001, P1-002
- **Estimated effort:** M
- **Assigned to:** 

### P2-010
- **Title:** Add skill execution lifecycle and concurrency tests
- **Priority:** P2
- **Description:** Add tests for `SkillExecution::new` id/timestamp behavior and concurrent register/remove operations.
- **Acceptance Criteria:**
  - Tests exist in `crates/nexis-skills/tests/execution.rs`.
  - Includes at least one concurrency test with simultaneous operations.
  - `cargo test -p nexis-skills` passes.
- **Dependencies:** P2-009
- **Estimated effort:** M
- **Assigned to:** 

## P3 (Reliability / Runtime Safety)

### P3-001
- **Title:** Fix `ai-summarizer` feature-gated module imports and exports
- **Priority:** P3
- **Description:** Replace invalid `crate::types::*` import usage and correctly expose required module(s) under feature gate.
- **Acceptance Criteria:**
  - `ai_summarizer.rs` imports from `summarizer_types` (or approved canonical replacement).
  - `lib.rs` exports required module paths under `ai-summarizer` feature.
  - `cargo check -p nexis-context --features ai-summarizer` passes.
- **Dependencies:** P0-003, P0-004
- **Estimated effort:** S
- **Assigned to:** 

### P3-002
- **Title:** Replace panic-based federation lock handling with recoverable errors
- **Priority:** P3
- **Description:** Remove `.expect(...mutex poisoned)` in production federation paths and propagate typed errors.
- **Acceptance Criteria:**
  - Targeted `.expect` lock calls in `crates/nexis-federation/src/lib.rs` are removed from runtime paths.
  - Failures return a documented `FederationError` variant.
  - `cargo test -p nexis-federation` passes existing tests.
- **Dependencies:** P0-003, P0-004
- **Estimated effort:** L
- **Assigned to:** 

### P3-003
- **Title:** Make metrics registration and export non-panicking
- **Priority:** P3
- **Description:** Convert metrics init/export unwrap chains to result-based handling with actionable errors.
- **Acceptance Criteria:**
  - `metrics.rs` modules no longer call `.unwrap()` in registration/export paths.
  - Duplicate registration and encoding errors return/log structured failures.
  - Metrics endpoint returns HTTP 500 (not panic) on encoding failures.
- **Dependencies:** P0-003, P0-004
- **Estimated effort:** M
- **Assigned to:** 

### P3-004
- **Title:** Resolve plugin extension-point API exposure mismatch
- **Priority:** P3
- **Description:** Either wire extension-point traits into runtime manager flow or remove/mark public exports as experimental.
- **Acceptance Criteria:**
  - Decision documented in code comments/changelog.
  - Public API matches actual runtime wiring (no dead exported surface without annotation).
  - `cargo check -p nexis-plugin` passes.
- **Dependencies:** P1-001, P1-002
- **Estimated effort:** M
- **Assigned to:** 

### P3-005
- **Title:** Implement deterministic plugin teardown on unregister
- **Priority:** P3
- **Description:** Refactor plugin ownership so `unregister` can invoke teardown safely with timeout/logging.
- **Acceptance Criteria:**
  - `unregister` triggers teardown callback for removable plugins.
  - Teardown timeout/error path is logged and non-panicking.
  - `cargo test -p nexis-plugin` includes teardown behavior coverage.
- **Dependencies:** P2-007
- **Estimated effort:** L
- **Assigned to:** 

## P4 (Architecture / Maintainability)

### P4-001
- **Title:** Split gateway router module by bounded context
- **Priority:** P4
- **Description:** Decompose `crates/nexis-gateway/src/router/mod.rs` into focused submodules (e.g., rooms/messages/health/admin/search).
- **Acceptance Criteria:**
  - Router code moved into multiple submodules with clear ownership.
  - Public API and route behavior remain unchanged (regression tests pass).
  - `cargo test -p nexis-gateway` passes.
- **Dependencies:** P3-003
- **Estimated effort:** L
- **Assigned to:** 

### P4-002
- **Title:** Split gateway DB module into query/service/model layers
- **Priority:** P4
- **Description:** Decompose `crates/nexis-gateway/src/db/mod.rs` into smaller modules with narrow interfaces.
- **Acceptance Criteria:**
  - Query, service, and model responsibilities are separated into dedicated modules.
  - Existing DB call sites compile without behavior change.
  - `cargo test -p nexis-gateway` passes.
- **Dependencies:** P4-001
- **Estimated effort:** L
- **Assigned to:** 

### P4-003
- **Title:** Decompose federation core into handshake/trust/delivery/security modules
- **Priority:** P4
- **Description:** Split `crates/nexis-federation/src/lib.rs` into domain modules to reduce churn and review complexity.
- **Acceptance Criteria:**
  - New modules exist with clear boundaries and minimal cross-module coupling.
  - Existing federation behaviors are preserved by tests.
  - `cargo test -p nexis-federation` passes.
- **Dependencies:** P3-002
- **Estimated effort:** L
- **Assigned to:** 

### P4-004
- **Title:** Unify duplicated context message domain types
- **Priority:** P4
- **Description:** Establish one canonical `Message`/`MessageRole` representation (or explicit conversion boundary) across context and summarizer types.
- **Acceptance Criteria:**
  - Duplicate type definitions are removed or formally bridged with `From` conversions.
  - Serialization compatibility is validated by tests.
  - `cargo test -p nexis-context` passes.
- **Dependencies:** P3-001
- **Estimated effort:** M
- **Assigned to:** 

### P4-005
- **Title:** Standardize async lock primitives and lock-ordering guidance
- **Priority:** P4
- **Description:** Migrate async-facing shared state from sync locks to `tokio::sync` equivalents where appropriate and document lock ordering.
- **Acceptance Criteria:**
  - Identified async hot paths no longer use `std::sync::{Mutex,RwLock}`.
  - Crate-level docs define lock ordering rules.
  - Relevant crate tests pass without deadlock regressions.
- **Dependencies:** P3-002
- **Estimated effort:** L
- **Assigned to:** 

### P4-006
- **Title:** Harden CI with all-features and disk preflight checks
- **Priority:** P4
- **Description:** Expand Rust CI to detect feature-gated failures, lint drift, and low-disk conditions early.
- **Acceptance Criteria:**
  - Workflow includes:
    - `cargo check --workspace --all-features`
    - `cargo test --workspace --all-features`
    - `cargo clippy --workspace --all-targets --all-features -D warnings`
  - CI preflight fails fast when disk is below defined threshold.
  - At least one PR run demonstrates new jobs executing.
- **Dependencies:** P0-002, P3-001
- **Estimated effort:** M
- **Assigned to:** 
