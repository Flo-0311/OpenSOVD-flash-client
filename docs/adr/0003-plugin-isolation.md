# ADR-0003: Plugin Isolation

## Status
Accepted

## Context
Plugins contain OEM-specific logic (security, backend integration, compliance). They must not leak into the open-source core or create tight coupling between components.

## Decision
Plugins are isolated via:

1. **Trait-based SPI**: Plugins implement well-defined Rust traits (`SecurityPlugin`, `BackendPlugin`, `WorkflowPlugin`, `ReportingPlugin`).
2. **Dynamic loading**: Plugins can be compiled as shared libraries (`.so`/`.dylib`/`.dll`) and loaded at runtime via `libloading`.
3. **No reverse dependencies**: The core never `use`s a specific plugin. Communication is strictly through trait interfaces.
4. **Separate build artifacts**: Plugins are built independently and distributed separately.

### Plugin Lifecycle
1. Discovery: Scan plugin directories
2. Loading: `dlopen` + symbol resolution
3. Registration: Plugin registers in the `PluginRegistry`
4. Execution: Core calls plugin traits at defined extension points
5. Unloading: Graceful shutdown on client exit

## Consequences

### Positive
- Clean separation of concerns
- Plugins can be proprietary without affecting the core's license
- Hot-reloading possible in future
- Testable via mock plugins

### Negative
- ABI stability requires careful version management
- Dynamic loading is `unsafe` in Rust
- Debugging across the FFI boundary is harder
- Plugin developers need Rust expertise

## Alternatives Considered
- **WASM plugins**: Better sandboxing but higher complexity and performance overhead
- **Process-based plugins**: Maximum isolation but high IPC cost
- **Script plugins (Lua/Python)**: Easy to write but poor type safety

We chose dynamic loading for performance and type safety, with WASM as a future option.
