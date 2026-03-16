# Distributed Tracing with OpenTelemetry

Nexis Gateway supports distributed tracing via OpenTelemetry Protocol (OTLP).

## Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `NEXIS_OTEL_EXPORTER` | Trace exporter type: `stdout`, `none`, `otlp` | `stdout` |
| `NEXIS_OTEL_EXPORT_ENDPOINT` | OTLP collector endpoint (required for `otlp`) | - |
| `RUST_LOG` | Log level filter | `nexis_gateway=info,tower_http=info` |
| `NEXIS_LOG_FORMAT` | Log format: `json` or `text` | `json` |
| `NEXIS_LOG_SPANS` | Include span lists in logs | `true` |

### Enable OTLP Export

```bash
export NEXIS_OTEL_EXPORTER=otlp
export NEXIS_OTEL_EXPORT_ENDPOINT=http://localhost:4317
export RUST_LOG=nexis_gateway=debug
```

## Collectors

### Jaeger

```bash
# Run Jaeger with OTLP enabled
docker run -d --name jaeger \
  -p 16686:16686 \
  -p 4317:4317 \
  -p 4318:4318 \
  jaegertracing/all-in-one:latest

# Configure Nexis
export NEXIS_OTEL_EXPORT_ENDPOINT=http://localhost:4317
```

### Grafana Tempo

```bash
# Tempo listens on port 4317 for OTLP gRPC
export NEXIS_OTEL_EXPORT_ENDPOINT=http://tempo:4317
```

### OpenTelemetry Collector

```yaml
# otel-collector-config.yaml
receivers:
  otlp:
    protocols:
      grpc:
        endpoint: 0.0.0.0:4317

exporters:
  jaeger:
    endpoint: jaeger:14250
    tls:
      insecure: true

service:
  pipelines:
    traces:
      receivers: [otlp]
      exporters: [jaeger]
```

## Graceful Shutdown

Call `observability::shutdown()` during application shutdown to ensure all pending spans are flushed:

```rust
// In your shutdown handler
nexis_gateway::observability::shutdown();
```

## Architecture

```
┌─────────────────┐     OTLP/gRPC     ┌──────────────────┐
│  Nexis Gateway  │ ─────────────────▶│ OTLP Collector   │
│  (tracing)      │                   │ (Jaeger/Tempo)   │
└─────────────────┘                   └──────────────────┘
```

## Span Attributes

Nexis Gateway automatically captures:

- HTTP request/response metadata
- WebSocket connection lifecycle
- Authentication events
- Message routing operations

## Troubleshooting

### No traces appearing

1. Verify collector is running and accessible
2. Check `NEXIS_OTEL_EXPORT_ENDPOINT` is correct
3. Ensure `NEXIS_OTEL_EXPORTER=otlp` is set
4. Check logs for connection errors

### High cardinality warnings

Adjust `RUST_LOG` to reduce trace volume:

```bash
export RUST_LOG=nexis_gateway=info
```
