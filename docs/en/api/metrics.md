# Nexis Metrics API

## Prometheus Metrics Endpoint

**GET /metrics**

Returns Prometheus-compatible metrics for monitoring Nexis Gateway.

## Available Metrics

### Connection Pool Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nexis_pool_connections_active` | Gauge | Active WebSocket connections in sharded pool |
| `nexis_pool_connections_peak` | Gauge | Peak connection count since startup |
| `nexis_pool_connections_total` | Counter | Total connections ever created |

### Message Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nexis_pool_messages_sent_total` | Counter | Messages successfully broadcast |
| `nexis_pool_messages_dropped_total` | Counter | Messages dropped (no receivers) |

### Room Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nexis_rooms_active` | Gauge | Currently active rooms |
| `nexis_rooms_created_total` | Counter | Total rooms created |

### AI Provider Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nexis_ai_requests_total` | Counter | AI provider requests by provider |
| `nexis_ai_errors_total` | Counter | AI provider errors |
| `nexis_ai_latency_seconds` | Histogram | AI request latency |

### Context Summarization Metrics

| Metric | Type | Description |
|--------|------|-------------|
| `nexis_context_summarization_attempts` | Counter | Summarization attempts (success/failure) |
| `nexis_context_summarization_latency_seconds` | Histogram | Summarization latency |

## Example

```bash
curl http://localhost:8080/metrics
```

## Grafana Dashboard

Import the provided dashboard JSON from `deploy/grafana/dashboard.json` for pre-configured visualizations.

## Alerting Rules

Recommended Prometheus alerts:

```yaml
groups:
  - name: nexis
    rules:
      - alert: HighConnectionCount
        expr: nexis_pool_connections_active > 80000
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High connection count on Nexis Gateway"
      
      - alert: HighMessageDropRate
        expr: rate(nexis_pool_messages_dropped_total[5m]) > 100
        for: 5m
        labels:
          severity: warning
        annotations:
          summary: "High message drop rate"
```
