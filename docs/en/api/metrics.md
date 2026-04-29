# Metrics API

## `GET /metrics`

Returns Prometheus text format metrics from the gateway process.

```bash
curl http://localhost:8080/metrics
```

The endpoint is implemented by `crates/nexis-gateway` and is registered beside
`GET /health`.

## Registered Metric Families

| Family | Purpose |
| --- | --- |
| `nexis_build_info` | Gateway package version and commit label |
| `nexis_http_requests_total` | HTTP request count by method and path |
| `nexis_http_responses_total` | HTTP response count by method, path, and status |
| `nexis_http_latency_seconds` | HTTP request latency histogram |
| `nexis_connections_active` | Active WebSocket connections |
| `nexis_connections_total` | Total WebSocket connections established |
| `nexis_connection_errors` | Connection errors by type |
| `nexis_pool_connections_active` | Active sharded-pool connections |
| `nexis_pool_connections_peak` | Peak sharded-pool connection count since start |
| `nexis_pool_connections_total` | Total sharded-pool connections created |
| `nexis_pool_messages_sent_total` | Broadcast messages sent through the pool |
| `nexis_pool_messages_dropped_total` | Broadcast messages dropped by the pool |
| `nexis_messages_received_total` | Messages received |
| `nexis_messages_sent_total` | Messages sent |
| `nexis_messages_by_type` | Messages grouped by type |
| `nexis_message_latency_seconds` | Message processing latency histogram |
| `nexis_message_size_bytes` | Message size histogram |
| `nexis_rooms_active` | Active rooms |
| `nexis_rooms_created_total` | Rooms created since start |
| `nexis_room_members` | Member count by room |
| `nexis_operation_throughput_total` | Domain operation throughput |
| `nexis_operation_errors_total` | Domain operation errors |
| `nexis_operation_latency_seconds` | Domain operation latency histogram |
| `nexis_ai_requests_total` | AI provider request count |
| `nexis_ai_errors_total` | AI provider errors |
| `nexis_ai_latency_seconds` | AI provider latency histogram |
| `nexis_ai_tokens_total` | AI provider token usage |

## Dashboards and Alerts

Use the operations guide for Prometheus, Grafana, and alert rule setup:

- [Monitoring](../operations/monitoring.md)
- [Troubleshooting](../operations/troubleshooting.md)
