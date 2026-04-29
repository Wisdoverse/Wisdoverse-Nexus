# Deployment Guide

The maintained deployment guide lives in the operations section:

- [Deployment](../operations/deployment.md)
- [Monitoring](../operations/monitoring.md)
- [Troubleshooting](../operations/troubleshooting.md)

For a local smoke test:

```bash
git clone https://github.com/Wisdoverse/Wisdoverse-Nexus.git
cd Wisdoverse-Nexus

docker compose up -d
curl http://localhost:8080/health
```
