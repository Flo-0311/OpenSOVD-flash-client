# ADR-0001: Open Core Principle

## Status
Accepted

## Context
The OpenSOVD Flash Client aims to replace proprietary monolithic tools like DTS Monaco. To foster adoption and community contribution, the core must be open source. However, OEM-specific functionality (authentication, backend integration, compliance) must remain protected.

## Decision
We adopt the **Open Core** model:

- The **core** (sovd-core, sovd-client, sovd-workflow, sovd-observe, sovd-cli) is fully open source under Apache-2.0.
- **Differentiation** is implemented exclusively via plugins that can be proprietary.
- The core never imports or depends on any plugin implementation.
- The plugin SPI (Service Provider Interface) is stable and public.

## Consequences

### Positive
- Open collaboration on the foundation
- OEM IP remains protected in plugins
- Community can contribute improvements to the core
- No vendor lock-in for the base tooling

### Negative
- Plugin SPI must be carefully versioned
- Dynamic loading introduces ABI stability concerns
- Core must be feature-complete enough to be useful without plugins

## Related
- ADR-0003: Plugin Isolation
