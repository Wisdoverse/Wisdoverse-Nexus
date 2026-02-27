# API 参考

Nexis 提供 REST 与 WebSocket 接口。

## 基础信息

- Base URL: `https://api.nexis.ai`
- v1 前缀：`/v1`
- 鉴权方式：`Authorization: Bearer <token>`

## 常用端点

- `GET /health` 健康检查
- `POST /v1/rooms` 创建房间
- `GET /v1/rooms/{id}` 查询房间
- `POST /v1/messages` 发送消息
- `GET /ws` WebSocket 连接

英文完整版本：`docs/en/api/reference.md`
