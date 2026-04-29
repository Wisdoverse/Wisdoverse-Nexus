# Monitoring

This page documents the monitoring surface that exists in the source-available
repository. Treat metric names and file paths here as the source of truth for
Prometheus, Grafana, and alert rule work.

## Endpoints

The gateway exposes Prometheus metrics from the same HTTP service as the API:

```bash
curl http://localhost:8080/metrics
```

The liveness endpoint is:

```bash
curl http://localhost:8080/health
```

## Prometheus Configuration

The repository includes a local scrape configuration at `deploy/prometheus.yml`:

```yaml
scrape_configs:
  - job_name: "nexis-gateway"
    metrics_path: "/metrics"
    static_configs:
      - targets: ["nexis-gateway:8080"]
```

For Kubernetes, create a `ServiceMonitor` or equivalent scrape job that points to
the gateway service on port `8080` and path `/metrics`.

## Gateway Metrics

Metrics are registered in `crates/nexis-gateway/src/metrics.rs`.

| Area | Metrics |
| --- | --- |
| Build | `nexis_build_info` |
| HTTP | `nexis_http_requests_total`, `nexis_http_responses_total`, `nexis_http_latency_seconds` |
| WebSocket connections | `nexis_connections_active`, `nexis_connections_total`, `nexis_connection_errors` |
| Connection pool | `nexis_pool_connections_active`, `nexis_pool_connections_peak`, `nexis_pool_connections_total` |
| Pool broadcasts | `nexis_pool_messages_sent_total`, `nexis_pool_messages_dropped_total` |
| Messages | `nexis_messages_received_total`, `nexis_messages_sent_total`, `nexis_messages_by_type`, `nexis_message_latency_seconds`, `nexis_message_size_bytes` |
| Rooms | `nexis_rooms_active`, `nexis_rooms_created_total`, `nexis_room_members` |
| Operations | `nexis_operation_throughput_total`, `nexis_operation_errors_total`, `nexis_operation_latency_seconds` |
| AI providers | `nexis_ai_requests_total`, `nexis_ai_errors_total`, `nexis_ai_latency_seconds`, `nexis_ai_tokens_total` |

Do not document database, cache, or user-count metrics as available until they
are registered in code.

## Dashboards

Dashboard files currently live in:

- `deploy/grafana/dashboards/nexis-overview.json`
- `ops/observability/grafana-dashboard-gateway.json`

When adding or changing a dashboard, verify its PromQL against the metrics above
and include the dashboard path in the pull request.

## Alert Rules

The maintained alert rule entrypoint is
`ops/observability/prometheus-alert-rules.yaml`. It currently covers:

- high gateway operation error rate
- high operation p95 latency

Run a syntax check before changing rules:

```bash
promtool check rules ops/observability/prometheus-alert-rules.yaml
```

If `promtool` is not installed, note that in the pull request and verify the
PromQL manually against a local Prometheus instance.

## Useful Queries

```text
rate(nexis_http_responses_total{status=~"5.."}[5m])
```

```text
histogram_quantile(
  0.95,
  sum(rate(nexis_http_latency_seconds_bucket[5m])) by (le, method, path)
)
```

```text
nexis_pool_connections_active
```

```text
sum(rate(nexis_operation_errors_total[5m]))
/
clamp_min(sum(rate(nexis_operation_throughput_total[5m])), 1)
```

## Operating Notes

- Set alert thresholds from observed baseline data, not from documentation
  examples.
- Keep dashboards and alert rules in source control.
- Avoid secrets in Alertmanager examples, screenshots, and exported dashboards.
- Include commit SHA, image tag, and environment label in incident notes.
