# 部署指南

## 环境要求

- Docker 24+
- Docker Compose 2.20+
- PostgreSQL 16+
- Qdrant 1.7+（向量检索）

## Docker Compose 快速部署

```bash
git clone https://github.com/gbrothersgroup/Nexis.git
cd Nexis
cp .env.example .env
# 修改 .env 配置

docker-compose up -d
```

## 生产部署步骤

1. 初始化数据库。
2. 执行迁移（`sqlx migrate run`）。
3. 启动网关（`nexis-gateway --config config/production.toml`）。
4. 配置反向代理（如 nginx/Caddy）。
