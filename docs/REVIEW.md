# Architecture & Code Review — OpenSOVD Flash Client

**Reviewer**: Senior Automotive Software Engineer (Rust)
**Date**: 2026-03-14
**Scope**: Full codebase, architecture, code quality, ecosystem redundancy

---

## Executive Summary

The OpenSOVD Flash Client is architecturally sound: clean crate separation,
strict SOVD-only communication (ADR-0004), capability-driven design, and a
well-defined plugin SPI. The 210+ tests and clippy-pedantic compliance are
solid. However, there are **12 actionable findings** across architecture,
code quality, security, and ecosystem alignment that should be addressed
before production readiness.

**Overall rating**: 🟢 Good foundation, 🟡 production-hardening needed.

---

## 1. Architecture Review

### 1.1 Crate Structure — ✅ Good

The 6-crate workspace follows clean layering:

```
sovd-core  ←  sovd-client  ←  sovd-workflow  ←  sovd-cli
                                    ↑
sovd-plugin ─────────────────────────┘
sovd-observe ────────────────────────┘
```

**Strengths**:
- `sovd-core` is dependency-free (except serde/chrono/uuid) — good for reuse.
- `sovd-client` only depends on `sovd-core` + `reqwest` — minimal surface.
- Plugin SPI in its own crate, separating extension from core.
- Observe crate separates cross-cutting concerns.

**Finding F-01 (Medium): Circular conceptual coupling between `sovd-workflow` and `sovd-client`**

`JobController::execute_flash()` directly calls `SovdClient` methods and
embeds SOVD-specific polling logic (status checks, progress parsing).
The `WorkflowEngine` holds both `SovdClient` and `JobController`.

This means the workflow crate has deep knowledge of the REST API's JSON
response structure (e.g., `val.get("state").and_then(|s| s.as_str())`
in `controller.rs:120-133`). This fragile coupling breaks when the SOVD
server changes response formats.

**Recommendation**: Introduce a `FlashService` trait in `sovd-core` that
`SovdClient` implements. `JobController` should depend on the trait, not
the concrete client. This also enables testing the workflow without
`wiremock`.

### 1.2 SOVD Compliance — ✅ Correct

The fundamental principle is maintained: the client speaks 100% SOVD REST.
No UDS, DoIP, or CAN code exists. `ComponentType` is correctly documented
as informational metadata only.

**Finding F-02 (Low): SOVD API paths are hardcoded strings**

All API paths (`/sovd/v1/components`, `/sovd/v1/capabilities`, etc.) are
scattered as string literals across `client.rs`. If the SOVD spec version
changes (e.g., `/sovd/v2/`), every path must be updated manually.

**Recommendation**: Centralize API path construction:
```rust
mod api_paths {
    pub const API_VERSION: &str = "v1";
    pub fn components() -> String { format!("/sovd/{API_VERSION}/components") }
    pub fn component(id: &str) -> String { format!("/sovd/{API_VERSION}/components/{id}") }
    // ...
}
```

### 1.3 Capability-Driven Design — ⚠️ Partial

`CapabilityResolver` is well-designed, but capability checks are **not
enforced before API calls**. For example, `SovdClient::read_config()`,
`get_live_data()`, `get_logs()` etc. make HTTP calls without checking if
the server declared those capabilities.

Only `WorkflowEngine::flash()` checks `supports_flashing()`.

**Finding F-03 (High): Missing capability enforcement on API calls**

Per ADR-0002 (Capability-Driven Workflows), features should be gated by
discovered capabilities. Currently, `read_config`, `get_live_data`,
`get_logs`, `subscribe_logs` all fire blindly.

**Recommendation**: Either:
- Add `require_capability()` guards in `SovdClient` methods, or
- Document that capability checking is the caller's responsibility and
  add checks in the CLI commands.

---

## 2. Code Quality

### 2.1 Error Handling — 🟡 Adequate, improvable

**Finding F-04 (Medium): String-based error classification in `main.rs`**

Exit code determination uses string matching:
```rust
// main.rs:344
if e.to_string().contains("Invalid") || e.to_string().contains("Config") {
    EXIT_CONFIG
} else {
    EXIT_ERROR
};
```

This is fragile — any error message containing "Invalid" gets EXIT_CONFIG,
even if it's not a config error.

**Recommendation**: Pattern-match on `SovdError` variants directly.
`anyhow` supports downcasting:
```rust
if let Some(sovd_err) = e.downcast_ref::<SovdError>() {
    match sovd_err {
        SovdError::Config(_) => EXIT_CONFIG,
        _ => EXIT_ERROR,
    }
} else { EXIT_ERROR }
```

### 2.2 Retry Logic — ✅ Correct, but duplicated

**Finding F-05 (Medium): Massive code duplication in HTTP methods**

`get()`, `post()`, `put()`, `delete()` in `client.rs` each contain
~40 lines of nearly identical retry logic. This violates DRY and makes
maintenance error-prone.

