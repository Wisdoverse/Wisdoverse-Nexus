# API Versioning Strategy

The current gateway routes use `/v1/*` URL paths. Wisdoverse Nexus is still
pre-1.0, so `/v1` means "current active API", not a long-term compatibility
guarantee.

## Current Version

| Version | Status |
| --- | --- |
| `v1` | Active pre-1.0 API |

## URL Path Scheme

```text
/v1/rooms
/v1/messages
/v1/search
```

Future incompatible API surfaces should use a new path prefix, for example
`/v2`.

## Breaking Changes

Breaking changes include:

- Removing endpoints
- Changing request or response structure
- Changing authentication behavior
- Changing error formats
- Renaming fields
- Changing WebSocket message formats

Non-breaking changes include:

- Adding optional fields
- Adding optional query parameters
- Adding new endpoints
- Improving performance or validation without rejecting previously valid input

## Migration Guide

No `v2` API is published yet. When a new API version is introduced, this page
must include endpoint-level migration notes, SDK compatibility notes, and removal
timelines for deprecated `v1` behavior.

## Client Guidance

- Pin SDK versions during production evaluation.
- Read [CHANGELOG.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/CHANGELOG.md) before upgrading.
- Validate against `/openapi.json` when integrating directly with the gateway.
- Avoid depending on undocumented response fields.
