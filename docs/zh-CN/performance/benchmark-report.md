# 性能基准

本页记录源码可用仓库中存在的 benchmark 入口。结果会受到硬件、操作系统、运行时配置和
feature flags 影响。本地结果应作为容量规划输入，而不是性能保证。

## Benchmark 范围

| 区域 | Benchmark |
| --- | --- |
| Gateway WebSocket 连接 | `crates/nexis-gateway/benches/websocket_connections.rs` |
| Gateway 消息吞吐 | `crates/nexis-gateway/benches/message_throughput.rs` |
| Gateway 路由 | `crates/nexis-gateway/benches/routing.rs` |
| 上下文摘要 | `crates/nexis-context/benches/context_summarization.rs` |
| Meeting SFU 域 | `crates/nexis-meeting/benches/sfu_benchmark.rs` |
| Document CRDT 域 | `crates/nexis-doc/benches/crdt_benchmark.rs` |

## 运行 Benchmark

```bash
cargo bench --workspace
```

运行单个 benchmark：

```bash
cargo bench -p nexis-gateway --bench websocket_connections
cargo bench -p nexis-gateway --bench message_throughput
cargo bench -p nexis-gateway --bench routing
cargo bench -p nexis-context --bench context_summarization
cargo bench -p nexis-meeting --bench sfu_benchmark
cargo bench -p nexis-doc --bench crdt_benchmark
```

## 报告结果要求

发布 benchmark 数字时应包含：

- Commit SHA
- CPU、内存、操作系统和 kernel version
- Rust version
- Build profile 和 feature flags
- Benchmark command
- 负载形态、连接数、消息大小和持续时间
- 是否使用 localhost、容器网络或远程网络

## 容量规划说明

- Gateway 连接容量取决于 file descriptor limits、内存、CPU、kernel 网络参数和负载形态。
- AI 摘要延迟主要取决于配置的 provider 和模型。
- 容器和 Kubernetes 部署在压测前应显式设置 CPU、内存和 file descriptor limits。
- 生产性能声明应基于目标环境中的可重复 benchmark。

## 测试期间监控

gateway 运行时在 `/metrics` 暴露 Prometheus metrics：

```bash
curl http://localhost:8080/metrics
```

建议同时采集主机 CPU、内存、网络和 file descriptor telemetry。
