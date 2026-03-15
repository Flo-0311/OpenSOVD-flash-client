# ADR-0006: GUI Technology — Tauri 2.0 + React / TypeScript

## Status

Accepted

## Context

The OpenSOVD Flash Client needs a graphical desktop application to replace
legacy monolithic tools (e.g. DTS Monaco) for interactive diagnostics and
flashing. The GUI must:

- Integrate natively with the existing Rust crates (sovd-client, sovd-workflow, etc.)
- Run cross-platform (Windows 10+, macOS 12+, Ubuntu 22.04+)
- Be lightweight enough for factory flashing stations
- Provide a modern, professional UX for automotive engineers
- Support real-time updates (flash progress, live monitoring)
- Remain open-source compatible (Eclipse ecosystem, no GPL conflicts)

## Decision

We use **Tauri 2.0** as the desktop application framework with a
**React + TypeScript** frontend.

### Frontend Stack

| Component | Technology | Rationale |
|---|---|---|
| Language | TypeScript 5.x | Static typing matches Rust philosophy |
| Framework | React 18.x | Component-based, large ecosystem |
| Styling | TailwindCSS 3.x | Utility-first, Dark/Light mode support |
| Components | shadcn/ui | Accessible, customizable, no vendor lock-in |
| State | Zustand 4.x | Lightweight, TypeScript-native |
| Data Fetching | TanStack Query 5.x | Caching, auto-refresh, retry |
| Tables | TanStack Table 8.x | Sortable, filterable data grids |
| Charts | Recharts 2.x | Monitoring sparklines, progress charts |
| i18n | i18next 23.x | German + English initial |
| Icons | Lucide React | Consistent, MIT-licensed icon set |
| Build | Vite 5.x | Fast dev server, HMR |

### Backend Integration

The Tauri Rust backend directly depends on the existing workspace crates:

```toml
[dependencies]
sovd-core     = { path = "../../crates/sovd-core" }
sovd-client   = { path = "../../crates/sovd-client" }
sovd-workflow  = { path = "../../crates/sovd-workflow" }
sovd-plugin   = { path = "../../crates/sovd-plugin" }
sovd-observe  = { path = "../../crates/sovd-observe" }
tauri         = { version = "2", features = ["shell-open"] }
```

Rust functions are exposed as `#[tauri::command]` and called from TypeScript
via `invoke()`. Real-time updates (flash progress, logs) use Tauri events.

## Alternatives Considered

### Electron

- (+) Mature ecosystem, proven in VS Code, Postman
- (−) ~150 MB binary (unacceptable for factory stations)
- (−) Bundles Chromium + Node.js (security surface, resource usage)
- (−) Rust integration requires Node.js native modules (napi-rs) or HTTP bridge

### egui / eframe (Pure Rust)

- (+) Same language as backend, no web stack needed
- (+) Small binary (~5 MB)
- (−) Limited UI component ecosystem (no rich tables, charts, forms)
- (−) Immediate-mode rendering not ideal for complex forms and wizards
- (−) Accessibility (WCAG) not well supported
- (−) Significant custom work for professional automotive UX

### Qt (C++ / QML)

- (+) Mature widget toolkit, good for desktop apps
- (−) GPL license conflicts with Eclipse open-source model (commercial license expensive)
- (−) Rust-Qt bindings (cxx-qt) still maturing
- (−) Separate C++/QML build chain adds complexity

### Slint (Rust-native declarative UI)

- (+) Rust-native, declarative, good performance
- (−) Smaller ecosystem than React
- (−) Community license restricts some uses
- (−) Less talent pool for contributors

## Consequences

### Positive

- **Zero code duplication** — GUI and CLI share identical Rust crates
- **~10 MB binary** — 15x smaller than Electron, suitable for factory stations
- **Native OS integration** — uses system webview (WebView2/WebKit), no bundled browser
- **Type-safe IPC** — serde (Rust) ↔ TypeScript interfaces, compile-time guarantees
- **Rich UI ecosystem** — access to entire React/npm ecosystem for components
- **Security** — no Node.js runtime, Content Security Policy enforced by Tauri
- **Rapid iteration** — Vite HMR for frontend, `cargo watch` for backend

### Negative

- **Two build systems** — Cargo (Rust) + npm (TypeScript), but Tauri CLI orchestrates both
- **WebView dependency** — Linux requires WebKitGTK system package
- **JavaScript runtime in webview** — not as performant as native rendering for extreme cases
- **Team needs both Rust and TypeScript skills**

### Risks

- Tauri 2.0 is newer than Electron; smaller community (mitigated: backed by CrabNebula, growing rapidly)
- WebView2 must be installed on Windows 10 (pre-installed on Windows 11)

## References

- [Tauri 2.0 Documentation](https://v2.tauri.app/)
- [ADR-0001: Open Core](0001-open-core.md)
- [ARCHITECTURE.md — GUI Architecture](../../ARCHITECTURE.md#gui-architecture)