**Recommendation**: Extract a generic retry helper:
```rust
async fn with_retry<F, Fut, T>(&self, method: &str, url: &str, f: F) -> SovdResult<T>
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<Response, reqwest::Error>>,
{ /* shared retry loop */ }
```

### 2.3 Async Patterns — ⚠️ Issues

**Finding F-06 (High): `std::env::set_var` in async context (unsound)**

```rust
// main.rs:332
if cli.verbose {
    std::env::set_var("RUST_LOG", "debug");
}
```

`std::env::set_var` is **not thread-safe** and is deprecated as of
Rust 1.66+ (will be `unsafe` in future editions). In an async `tokio`
runtime, this is UB if another thread reads env vars concurrently.

**Recommendation**: Use `tracing_subscriber::EnvFilter::builder()` to
set the level programmatically, or set the env var *before* starting the
tokio runtime.

### 2.4 Memory Safety — ⚠️ Plugin FFI

**Finding F-07 (Medium): `Box::leak` in test mock manifests**

Engine tests in `engine.rs` use `Box::leak(Box::new(PluginManifest{..}))`
to create `&'static` references from `manifest()`. This leaks memory
on every test call.

While acceptable in tests, if this pattern is copied to production
plugin code, it becomes a real leak.

**Recommendation**: Store the manifest as an owned field in the plugin
struct (as done correctly in `manager.rs` tests) and return `&self.manifest`.

---

## 3. Redundancy Check with Eclipse OpenSOVD Ecosystem

### 3.1 Ecosystem Map

