# Phase 6 设计 - 协作能力

## 0. 范围与目标

- 版本范围：Phase 6（2026-04-13 ~ 2026-07-03，10 周）
- 目标：为 Nexis 交付协作核心能力，覆盖视频会议、文档协作、任务管理、日程管理，并实现 AI 原生参与。
- 对齐基线：
  - `docs/zh-CN/roadmap/team-and-roadmap-2026.md`
  - `docs/zh-CN/design/nexis-vision-design.md`

本设计以 P0 能力优先上线、P1 能力同阶段收敛为原则，确保“可用闭环 + 可扩展架构 + 安全可审计”。

## 1. 功能设计

### 1.1 视频会议（P0，4 周）

#### 功能范围
- WebRTC 多人音视频通话
- 屏幕共享（单流共享 + 演讲者切换）
- AI Agent 参会（ASR/TTS + 文本消息同步）
- 会议录制、转写、摘要和行动项提取

#### 关键交互
- 用户创建会议 -> 参与者加入 -> 信令协商 -> SFU 转发媒体流
- 房主可邀请 AI 参会，AI 默认静音，授权后可发言
- 会议结束后自动触发“录制切片 -> 转写 -> 摘要生成 -> 回写文档/任务”

#### 验收指标
- 加入会议成功率 >= 99.5%
- 会议纪要生成时延 <= 会后 3 分钟
- 屏幕共享首帧时间 P95 <= 2 秒

### 1.2 文档协作（P0，3 周）

#### 功能范围
- 实时协同编辑（多人并发）
- AI 辅助写作（润色、续写、摘要、改写）
- 文档版本管理（快照、diff、回滚）
- 评论与批注（线程、@提及、建议转任务）

#### 关键交互
- 客户端通过 WebSocket 同步 CRDT 增量更新
- 服务端周期快照 + 操作日志持久化
- 评论中的“转任务”可直接创建任务并关联文档锚点

#### 验收指标
- 冲突恢复成功率 >= 99.99%
- 文档同步延迟 P95 <= 200ms（同 Region）
- 版本回滚成功率 = 100%

### 1.3 任务管理（P1，2 周）

#### 功能范围
- 任务创建、分配、状态流转
- 进度跟踪（看板 + 时间线）
- AI 自动提醒（截止前、阻塞态、逾期）
- 周报/月报自动生成

#### 关键交互
- 文档评论、会议行动项可一键转任务
- AI 根据任务 SLA 和 owner 时区推送提醒
- 报告聚合任务进度、风险和依赖阻塞

#### 验收指标
- 任务状态一致性正确率 100%
- AI 提醒触达成功率 >= 99%
- 报告生成时延 P95 <= 5 秒

### 1.4 日程管理（P1，1 周）

#### 功能范围
- 日历视图（日/周/月）
- 会议与任务截止事项排程
- 智能提醒（时区感知 + 冲突检测）

#### 关键交互
- 会议创建后自动生成日程事件
- 日程冲突时建议候选时间段
- 提醒通过站内通知 + 邮件（可配置）

#### 验收指标
- 日程创建成功率 >= 99.9%
- 冲突检测命中率 >= 95%

## 2. 技术架构

### 2.1 总体架构

- 客户端层：Web/Mobile
- 接入层：`gateway-service`（REST + WebSocket，鉴权、限流）
- 协作域服务：
  - `meeting-service`（信令、房间控制、录制编排）
  - `doc-service`（CRDT 协同、版本、评论）
  - `task-service`（任务流转、报告）
  - `calendar-service`（日程与提醒）
- AI 能力层：`ai-runtime-service`（ASR/TTS、摘要、写作、分配建议）
- 实时层：`realtime-service`（WebSocket 会话、presence、事件分发）
- 数据与事件层：PostgreSQL + Redis + Object Storage + NATS/Kafka
- 审计与策略层：`audit-policy-service`

### 2.2 视频会议架构（WebRTC P2P/SFU）

