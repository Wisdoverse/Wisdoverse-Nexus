# Troubleshooting

Use this page for first-pass diagnosis of the source-available gateway. Prefer
evidence from logs, `/health`, `/metrics`, and the exact commit under test.

## Startup Failures

Run the gateway locally:

```bash
cargo run -p nexis-gateway
```

Check the basic endpoint:

```bash
curl -fsS http://localhost:8080/health
```

Common checks:

| Symptom | Check |
| --- | --- |
| Process exits in production mode | Ensure `JWT_SECRET` is set when `NEXIS_ENV=production`. |
| Port bind failure | Check the process using `8080` with `lsof -i :8080`. |
| Container starts but health fails | Inspect `docker compose logs nexis-gateway` and confirm the container exposes port `8080`. |
| Kubernetes rollout stalls | Inspect `kubectl describe pod` and `kubectl logs deployment/nexis-gateway -n <namespace>`. |

## Metrics Checks

Confirm the metrics endpoint:

```bash
curl -fsS http://localhost:8080/metrics | head
```

Prometheus target check:

```bash
curl -fsS http://localhost:9090/api/v1/targets
```

Useful PromQL:

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

## WebSocket Issues

Check these first:

- The client connects to the configured gateway host and `/ws` route.
- The token is signed with the same `JWT_SECRET` used by the gateway.
- Reverse proxies preserve WebSocket upgrade headers.
- Idle timeout values on proxies and load balancers are higher than the client
  heartbeat interval.

For local reproduction, include:

- gateway command and environment
- client connection URL with secrets redacted
- response status or close code
- relevant `RUST_LOG` output

## Deployment Checks

Docker Compose:

```bash
docker compose -f deploy/docker-compose.yml ps
docker compose -f deploy/docker-compose.yml logs --tail=200 nexis-gateway
```

Helm:

```bash
helm template nexis-gateway deploy/helm/nexis-gateway
helm lint deploy/helm/nexis-gateway
kubectl rollout status deployment/nexis-gateway -n <namespace>
```

Prometheus:

```bash
promtool check config deploy/prometheus.yml
promtool check rules ops/observability/prometheus-alert-rules.yaml
```

## Logs

The gateway uses Rust tracing. Increase detail locally with:

```bash
RUST_LOG=nexis_gateway=debug,tower_http=debug cargo run -p nexis-gateway
```

For containers:

```bash
docker compose -f deploy/docker-compose.yml logs -f nexis-gateway
```

For Kubernetes:

```bash
kubectl logs -f deployment/nexis-gateway -n <namespace>
```

## Performance Investigation

Start with application metrics and repeatable benchmark commands:

```bash
cargo bench -p nexis-gateway --bench websocket_connections
cargo bench -p nexis-gateway --bench message_throughput
cargo bench -p nexis-gateway --bench routing
```

When reporting performance results, include hardware, OS, Rust version, commit
SHA, exact command, and whether the run was local, containerized, or Kubernetes.

## Issue Reports

Bug reports should include:

- commit SHA and branch
- OS and runtime environment
- exact command or deployment path
- relevant logs with secrets redacted
- `/health` result
- `/metrics` snippets when operational behavior is involved
- expected behavior and actual behavior

Use GitHub Issues for reproducible bugs and GitHub Discussions for design
questions.