| Eclipse OpenSOVD Component | Repo | Overlap with Flash Client |
|---|---|---|
| **SOVD Client** | [opensovd design](https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md) | ⚠️ **Direct overlap** — OpenSOVD defines "SOVD Client" as an in-scope component |
| **Classic Diagnostic Adapter** | [eclipse-opensovd/classic-diagnostic-adapter](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) | ✅ No overlap — server-side, transparent to client |
| **Fault Library** | [eclipse-opensovd/fault-lib](https://github.com/eclipse-opensovd/fault-lib) | ⚠️ **DTC model overlap** — both define DTC structures |
| **ODX Converter** | [eclipse-opensovd/odx-converter](https://github.com/eclipse-opensovd/odx-converter) | ✅ No overlap — Kotlin, server-side |
| **UDS2SOVD Proxy** | [eclipse-opensovd/uds2sovd-proxy](https://github.com/eclipse-opensovd/uds2sovd-proxy) | ✅ No overlap — server-side |
| **SOVD Server / Gateway** | [opensovd design](https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md) | ✅ No overlap — server-side |

### 3.2 Key Redundancy Findings

**Finding F-08 (High): SOVD Client is defined as in-scope in OpenSOVD**

The [OpenSOVD design document](https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md)
explicitly lists "SOVD Client" as an in-scope component:

> *"Off-board, on-board or cloud client that initiates diagnostics via
> SOVD protocol. Can be used by developers, testers, ECUs or cloud
> services; should be deployment agnostic."*

**Risk**: The Eclipse OpenSOVD project may develop its own generic SOVD
client library, which would overlap with `sovd-client` and parts of
`sovd-core`.

**Recommendation**:
1. **Engage early** with the OpenSOVD "Workstream Core (Server, Gateway
   & Client)" — Tuesdays 11:30–12:15 CET — to coordinate.
2. **Propose** this Flash Client's `sovd-client` crate as the reference
   SOVD client implementation, or align models with their emerging API.
3. **Clearly scope** this project as a *flash-focused tool* built *on top
   of* the OpenSOVD client SDK, not a replacement for it.

**Finding F-09 (Medium): DTC / Fault model duplication with `fault-lib`**

The `DiagnosticTroubleCode` struct in `sovd-core/src/model/datatypes.rs`
defines DTC fields (`id`, `code`, `status`, `severity`, `component_id`).

The OpenSOVD [Fault Library](https://github.com/eclipse-opensovd/fault-lib)
and [Diagnostic DB design](https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md)
define their own DTC/Fault data format including:
- Diagnostic Trouble Code (DTC): OEM-specific code
- Fault ID (FID): ECU-specific ID
- Count: occurrence count
- Meta data

These will likely converge into a shared schema. Maintaining a parallel
DTC model in this client risks **schema drift**.

**Recommendation**:
1. Track the `fault-lib` crate's DTC types as they stabilize.
2. Once available, depend on `fault-lib` types or a shared `sovd-types`
   crate instead of maintaining local `DiagnosticTroubleCode`.
3. For now, add `// TODO: Align with eclipse-opensovd/fault-lib once DTC schema stabilizes`
   to `datatypes.rs`.

**Finding F-10 (Low): Capability categories should align with SOVD spec**

The `CapabilityCategory` enum includes `Bulk` and `Other(String)` which
are not part of ISO 17978. The OpenSOVD design lists specific service
categories (diagnostics, logging, software updates, fault management).

**Recommendation**: Document which categories are standard SOVD vs.
extension. Use `#[serde(untagged)]` on `Other` only if the SOVD spec
allows arbitrary extensions.

---

## 4. Security Review

**Finding F-11 (High): Auth token stored in plaintext in memory**

`SovdClient.auth_token: Option<String>` holds the bearer token as a
plain `String`. If the process core-dumps or is attached by a debugger,
the token is trivially extractable.

The OpenSOVD design explicitly states:
> *"The client lib(s) need to be developed with the same quality
> standards as safe components [...] and also provide FFI guarantees."*

**Recommendation**:
1. Use `secrecy::Secret<String>` from the `secrecy` crate to wrap the
   token. This prevents accidental logging/serialization and zeroizes
   on drop.
2. Alternatively, mark with `#[doc(hidden)]` and ensure the field is
   never included in `Debug` output.

**Finding F-12 (Medium): Dynamic plugin loading has no signature verification**

`PluginManager::load_dynamic()` calls `libloading::Library::new()` on
arbitrary `.so`/`.dylib` files with a doc comment *"Only load plugins
from trusted sources"*. There is no integrity check.

The OpenSOVD design emphasizes:
> *"Handles access control on the client side – e.g. by providing
> relevant certificates."*

**Recommendation**: Before loading, verify plugin checksums or code
signatures via a `SecurityPlugin::verify_signature()` call. At minimum,
log the full path and hash of loaded libraries for audit trails.

---

## 5. Summary of Findings

| ID | Severity | Category | Finding |
|----|----------|----------|---------|
| F-01 | Medium | Architecture | Tight coupling: workflow embeds client JSON parsing |
| F-02 | Low | Architecture | SOVD API paths hardcoded as string literals |
| F-03 | **High** | Architecture | Missing capability enforcement on API calls |
| F-04 | Medium | Code Quality | String-based error classification for exit codes |
| F-05 | Medium | Code Quality | Duplicated retry logic across 4 HTTP methods |
| F-06 | **High** | Code Quality | `std::env::set_var` in async context (unsound) |
| F-07 | Medium | Code Quality | `Box::leak` pattern in test plugin manifests |
| F-08 | **High** | Ecosystem | SOVD Client overlap with OpenSOVD in-scope component |
| F-09 | Medium | Ecosystem | DTC model duplication with `fault-lib` |
| F-10 | Low | Ecosystem | Capability categories not fully aligned with ISO 17978 |
| F-11 | **High** | Security | Auth token in plaintext `String` |
| F-12 | Medium | Security | No signature verification for dynamic plugins |

### Priority Action Items

1. **Immediate** (F-06): Move `set_var` before tokio runtime or use programmatic filter.
2. **Immediate** (F-11): Wrap auth token with `secrecy::Secret<String>`.
3. **Short-term** (F-03): Add capability guards to API methods.
4. **Short-term** (F-04, F-05): Refactor exit codes and retry logic.
5. **Medium-term** (F-08, F-09): Engage with OpenSOVD workstream, align client/DTC models.
6. **Long-term** (F-01, F-12): Introduce trait abstraction, plugin signing.

---

## 6. Eclipse OpenSOVD Ecosystem Links

| Component | Repository | Relevance |
|---|---|---|
| Main Design | [eclipse-opensovd/opensovd](https://github.com/eclipse-opensovd/opensovd) | Architecture, SOVD Client scope definition |
| Design Document | [docs/design/design.md](https://github.com/eclipse-opensovd/opensovd/blob/main/docs/design/design.md) | In-scope component list, security/safety |
| Classic Diagnostic Adapter | [eclipse-opensovd/classic-diagnostic-adapter](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) | Server-side SOVD→UDS, no client overlap |
| Fault Library | [eclipse-opensovd/fault-lib](https://github.com/eclipse-opensovd/fault-lib) | DTC types — align when stable |
| ODX Converter | [eclipse-opensovd/odx-converter](https://github.com/eclipse-opensovd/odx-converter) | No overlap (Kotlin, server-side) |
| UDS2SOVD Proxy | [eclipse-opensovd/uds2sovd-proxy](https://github.com/eclipse-opensovd/uds2sovd-proxy) | No overlap (server-side) |
| Workstream Core | Tuesdays 11:30–12:15 CET | Coordinate client scope |
| Architecture Board | Mondays 11:30–12:30 CET | Align on shared types |

---

## 7. What's Done Well

- **SOVD-only principle** is consistently enforced — no protocol leakage.
- **Plugin SPI** is clean: 4 typed traits, no downcasting needed.
- **State machine** is well-tested with exhaustive transition coverage.
- **210+ tests** across all 6 crates, clippy-pedantic clean.
- **ADRs** document key decisions; ARCHITECTURE.md with C4 diagrams is excellent.
- **CDA transparency** is correctly modeled — `ComponentType` is metadata, not routing.
- **Retry/backoff** is production-grade with configurable parameters.
- **CLI UX** is solid: interactive wizard, progress bars, bulk ops, consistent exit codes.