- 1:1 会议优先 P2P 直连（低成本、低延迟）
- 3 人及以上自动切换 SFU（可扩展、稳定）
- 信令通道使用 WebSocket，媒体通道使用 WebRTC（DTLS-SRTP）
- 录制采用 SFU 侧混流/分轨策略：
  - 默认分轨（便于后处理）
  - 导出纪要时可混流生成回放

### 2.3 文档协作架构（CRDT/OT）

- 主方案：CRDT（Yjs）
- 兼容层：保留 OT 适配接口用于外部文档系统迁移
- 状态持久化：
  - 高频增量更新写入 Redis Stream（短期）
  - 周期快照与版本元数据写入 PostgreSQL
  - 大型版本内容写入 Object Storage

### 2.4 实时同步架构（WebSocket）

- 单连接复用：消息、文档、会议控制、任务提醒走统一 WS 会话
- 事件主题：`meeting.*`、`doc.*`、`task.*`、`calendar.*`
- 断线恢复：客户端携带 `last_event_id` 请求增量补偿

### 2.5 AI 集成架构

- 视频：音频流 -> ASR -> LLM -> TTS -> AI 语音轨道注入会议
- 文档：用户选区 + 指令 -> LLM -> 生成建议 Patch -> 用户确认应用
- 任务：读取任务/成员负载/日程冲突 -> 输出分配建议和提醒计划
- 所有 AI 操作写审计日志（输入摘要、策略命中、输出摘要、trace_id）

### 2.6 安全架构

- 端到端加密：
  - 媒体流采用 WebRTC E2EE（Insertable Streams）
  - 文档协同通道使用 TLS + 租户级密钥封装
- 访问控制：
  - RBAC（角色）+ ABAC（资源属性/上下文）
  - 会议、文档、任务、日程均做租户隔离
- 审计日志：
  - 覆盖鉴权失败、权限变更、AI 自动动作、导出与删除操作
  - 日志不可变存储，保留 180 天（默认）

## 3. API 设计

### 3.1 会议 API

- `POST /api/v1/meetings`
  - 创建会议，返回 `meeting_id`、`join_token`
- `POST /api/v1/meetings/{meeting_id}/join`
  - 加入会议，返回 WebRTC 信令参数
- `POST /api/v1/meetings/{meeting_id}/screen-share/start`
- `POST /api/v1/meetings/{meeting_id}/recording/start`
- `POST /api/v1/meetings/{meeting_id}/recording/stop`
- `POST /api/v1/meetings/{meeting_id}/ai-agent/join`
- `GET /api/v1/meetings/{meeting_id}/summary`

示例响应（纪要）：
```json
{
  "meeting_id": "m_123",
  "summary": "讨论了 Phase 6 里程碑与风险项",
  "action_items": [
    {
      "text": "完成文档协同压测",
      "assignee_id": "u_42",
      "due_at": "2026-05-08T10:00:00Z"
    }
  ],
  "generated_at": "2026-05-01T09:03:00Z"
}
```

### 3.2 文档 API

- `POST /api/v1/docs`
- `GET /api/v1/docs/{doc_id}`
- `GET /api/v1/docs/{doc_id}/versions`
- `POST /api/v1/docs/{doc_id}/comments`
- `POST /api/v1/docs/{doc_id}/ai-assist`
- `POST /api/v1/docs/{doc_id}/rollback`

WebSocket 事件：
- `doc.patch.applied`
- `doc.comment.created`
- `doc.version.created`
- `doc.ai.suggestion.ready`

### 3.3 任务 API

- `POST /api/v1/tasks`
- `PATCH /api/v1/tasks/{task_id}`
- `POST /api/v1/tasks/{task_id}/assign`
- `GET /api/v1/tasks?project_id=...&status=...`
- `POST /api/v1/tasks/reports/generate`

### 3.4 日程 API

- `POST /api/v1/calendar/events`
- `PATCH /api/v1/calendar/events/{event_id}`
- `GET /api/v1/calendar/events?from=...&to=...`
- `POST /api/v1/calendar/conflicts/check`
- `POST /api/v1/calendar/reminders/snooze`

### 3.5 鉴权与幂等

