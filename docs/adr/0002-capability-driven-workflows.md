# ADR-0002: Capability-Driven Workflows

## Status
Accepted

## Context
SOVD servers expose varying sets of capabilities depending on configuration, licensing, and the connected vehicle. The client must adapt dynamically rather than assuming a fixed feature set.

## Decision
All workflows are **capability-driven**:

1. On connection, the client discovers capabilities via `GET /sovd/v1/capabilities`.
2. The `CapabilityResolver` interprets the result and determines available operations.
3. UI and CLI commands check capability availability before execution.
4. Missing capabilities produce clear error messages, not crashes.

## Consequences

### Positive
- Works with any SOVD-compliant server regardless of feature set
- Graceful degradation when capabilities are unavailable
- No hardcoded assumptions about server behavior
- Supports forward compatibility with future SOVD extensions

### Negative
- Slightly more complex initialization flow
- UI must handle dynamic feature availability
- Testing requires capability fixtures

## Related
- ADR-0001: Open Core
