# Deployment Guide

## Quick Start

1. Build and start the gateway:
   ```bash
   docker compose up --build gateway
   ```
2. Verify service health:
   ```bash
   curl -f http://localhost:8080/health
   ```
3. Stop the stack when done:
   ```bash
   docker compose down
   ```

To run optional local dependencies:

```bash
docker compose --profile with-db --profile with-cache up --build
```

## Environment Variables

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `NEXIS_BIND_ADDR` | No | `0.0.0.0:8080` | Gateway bind address. |
| `NEXIS_LOG_LEVEL` | No | `info` | Log verbosity (`error`, `warn`, `info`, `debug`, `trace`). |
| `NEXIS_CORS_ALLOW_ORIGINS` | Yes (prod) | `http://localhost:5173,http://127.0.0.1:5173` | Comma-separated allowed origins. |
| `NEXIS_CORS_ALLOW_CREDENTIALS` | No | `true` | Enables credentialed CORS requests. |
| `NEXIS_HTTPS_REDIRECT_ENABLED` | Yes (prod) | `false` | Redirect HTTP requests to HTTPS. |
| `NEXIS_HSTS_ENABLED` | Yes (prod) | `true` | Adds HSTS response header. |
| `NEXIS_CSP_POLICY` | No | Secure default policy | Content-Security-Policy header override. |

## Production Checklist

- Use a production image tag (not `latest`) and pin base image digests.
- Enable HTTPS redirect and HSTS.
- Set strict `NEXIS_CORS_ALLOW_ORIGINS` values for your deployed frontend domains.
- Store JWT secrets and credentials in a managed secret store.
- Keep `restart` and resource limits enabled in `docker-compose.yml`.
- Keep log rotation enabled (`max-size` / `max-file`) to avoid disk exhaustion.
- Run database and cache with persistent volumes and backup policies.
- Monitor `/health`, request error rates, and container restarts.
- Apply dependency and OS package updates on a regular cadence.
