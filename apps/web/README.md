# @wisdoverse/nexus-web

Wisdoverse Nexus Web UI — AI-powered team chat client.

## Tech Stack

- **React 18** + TypeScript
- **Vite** — dev server & bundler
- **TailwindCSS** — utility-first styling
- **@wisdoverse/nexus-sdk** — official Wisdoverse Nexus client SDK

## Getting Started

```bash
cd apps/web
pnpm install
pnpm dev
```

Opens at `http://localhost:3000`. API & WS requests proxy to `localhost:8080`.

## Project Structure

```
src/
├── pages/          # Route-level views
│   ├── Login.tsx       — Email/password login
│   ├── Register.tsx    — Sign up → auto Welcome room + AI greeting
│   ├── Home.tsx        — Lobby: sidebar + room list
│   ├── Room.tsx        — Chat room: messages + input + members
│   └── Settings.tsx    — User profile & preferences
├── components/     # Reusable UI
│   ├── Sidebar.tsx      — Collapsible sidebar shell
│   ├── MessageList.tsx  — Message feed with AI badges & actions
│   ├── MessageInput.tsx — Text input with send
│   ├── MemberList.tsx   — Online members panel (human + AI)
│   └── RoomList.tsx     — User's rooms + create button
├── hooks/          # React hooks
│   ├── useAuth.ts       — Login/logout/register, user state
│   ├── useWebSocket.ts  — WS connection, reconnect, event dispatch
│   └── useRoom.ts       — Room CRUD, message send/receive
└── App.tsx         — Router + auth guard
```

## AHA Moment Flow

1. User registers → backend creates **"Welcome"** room + adds AI member
2. AI member sends welcome message with feature highlights
3. User sends first message → AI replies with personalized greeting
4. AI messages show "✨ Summarize" action button
5. User immediately understands: *this is chat, but smarter*

## Design Tokens

- **Brand color:** Indigo `#6366f1` → `#4f46e5`
- **AI accent:** `brand-50` background, `brand-600` badges
- **Rounding:** `rounded-xl` (cards), `rounded-lg` (inputs/buttons)
- **Shadows:** `shadow-lg` (cards), `shadow-sm` (inputs)
