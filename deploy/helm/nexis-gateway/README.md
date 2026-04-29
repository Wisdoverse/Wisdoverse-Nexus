# Wisdoverse Nexus Gateway Helm Chart

Helm chart for deploying the Wisdoverse Nexus gateway on Kubernetes.

## Features

- non-root container security context
- optional horizontal pod autoscaling
- optional NetworkPolicy
- optional ingress and cert-manager annotations
- liveness and readiness probes

## Quick Start

```bash
# Add namespace
kubectl create namespace production

# Install chart
helm install nexis-gateway ./deploy/helm/nexis-gateway \
  -n production \
  --set image.repository=ghcr.io/wisdoverse/wisdoverse-nexus \
  --set image.tag=0.1.0 \
  --set secrets.jwtSecret='<replace-with-32-byte-secret>' \
  --set secrets.databaseUrl='postgresql://user:pass@host:5432/nexis'
```

The chart is a baseline. Operators must review ingress, storage, database,
secrets, resource limits, and network policy before using it for production
traffic.

## Configuration

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `3` |
| `image.repository` | Image repository | `ghcr.io/wisdoverse/wisdoverse-nexus` |
| `image.tag` | Image tag | `0.1.0` |
| `service.type` | Service type | `ClusterIP` |
| `service.port` | Service port | `8080` |
| `autoscaling.enabled` | Enable HPA | `true` |
| `autoscaling.minReplicas` | Minimum replicas | `3` |
| `autoscaling.maxReplicas` | Maximum replicas | `100` |
| `networkPolicy.enabled` | Enable NetworkPolicy | `true` |
| `ingress.enabled` | Enable Ingress | `true` |
| `ingress.className` | Ingress class | `nginx` |

## Security Configuration

The chart applies these security best practices by default:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 1000
  readOnlyRootFilesystem: true
  allowPrivilegeEscalation: false
  capabilities:
    drop:
      - ALL
```

## TLS Configuration

```bash
helm install nexis-gateway ./deploy/helm/nexis-gateway \
  -n production \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=api.wisdoverse.com
```

## Resources

Default resource configuration:

```yaml
resources:
  limits:
    cpu: 2000m
    memory: 2Gi
  requests:
    cpu: 500m
    memory: 512Mi
```

## Upgrading

```bash
helm upgrade nexis-gateway ./deploy/helm/nexis-gateway -n production
```

## Uninstalling

```bash
helm uninstall nexis-gateway -n production
```
