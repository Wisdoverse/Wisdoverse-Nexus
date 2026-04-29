# Performance Benchmarks

This page documents the benchmark entry points that are present in the
source-available repository. Results vary by hardware, operating system, runtime
configuration, and feature flags. Treat local results as capacity-planning input,
not as a guarantee.

## Benchmark Scope

| Area | Benchmark |
| --- | --- |
| Gateway WebSocket connections | `crates/nexis-gateway/benches/websocket_connections.rs` |
| Gateway message throughput | `crates/nexis-gateway/benches/message_throughput.rs` |
| Gateway routing | `crates/nexis-gateway/benches/routing.rs` |
| Context summarization | `crates/nexis-context/benches/context_summarization.rs` |
| Meeting SFU domain | `crates/nexis-meeting/benches/sfu_benchmark.rs` |
| Document CRDT domain | `crates/nexis-doc/benches/crdt_benchmark.rs` |

## Run Benchmarks

```bash
cargo bench --workspace
```

Run a focused benchmark:

```bash
cargo bench -p nexis-gateway --bench websocket_connections
cargo bench -p nexis-gateway --bench message_throughput
cargo bench -p nexis-gateway --bench routing
cargo bench -p nexis-context --bench context_summarization
cargo bench -p nexis-meeting --bench sfu_benchmark
cargo bench -p nexis-doc --bench crdt_benchmark
```

## Reporting Results

When publishing benchmark numbers, include:

- Commit SHA
- Host CPU, memory, operating system, and kernel version
- Rust version
- Build profile and feature flags
- Benchmark command
- Load shape, connection count, message size, and duration
- Whether the run used localhost, container networking, or a remote network

## Capacity Planning Notes

- Gateway connection capacity depends on file descriptor limits, memory, CPU,
  kernel networking settings, and workload shape.
- AI summarization latency depends heavily on the configured provider and model.
- Container and Kubernetes deployments should set explicit CPU, memory, and file
  descriptor limits before load testing.
- Production claims should be based on repeatable benchmarks in the target
  environment.

## Monitoring During Tests

The gateway exposes Prometheus metrics at `/metrics` when running:

```bash
curl http://localhost:8080/metrics
```

Use these metrics together with host-level CPU, memory, network, and file
descriptor telemetry.
