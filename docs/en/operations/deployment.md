# Deployment

This guide describes the deployment assets that are present in the source-available
repository and the operational checks maintainers expect downstream users to
perform before production use.

## Repository Assets

| Asset | Purpose |
| --- | --- |
| `docker-compose.yml` | Minimal local gateway stack |
| `docker/Dockerfile.gateway` | Gateway container image build |
| `deploy/docker-compose.yml` | Development stack with PostgreSQL, Redis, Prometheus, and Grafana |
| `deploy/docker-compose.prod.yml` | Production-oriented Compose example |
| `deploy/helm/nexis-gateway/` | Helm chart for the gateway |
| `deploy/prometheus.yml` | Prometheus scrape configuration |
| `deploy/grafana/dashboards/` | Grafana dashboard assets |

## Local Gateway

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

docker compose up -d
docker compose ps
curl http://localhost:8080/health
```

The expected health response is plain text:

```text
OK
```

Stop the stack:

```bash
docker compose down
```

## Development Infrastructure

Use the development compose file when you need PostgreSQL, Redis, Prometheus, and
Grafana for integration work:

```bash
docker compose -f deploy/docker-compose.yml up -d
docker compose -f deploy/docker-compose.yml ps
```

Default local ports:

| Service | Port |
| --- | --- |
| Gateway | `8080` |
| PostgreSQL | `5432` |
| Redis | `6379` |
| Prometheus | `9090` |
| Grafana | `3000` |

## Helm

The gateway Helm chart is under `deploy/helm/nexis-gateway/`.

```bash
helm lint deploy/helm/nexis-gateway
helm template nexis-gateway deploy/helm/nexis-gateway
```

Before installing into a cluster, review values for:

- Image repository and tag
- Resource requests and limits
- Environment variables and secrets
- Ingress and TLS
- Service account permissions
- Network policies

## Runtime Configuration

The repository uses `NEXIS_*` environment variables for gateway behavior. Common
operator-owned settings include:

| Variable | Purpose |
| --- | --- |
| `RUST_LOG` | Rust tracing/log filter |
| `NEXIS_ENV` | Environment label used by deployment config |
| `NEXIS_JWT_SECRET` or `JWT_SECRET` | JWT signing secret, depending on the entrypoint path |
| `NEXIS_CORS_ALLOW_ORIGINS` | Allowed browser origins |
| `NEXIS_HTTPS_REDIRECT_ENABLED` | HTTPS redirect middleware toggle |
| `NEXIS_HSTS_ENABLED` | HSTS middleware toggle |

Keep secrets out of git and inject them through your runtime platform.

## Health and Observability

| Endpoint | Description |
| --- | --- |
| `GET /health` | Basic liveness check |
| `GET /metrics` | Prometheus metrics |
| `GET /openapi.json` | OpenAPI document |
| `GET /docs` | Swagger UI |

Example:

```bash
curl http://localhost:8080/health
curl http://localhost:8080/metrics
```

## Production Checklist

Before production deployment, complete and record:

- Image provenance and version pinning
- TLS termination and certificate renewal
- Secrets management and rotation plan
- Database backup and restore test
- Resource limits and file descriptor limits
- CORS policy for the exact public origins
- Log retention and privacy review
- Prometheus alert rules and on-call routing
- Rollback plan
- Load test in the target environment
- Security review for exposed endpoints and network policies

## Post-Deployment Smoke Test

```bash
curl -fsS https://<your-host>/health
curl -fsS https://<your-host>/openapi.json >/tmp/wisdoverse-nexus-openapi.json
```

If metrics are exposed internally, validate from the monitoring network:

```bash
curl -fsS http://<gateway-service>:8080/metrics
```

## Related Documentation

- [Monitoring](monitoring.md)
- [Troubleshooting](troubleshooting.md)
- [Security Overview](../architecture/security/overview.md)
- [Key Management](../architecture/security/key-management.md)
