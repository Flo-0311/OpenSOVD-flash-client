# ADR-0005: Mandatory Unit Test Coverage

## Status

Accepted

## Date

2026-03-14

## Context

The OpenSOVD Flash Client is safety-relevant software used for ECU diagnostics and flashing
in vehicle production and aftersales environments. Untested code paths in this context can
lead to bricked ECUs, incomplete flash operations, or silent data corruption.

As the codebase grows across multiple crates (`sovd-core`, `sovd-client`, `sovd-plugin`,
`sovd-workflow`, `sovd-observe`, `sovd-cli`), maintaining quality requires a systematic
approach to testing — not as an afterthought but as a mandatory design rule.

## Decision

**Every public module, function, struct, and enum MUST have corresponding unit tests.**

### Rules

1. **No code without tests.** Every source file containing logic (`.rs` files with `pub` items)
   MUST contain a `#[cfg(test)] mod tests { ... }` block exercising all public API surface.

2. **Test categories required per module:**
   - **Construction / Default**: Verify `new()`, `default()`, and builder patterns.
   - **Serialization roundtrip**: All `Serialize + Deserialize` types must survive JSON (and TOML
     where applicable) roundtrips with field-level assertions.
   - **Business logic**: All methods with branching logic (filters, state transitions, validations)
     must have positive and negative test cases.
   - **Error paths**: All `Result`-returning functions must test both `Ok` and `Err` variants.
   - **Display / Debug**: Types implementing `Display` must verify formatted output.

3. **CI enforcement.** The CI pipeline MUST run `cargo test --workspace` and fail the build
   on any test failure. No PR may be merged with failing tests.

4. **Coverage gate.** When tooling permits (e.g. `cargo-llvm-cov`), a minimum coverage
   threshold of **80%** per crate SHOULD be enforced. The threshold MUST NOT decrease
   between releases.

5. **Test naming convention.** Test functions use `snake_case` names that describe the
   scenario, e.g. `cancel_completed_job_fails`, `component_type_defaults_on_missing_field`.

6. **Async tests.** Async code MUST be tested using `#[tokio::test]`.

7. **No test pollution.** Tests MUST NOT depend on external services, network access, or
   mutable global state. Use in-memory constructs and mock data.

### Scope

| Crate | Minimum test areas |
|---|---|
| `sovd-core` | All models (Component, Capability, Job, DataTypes), error types, serde |
| `sovd-client` | Client construction, URL building, CapabilityResolver logic |
| `sovd-plugin` | SPI types, PluginRegistry CRUD, PluginManager lifecycle |
| `sovd-workflow` | StateMachine transitions (valid + invalid), JobController CRUD + events |
| `sovd-observe` | EventRecorder CRUD, ReportGenerator output, file I/O |
| `sovd-cli` | Config defaults + serde, OutputFormat formatting |

## Consequences

### Positive

- **Regression safety**: Every change is validated against existing behavior.
- **Documentation by example**: Tests serve as living documentation for each API.
- **Refactoring confidence**: Comprehensive tests enable safe restructuring.
- **CI gatekeeping**: No untested code reaches `main`.

### Negative

- **Development overhead**: Writing tests adds time to each feature/fix.
- **Test maintenance**: Model changes require updating corresponding tests.
- **CI duration**: More tests increase build time (~acceptable for safety-critical software).

## Compliance

Violations of this ADR (modules without tests, PRs reducing coverage) MUST be flagged
in code review and blocked from merging.

## Related

- [ADR-0001: Open Core](0001-open-core.md)
- [ADR-0002: Capability-Driven Workflows](0002-capability-driven-workflows.md)
- [ADR-0003: Plugin Isolation](0003-plugin-isolation.md)
- [ADR-0004: SOVD-Only Communication](0004-sovd-only-communication.md)
