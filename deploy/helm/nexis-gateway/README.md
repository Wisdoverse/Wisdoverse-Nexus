# Wisdoverse Nexus Gateway Helm Chart

This chart describes the M1 evaluation deployment of the gateway. It is a
single-replica, in-memory profile; it is not qualified for production traffic,
horizontal scaling, or durable room and message state.

## M1 operating limits

- `replicaCount` must be `1` and `autoscaling.enabled` must be `false`. The
  Deployment template fails rendering if either setting requests multiple
  replicas or an HPA.
- The Deployment uses the `Recreate` strategy so an upgrade does not overlap
  old and new gateway processes. This causes downtime, and restart or upgrade
  loses process-local room and message state.
- The chart injects `JWT_SECRET` from a Kubernetes Secret that you provision
  through your secret manager. It does not create a Secret or accept secret
  material in Helm values.
- `startupProbe`, `readinessProbe`, and `livenessProbe` use `/health`; the pod
  has a 30-second termination grace period. The container runs as non-root with
  a read-only root filesystem, privilege escalation disabled, and all Linux
  capabilities dropped.
- Set `image.digest` to pin the built image. The chart's tag fallback is for
  evaluation convenience; a chart default tag does not identify an image built
  for a particular change.

These settings do not qualify durable recovery, multi-node operation, cluster
runtime behavior, or WebSocket draining. Those remain open validation and
architecture work for M3.

## Render and lint

Run these commands from the repository root. The synthetic Secret name and
image digest below are render fixtures only. Helm does not create a Secret or
deploy resources during `template` or `lint`.

```bash
helm template nexis-gateway ./deploy/helm/nexis-gateway \
  --namespace evaluation \
  --set-string image.repository=example.invalid/nexis-gateway \
  --set-string image.digest=sha256:0000000000000000000000000000000000000000000000000000000000000000 \
  --set-string secrets.existingSecret=synthetic-auth-fixture

helm lint ./deploy/helm/nexis-gateway \
  --set-string image.repository=example.invalid/nexis-gateway \
  --set-string image.digest=sha256:0000000000000000000000000000000000000000000000000000000000000000 \
  --set-string secrets.existingSecret=synthetic-auth-fixture
```

These are the commands for validation; their results must be recorded for the
revision under review. Rendering and linting do not prove cluster behavior or
deployment qualification.

## Install an evaluation release

Before installing, ensure the `evaluation` namespace exists and have your
secret manager provision the Secret there. By default, the chart reads the
`jwt-secret` key from that Secret. The key must contain the JWT signing secret
expected by the gateway. Supply a repository and digest for an image you have
built; do not use a floating tag for a reproducible evaluation.

```bash
helm install nexis-gateway ./deploy/helm/nexis-gateway \
  --namespace evaluation \
  --set-string image.repository="${GATEWAY_IMAGE_REPOSITORY}" \
  --set-string image.digest="${GATEWAY_IMAGE_DIGEST}" \
  --set-string secrets.existingSecret=nexis-gateway-auth
```

`GATEWAY_IMAGE_REPOSITORY` and `GATEWAY_IMAGE_DIGEST` must identify the image
you built. The `nexis-gateway-auth` Secret must already exist in the
`evaluation` namespace. This command installs a single-instance evaluation
release; it does not establish production readiness or durable state.

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Must remain one for M1 | `1` |
| `autoscaling.enabled` | HPA is rejected for M1 | `false` |
| `image.repository` | Image repository; set to the repository you built | `ghcr.io/wisdoverse/wisdoverse-nexus` |
| `image.digest` | Optional immutable image digest; preferred for reproducible deploys | `""` |
| `image.tag` | Used only when `image.digest` is empty | chart default |
| `secrets.existingSecret` | Pre-existing Secret containing the JWT key | `""` (required) |
| `secrets.jwtSecretKey` | Key in the pre-existing Secret | `jwt-secret` |
| `terminationGracePeriodSeconds` | Pod shutdown grace period | `30` |
| `service.type` | Kubernetes Service type | `ClusterIP` |
| `service.port` | Service port | `8080` |
| `startupProbe.enabled` | Enable startup probe | `true` |
| `readinessProbe.enabled` | Enable readiness probe | `true` |
| `livenessProbe.enabled` | Enable liveness probe | `true` |

The previous `secrets.jwtSecret`, `secrets.databaseUrl`, and
`secrets.redisUrl` inline values have been removed. Migrate from old `--set`
usage by provisioning the Kubernetes Secret through your secret manager and
passing only its name with `secrets.existingSecret`. Helm values cannot create
or populate that Secret.

## Upgrade and uninstall

Upgrades use `Recreate`, so the old process stops before the replacement starts.
Expect downtime and loss of in-memory room/message state during upgrade or
restart. Durable recovery and graceful WebSocket draining have not been
qualified.

```bash
helm upgrade nexis-gateway ./deploy/helm/nexis-gateway \
  --namespace evaluation \
  --set-string image.repository="${GATEWAY_IMAGE_REPOSITORY}" \
  --set-string image.digest="${GATEWAY_IMAGE_DIGEST}" \
  --set-string secrets.existingSecret=nexis-gateway-auth

helm uninstall nexis-gateway --namespace evaluation
```
