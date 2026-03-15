# ADR-0004: SOVD-Only Communication (CDA Transparency)

## Status
Accepted

## Context
The OpenSOVD Flash Client must communicate with both:
- **Native SOVD HPCs** (e.g. AUTOSAR Adaptive, Eclipse S-CORE based systems)
- **Classic UDS ECUs** (legacy ECUs speaking UDS/ISO 14229 via DoIP or CAN)

Legacy tools like DTS Monaco implemented both SOVD and UDS protocols directly, leading to protocol-specific code paths, complex session management, and tight coupling to transport layers.

The Eclipse OpenSOVD ecosystem provides a **Classic Diagnostic Adapter (CDA)** — a server-side Rust component that translates SOVD REST calls to UDS/DoIP using diagnostic descriptions (MDD/ODX). This means the client does not need to implement UDS.

## Decision
The OpenSOVD Flash Client communicates **exclusively via SOVD REST APIs (ISO 17978)**.

- **No UDS, DoIP, CAN, or any other lower-level protocol is implemented in the client.**
- The SOVD Gateway routes requests to the appropriate backend:
  - Native SOVD HPCs → direct SOVD forwarding
  - Classic UDS ECUs → Classic Diagnostic Adapter (CDA) → UDS/DoIP
- The client treats all components identically via the SOVD API.
- `ComponentType` (NativeSovd / ClassicUds) is exposed as **informational metadata only** — it has no effect on client behavior or API calls.

### Communication Flow

```
Client ──SOVD REST──▶ SOVD Server ──▶ SOVD Gateway ──┬──▶ Native HPC (SOVD)
                                                      └──▶ CDA ──UDS/DoIP──▶ Classic ECU
```

The client only sees the leftmost arrow. Everything else is transparent.

## Consequences

### Positive
- **Radical simplicity**: One protocol, one code path, one API for all ECUs
- **No protocol expertise required**: Client developers don't need UDS/DoIP knowledge
- **Leverages OpenSOVD CDA**: No need to reimplement SOVD→UDS translation
- **Future-proof**: New ECU types only need a server-side adapter, not a client change
- **Testability**: Client can be tested against any SOVD-compliant mock server
- **Reduced attack surface**: No raw protocol handling in the client

### Negative
- **Dependency on CDA**: Classic ECU access requires a running CDA instance
- **Latency**: Additional hop through CDA for classic ECUs (mitigated by CDA's async Rust design)
- **Reduced visibility**: Client cannot inspect raw UDS frames (by design — this is a feature, not a bug)
- **Server infrastructure required**: Unlike legacy tools, cannot work standalone against a bare ECU

### Why this is correct
The SOVD standard (ISO 17978) explicitly defines this architecture. The CDA is the canonical way to integrate classic ECUs. Implementing UDS in the client would:
1. Violate the SOVD architecture
2. Duplicate CDA functionality
3. Require maintaining ODX/MDD parsing (already done by odx-converter)
4. Create two code paths that must be kept in sync

## Related
- Eclipse OpenSOVD CDA: https://github.com/eclipse-opensovd/classic-diagnostic-adapter
- Eclipse OpenSOVD ODX Converter: https://github.com/eclipse-opensovd/odx-converter
- Eclipse OpenSOVD Design: https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md
- ADR-0001: Open Core
- ADR-0002: Capability-Driven Workflows
