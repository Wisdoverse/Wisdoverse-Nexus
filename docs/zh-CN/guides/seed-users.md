# 种子用户计划

## 目标

- **首批用户**: 100 人
- **反馈率**: >60%
- **社区参与**: 建立 early adopter 社区

## 策略

### 1. 开发者社区推广

- **定向邀请**: Rust/TypeScript 社区 KOL
- **激励计划**: 早期用户终身免费 Pro 版
- **推荐奖励**: 每推荐 1 个活跃用户 = 1 个月免费

### 2. Product Hunt 发布

**发布前 (2周)**
- [ ] 准备产品截图和 Demo 视频
- [ ] 撰写 Hunter's Note
- [ ] 联系 Hunter 帮助推

**发布日**
- [ ] 00:01 PST 发布
- [ ] 团队全员 upvote
- [ ] 社交媒体同步推送
- [ ] 回复所有评论

### 3. Hacker News 策略

- **标题**: "Show HN: Nexis – An open-source real-time collaboration server with AI teammates"
- **时间**: 周二 9:00 AM PST
- **要点**: 强调开源 + AI + 100K 连接

### 4. 社交媒体内容日历

| 周 | 主题 | 平台 |
|----|------|------|
| 1 | 项目发布 | Twitter, LinkedIn |
| 2 | 技术深挖 | Twitter, Dev.to |
| 3 | 用户故事 | Twitter, LinkedIn |
| 4 | 开源贡献 | GitHub, Twitter |
| 5 | 插件生态 | Dev.to, Medium |
| 6 | 路线图更新 | Twitter, LinkedIn |

## 用户旅程

```
注册 → Welcome 房间 → AI 问候 → 发送第一条消息 → AHA Moment
  ↓         ↓           ↓            ↓              ↓
验证邮箱  自动加入    智能引导      庆祝动画      功能发现
```

### AHA Moment 设计

- 用户发送 3+ 条消息后触发庆祝动画
- AI 自动总结对话要点
- 展示"你的对话已被 AI 理解"提示

## 反馈收集

### NPS 调查

触发时机: 用户发送第 10 条消息后

```
问题: "你有多大可能向朋友推荐 Nexis？"
评分: 0-10
追问: "为什么给出这个分数？"
```

### 用户访谈模板

- **时长**: 20-30 分钟
- **问题**:
  1. 你是如何发现 Nexis 的？
  2. 你主要用它做什么？
  3. 最喜欢的功能是什么？
  4. 最不喜欢的功能是什么？
  5. 缺少什么功能？

### Beta 反馈表单

- GitHub Issues (bug 报告)
- Discord 社区 (功能讨论)
- Typeform (详细反馈)

## 指标仪表盘

| 指标 | 目标 | 追踪方式 |
|------|------|----------|
| Day 1 留存 | >60% | 注册后 24h 活跃 |
| Day 7 留存 | >40% | 注册后 7d 活跃 |
| Day 30 留存 | >25% | 注册后 30d 活跃 |
| NPS 分数 | >40 | 季度调查 |
| 消息数/用户/天 | >10 | Prometheus |
| WebSocket 连接峰值 | 追踪 | Prometheus |

## 时间线

| 周 | 活动 |
|----|------|
| 1 | 内部测试 + 文档完善 |
| 2 | Alpha 测试 (10 人) |
| 3-4 | Beta 测试 (50 人) |
| 5 | Product Hunt 发布 |
| 6-8 | 公开 Beta + 社区建设 |
