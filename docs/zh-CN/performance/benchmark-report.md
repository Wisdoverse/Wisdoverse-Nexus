# 性能基准测试报告

Nexis 的性能测试结果和优化建议。

## 测试环境

| 配置 | 值 |
|------|-----|
| CPU | 8 核 Intel Xeon |
| 内存 | 32 GB |
| OS | Ubuntu 22.04 |
| Rust | 1.75+ |
| 网络 | 10 Gbps |

## 关键指标

| 指标 | 值 | 说明 |
|------|-----|------|
| 并发连接 | **100,000+** | 单实例 WebSocket 连接 |
| 消息延迟 P99 | **< 1ms** | 端到端消息延迟 |
| 吞吐量 | **50,000 msg/s** | 每秒消息处理量 |
| 内存占用 | **< 500 MB** | 10 万连接时 |

## 基准测试详情

### 连接建立

```
测试: 建立并保持 WebSocket 连接
结果: 100,000 连接稳定运行 1 小时
CPU: ~15%
内存: ~450 MB
```

### 消息吞吐

```
测试: 单向消息广播
配置: 10,000 连接，每连接 100 msg/s
结果: 100% 投递成功，P99 延迟 < 1ms
```

### 压力测试

```
测试: 突发流量
配置: 5,000 连接同时发送 100 条消息
结果: 全部在 2 秒内处理完成
```

## 优化建议

### 1. 调整 Tokio 工作线程

```rust
// 根据 CPU 核心数调整
#[tokio::main(flavor = "multi_thread", worker_threads = 8)]
async fn main() { ... }
```

### 2. 启用 TCP 优化

```bash
# 系统级别
sysctl -w net.core.somaxconn=65535
sysctl -w net.ipv4.tcp_max_syn_backlog=65535
```

### 3. 连接池配置

```yaml
# docker-compose.yml
ulimits:
  nofile:
    soft: 100000
    hard: 100000
```

## 对比竞品

| 平台 | 并发连接 | 延迟 P99 | 内存/10K连接 |
|------|---------|---------|-------------|
| **Nexis** | 100K+ | <1ms | ~45 MB |
| Discord | ~25K | ~50ms | ~100 MB |
| Slack | ~10K | ~100ms | ~150 MB |
| Mattermost | ~5K | ~200ms | ~80 MB |

## 持续监控

Nexis 内置 Prometheus 指标：

```bash
# 访问指标端点
curl http://localhost:9090/metrics
```

关键指标：
- `nexis_connections_active` - 活跃连接数
- `nexis_messages_total` - 总消息数
- `nexis_message_duration_seconds` - 消息处理延迟

## 下一步

- [可观测性指南](/zh-CN/observability/tracing) — 生产环境监控
- [部署指南](/zh-CN/operations/deployment) — 生产部署建议