- 所有写操作要求 `Authorization: Bearer <token>`
- 关键写接口支持 `Idempotency-Key`
- 租户隔离头：`X-Tenant-Id`

## 4. 数据模型

### 4.1 会议域

- `meetings(id, tenant_id, room_id, title, host_member_id, status, started_at, ended_at, created_at)`
- `meeting_participants(id, meeting_id, member_id, role, joined_at, left_at)`
- `meeting_recordings(id, meeting_id, storage_url, track_type, duration_sec, created_at)`
- `meeting_summaries(id, meeting_id, summary_text, action_items_json, model, generated_at)`

### 4.2 文档域

- `docs(id, tenant_id, space_id, title, current_version, created_by, created_at, updated_at)`
- `doc_snapshots(id, doc_id, version, storage_url, checksum, created_by, created_at)`
- `doc_ops(id, doc_id, op_seq, actor_id, op_type, payload_json, created_at)`
- `doc_comments(id, doc_id, anchor, content, author_id, status, created_at)`
- `doc_comment_threads(id, doc_id, root_comment_id, resolved_by, resolved_at)`

### 4.3 任务域

- `tasks(id, tenant_id, project_id, title, description, status, priority, assignee_id, due_at, source_type, source_ref_id, created_at, updated_at)`
- `task_history(id, task_id, actor_id, event_type, before_json, after_json, created_at)`
- `task_reports(id, tenant_id, period_type, period_start, period_end, content_json, generated_at)`

### 4.4 日程域

- `calendar_events(id, tenant_id, owner_id, title, start_at, end_at, source_type, source_ref_id, reminder_rule, created_at)`
- `calendar_attendees(id, event_id, member_id, response_status, updated_at)`
- `calendar_conflicts(id, tenant_id, member_id, event_a_id, event_b_id, severity, detected_at)`

### 4.5 安全与审计

- `access_policies(id, tenant_id, resource_type, action, effect, condition_json, updated_at)`
- `audit_logs(id, tenant_id, actor_id, actor_type, action, resource_type, resource_id, trace_id, result, metadata_json, created_at)`

## 5. 实现路线图

### 5.1 里程碑与节奏（10 周）

1. 周 1-2：会议基础设施
- 交付：WebRTC 信令、1:1 P2P、多人 SFU 房间
- 验收：多人会议稳定，断线重连可用

2. 周 3-4：会议增强 + AI 纪要
- 交付：屏幕共享、录制、ASR/TTS、自动摘要
- 验收：纪要时延达标（<=3 分钟）

3. 周 5-7：文档协作 P0
- 交付：CRDT 实时编辑、版本、评论、AI 写作
- 验收：冲突恢复与同步时延达标

4. 周 8-9：任务管理 P1
- 交付：任务流转、分配、提醒、报告
- 验收：任务闭环路径完整，提醒准确

5. 周 10：日程管理 P1 + 整体联调
- 交付：日历视图、智能提醒、冲突检测、跨模块联动
- 验收：会议-文档-任务-日程全链路通过

### 5.2 依赖关系

- `meeting-service` 依赖 `realtime-service` 和对象存储
- `doc-service` 依赖 WebSocket 与快照存储
- `task-service` 与 `calendar-service` 共享通知能力
- `ai-runtime-service` 贯穿四个子域并依赖统一审计

### 5.3 风险与缓解

- WebRTC 跨网络质量波动
  - 缓解：自适应码率、TURN 中继、QoS 监控
- CRDT 文档体积膨胀
  - 缓解：增量压缩、快照裁剪、后台 compaction
- AI 结果不稳定
  - 缓解：提示词模板化、结果置信度阈值、人工确认开关
- 权限误配风险
  - 缓解：策略回归测试 + 审计告警 + 最小权限默认策略

### 5.4 发布门禁

- 功能门禁：P0 场景 100% 通过
- 质量门禁：关键路径自动化回归通过，阻断缺陷为 0
- 安全门禁：高危漏洞为 0，审计覆盖率 100%
- 性能门禁：达到文档/会议/提醒关键 P95 指标

