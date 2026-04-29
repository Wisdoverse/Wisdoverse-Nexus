# API Versioning

Wisdoverse Nexus is pre-1.0. The current REST API uses `/v1/*` paths, but the
project has not yet published a long-term compatibility guarantee.

## Current API

| Version | Status | Notes |
| --- | --- | --- |
| `v1` | Active pre-1.0 API | May change before the first stable release |

## Versioning Rules

The repository uses URL path versioning for public REST routes:

```text
/v1/rooms
/v1/messages
/v1/search
```

When a future incompatible version is introduced, it should use a new path
prefix such as `/v2`.

## Breaking Changes

Before 1.0, breaking changes are allowed but should be documented in
[CHANGELOG.md](https://github.com/Wisdoverse/Wisdoverse-Nexus/blob/main/CHANGELOG.md) and pull request descriptions.

Examples of breaking changes:

- Removing an endpoint
- Changing an HTTP method
- Renaming request or response fields
- Changing authentication requirements
- Changing WebSocket message formats
- Removing accepted configuration variables

Non-breaking changes usually include:

- Adding optional response fields
- Adding optional query parameters
- Adding new endpoints
- Improving validation without changing accepted valid requests

## Deprecation Expectations

When practical, maintainers should:

1. Mark deprecated behavior in documentation.
2. Add warnings in release notes.
3. Keep old behavior for at least one minor release.
4. Provide migration examples before removal.

Security fixes may require a shorter timeline.

## Client Guidance

- Pin SDK versions or commit SHAs during production evaluation.
- Read changelog entries before upgrading.
- Run integration tests against `/openapi.json` or the gateway routes you use.
- Avoid depending on undocumented response fields.

## Future Stable Policy

Before a 1.0 release, the project should publish:

- Stable API compatibility rules.
- Deprecation and sunset timelines.
- SDK compatibility policy.
- Migration guide format.
- WebSocket message compatibility policy.
