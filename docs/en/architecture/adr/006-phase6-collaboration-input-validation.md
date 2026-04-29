# ADR-006: Phase 6 Collaboration Input Validation

## Status

Accepted

## Context

Phase 6 collaboration endpoints introduced multiple new write APIs.
Without shared validation rules, endpoints can diverge on:
- Allowed identifier formats
- Required field semantics
- Maximum payload sizes
- Time-range validation for calendar operations

Inconsistent validation increases security risk and causes unpredictable client behavior.

## Decision

Adopt shared input-validation helpers in the collaboration API layer and enforce:
- Required trimmed string fields
- Length limits for names, titles, identifiers, and content
- Strict identifier character policy (`a-z`, `A-Z`, `0-9`, `_`, `-`)
- Valid calendar time windows (`starts_at < ends_at`)

## Consequences

### Positive
- Consistent `400 BAD_REQUEST` behavior across collaboration endpoints
- Reduced attack surface from malformed input
- Better operational signals and easier client-side handling

### Negative
- Existing clients with lax payload formatting may need adjustment
- Validation changes require explicit versioning discipline

### Mitigation
- Keep error messages explicit and stable
- Cover validation rules with endpoint tests
