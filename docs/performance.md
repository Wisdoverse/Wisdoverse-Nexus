# Performance Benchmarks

This document defines the limited-scope benchmarks used for fast CI iteration in Wisdoverse Nexus.

## Benchmark Targets

### `nexis-meeting` - `sfu_benchmark`

- `sfu/join_leave_room`: Measures in-memory `SfuRoom::join_room` and `SfuRoom::leave_room`.
- `sfu/publish_subscribe_track`: Measures `SfuRoom::publish_track` and `SfuRoom::subscribe_track` for active participants.

### `nexis-doc` - `crdt_benchmark`

- `crdt/apply_update`: Measures `CRDTDocument::apply_update` for a representative binary update payload.
- `crdt/encode_update`: Measures `CRDTDocument::encode_update` full-state serialization path.

## Latency Targets

- `sfu/join_leave_room` target: `< 1 ms` per operation pair.
- `crdt/apply_update` target: `< 100μs` per operation.

## Optimization Notes

- Keep benchmarks narrowly scoped and deterministic to maintain fast CI cycle times.
- Prefer minimizing heap allocations in hot paths (`join_room`, `apply_update`, `encode_update`).
- Reuse preallocated payload buffers in benchmark setup to isolate method execution cost.
- Track regressions with Criterion history and investigate changes that exceed target latency.

## Run Commands

```bash
cargo bench -p nexis-meeting --bench sfu_benchmark
cargo bench -p nexis-doc --bench crdt_benchmark
```
