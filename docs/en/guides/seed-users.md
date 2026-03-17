# Seed User Plan

## Goals

- **First 100 users**: 100 people
- **Feedback rate**: >60%
- **Community**: Build early adopter community

## Strategy

### 1. Developer Community Outreach

- **Targeted invites**: Rust/TypeScript community KOLs
- **Incentives**: Lifetime free Pro for early users
- **Referral rewards**: 1 month free per active referral

### 2. Product Hunt Launch

**Pre-launch (2 weeks)**
- [ ] Prepare screenshots and demo video
- [ ] Write Hunter's Note
- [ ] Contact Hunters for support

**Launch Day**
- [ ] Launch at 00:01 PST
- [ ] Team upvotes
- [ ] Social media sync
- [ ] Reply to all comments

### 3. Hacker News Strategy

- **Title**: "Show HN: Nexis – An open-source real-time collaboration server with AI teammates"
- **Timing**: Tuesday 9:00 AM PST
- **Key points**: Open source + AI + 100K connections

### 4. Social Media Calendar

| Week | Theme | Platform |
|------|-------|----------|
| 1 | Launch | Twitter, LinkedIn |
| 2 | Tech Deep Dive | Twitter, Dev.to |
| 3 | User Stories | Twitter, LinkedIn |
| 4 | Open Source | GitHub, Twitter |
| 5 | Plugin Ecosystem | Dev.to, Medium |
| 6 | Roadmap Update | Twitter, LinkedIn |

## User Journey

```
Sign Up → Welcome Room → AI Greeting → First Message → AHA Moment
   ↓          ↓            ↓              ↓              ↓
Verify     Auto-join    Smart guide    Celebration    Discovery
```

### AHA Moment Design

- Celebration animation after 3+ messages
- AI auto-summarizes conversation
- Show "Your conversation is understood by AI" prompt

## Feedback Collection

### NPS Survey

Trigger: After 10th message

```
Question: "How likely are you to recommend Nexis to a friend?"
Score: 0-10
Follow-up: "Why did you give this score?"
```

### User Interview Template

- **Duration**: 20-30 minutes
- **Questions**:
  1. How did you discover Nexis?
  2. What do you mainly use it for?
  3. What's your favorite feature?
  4. What's your least favorite feature?
  5. What's missing?

### Beta Feedback Form

- GitHub Issues (bugs)
- Discord community (feature discussions)
- Typeform (detailed feedback)

## Metrics Dashboard

| Metric | Target | Tracking |
|--------|--------|----------|
| Day 1 Retention | >60% | Active within 24h |
| Day 7 Retention | >40% | Active within 7d |
| Day 30 Retention | >25% | Active within 30d |
| NPS Score | >40 | Quarterly survey |
| Messages/User/Day | >10 | Prometheus |
| Peak WebSocket Connections | Track | Prometheus |

## Timeline

| Week | Activity |
|------|----------|
| 1 | Internal testing + docs polish |
| 2 | Alpha testing (10 users) |
| 3-4 | Beta testing (50 users) |
| 5 | Product Hunt launch |
| 6-8 | Public beta + community building |
