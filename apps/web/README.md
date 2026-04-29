# Wisdoverse Nexus Web App

React + Vite client for Wisdoverse Nexus.

## Requirements

- Node.js `>=20.19.0`
- pnpm `>=10.30.0`

## Development

Run from the repository root:

```bash
pnpm install --frozen-lockfile --ignore-scripts
pnpm --filter @wisdoverse/nexus-web dev
```

The app runs on `http://localhost:3000` and proxies API/WebSocket traffic to the
local gateway on `localhost:8080`.

## Verification

```bash
pnpm --filter @wisdoverse/nexus-web build
```

## Main Paths

| Path | Purpose |
| --- | --- |
| `src/pages/` | Route-level views |
| `src/components/` | Reusable UI components |
| `src/features/` | Feature modules |
| `src/app/` | Application shell and layout |
| `src/shared/` | Shared API, styles, UI, and WebSocket helpers |
