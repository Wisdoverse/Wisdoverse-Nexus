# Deployment Guide

## Prerequisites

- Docker 24+
- Docker Compose 2.20+
- PostgreSQL 16+
- Qdrant 1.7+ (for vector search)

## Quick Deploy (Docker Compose)

```bash
# Clone repository
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis

# Configure environment
cp .env.example .env
# Edit .env with your settings

# Start services
docker-compose up -d
```

## Production Deployment

### 1. Database Setup

```sql
CREATE DATABASE nexis;
CREATE USER nexis WITH PASSWORD 'secure-password';
GRANT ALL PRIVILEGES ON DATABASE nexis TO nexis;
```

### 2. Run Migrations

```bash
sqlx migrate run
```

### 3. Start Gateway

```bash
nexis-gateway --config config/production.toml
```

### 4. Configure Reverse Proxy

Use nginx or Caddy as reverse proxy:

```nginx
server {
    listen 443 ssl;
    server_name api.nexis.ai;

    location / {
        proxy_pass http://localhost:3000;
        proxy_set_header Host $host;
    }
}
```

## Health Checks

- `/health` - Basic health check
- `/ready` - Readiness check (includes DB)

## Monitoring

See [Security](../architecture/security/overview.md) for security hardening.
