# AGENTS.md

Guidelines for AI coding agents working on the OpenSOVD Flash Client.

## Project Overview

Rust workspace (6 crates + GUI) implementing a SOVD-native diagnostic and flash client.
The client communicates **exclusively via SOVD REST APIs** (ISO 17978) — no UDS, no DoIP.

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full C4-model documentation.

### Crate Map

| Crate | Path | Purpose |
|---|---|---|
| `sovd-core` | `crates/sovd-core/` | Types, models, error handling — no I/O |
| `sovd-client` | `crates/sovd-client/` | SOVD REST client (reqwest), capability discovery |
| `sovd-plugin` | `crates/sovd-plugin/` | Plugin SPI traits, registry, dynamic loading (libloading) |
| `sovd-workflow` | `crates/sovd-workflow/` | Workflow engine, job controller, state machine |
| `sovd-observe` | `crates/sovd-observe/` | Event recording, report generation, tracing setup |
| `sovd-cli` | `crates/sovd-cli/` | CLI binary `sovd-flash` (clap) |
| `sovd-gui` | `sovd-gui/src-tauri/` | Tauri 2.0 GUI backend (commands, state) |
| `example-plugin` | `plugins/example-plugin/` | Example cdylib plugin |

### GUI Frontend

- Path: `sovd-gui/src/`
- Stack: React 18 + TypeScript 5 + Vite 6 + TailwindCSS 3
- State: Zustand 5
- Icons: Lucide React
- IPC: Tauri `invoke()` commands + Tauri events
- Language: English only (no i18n framework)

## Key Constraints

1. **SOVD-Only** — Never add UDS/DoIP/KWP protocol handling to the client. The CDA handles translation server-side. (ADR-0004)
2. **Capability-Driven** — All operations must check server capabilities before execution. (ADR-0002)
3. **Open Core** — The core crates never import plugin implementations. Differentiation is plugins-only. (ADR-0001)
4. **Plugin Isolation** — Plugins are dynamically loaded cdylib crates. Core must not depend on any plugin. (ADR-0003)
5. **Mandatory Tests** — Every `.rs` file with `pub` items must have a `#[cfg(test)]` module. (ADR-0005)

## Build & Test

```bash
# Build everything
cargo build --workspace

# Run all tests (244 tests across all crates)
cargo test --workspace

# Clippy (warnings = errors in CI)
cargo clippy --workspace -- -D warnings

# Format check
cargo fmt --all -- --check

# GUI dev server
cd sovd-gui && npm run tauri dev
```

## Code Style Rules

- **Imports** at the top of every file, never inline
- **Error types**: `thiserror` for library errors, `anyhow` for application errors
- **Logging**: `tracing` macros only — never `println!` or `eprintln!` in library code
- **Display trait**: All user-facing enums must implement `Display` (not rely on `Debug`)
- **Serde**: Use `#[serde(rename_all = "snake_case")]` for API-facing enums
- **SecretString**: Auth tokens must use `secrecy::SecretString` for zeroize-on-drop
- **No comments/docs changes** unless explicitly requested
- **Minimal edits** — prefer targeted fixes over large refactors

## Dependency Policy

- All dependencies must be **Apache-2.0 compatible** (MIT, MIT/Apache-2.0, ISC, BSD)
- Justify new dependencies — prefer stdlib or existing deps
- Workspace dependencies are defined in the root `Cargo.toml`
- Sub-crates use `{ workspace = true }` references

## Testing Conventions

- Unit tests: inline `#[cfg(test)] mod tests { ... }` in each source file
- Integration tests: `tests/` directory (e.g., `sovd-client/tests/integration_mock.rs`)
- Mock server: `wiremock` for HTTP mocking in client tests
- CI enforces 80% code coverage minimum
- Run `scripts/check-tests.sh` locally before committing

## Architecture Decision Records

ADRs live in `docs/adr/` and follow the format `NNNN-title.md`:

- **0001** — Open Core Principle
- **0002** — Capability-Driven Workflows
- **0003** — Plugin Isolation
- **0004** — SOVD-Only Communication
- **0005** — Mandatory Test Coverage
- **0006** — GUI Technology (Tauri 2.0)

## Common Pitfalls

- **Don't bypass capability checks** — always call `require_category()` before SOVD operations
- **Don't use `Debug` for user-facing output** — use `Display` implementations
- **Don't add `node_modules/` or `target/`** — check `.gitignore`
- **Don't hardcode URLs or tokens** — use environment variables or config
- **API path construction** — use `percent-encoding` for component/data IDs to prevent path traversal
- **State machine transitions** — follow the defined phase order: PreCheck → Deployment → Monitoring → Verification → Reporting

## File Naming

- Rust: `snake_case.rs`
- TypeScript/React: `PascalCase.tsx` for components, `camelCase.ts` for utilities/stores
- Docs: `UPPER_CASE.md` for root docs, `lower-case.md` for ADRs

## Eclipse Foundation

This project follows the [eclipse-score/module_template](https://github.com/eclipse-score/module_template) conventions:

- `LICENSE` — Apache-2.0
- `NOTICE` — Third-party attribution
- `CONTRIBUTION.md` — Requires ECA + DCO signing
- `project_config.bzl` — Project metadata (Rust, QM level)
- `.github/CODEOWNERS` — Review ownership
- Issue/PR templates in `.github/`
