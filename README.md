## Quck start

sudo docker build -f Dockerfile.mock -t sovd-mock:latest

sudo docker run -p 8080:8080 sovd-mock:latest
npm run tauri dev





# OpenSOVD Flash Client

[![License](https://img.shields.io/badge/License-Apache_2.0-blue.svg)](LICENSE)
[![CI](https://github.com/rettde/OpenSOVD-flash-client/actions/workflows/ci.yml/badge.svg)](https://github.com/rettde/OpenSOVD-flash-client/actions/workflows/ci.yml)
[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org)

A modern, open-source diagnostic and flashing client that communicates **exclusively via SOVD (Service-Oriented Vehicle Diagnostics, ISO 17978)** APIs.

Part of the [Eclipse OpenSOVD](https://github.com/eclipse-opensovd) ecosystem. **Replaces legacy monolithic tools like DTS Monaco** with a clean, extensible, automation-ready architecture.

## Key Principle

> **100% SOVD — to every ECU.**
>
> Whether the target is a native SOVD HPC or a classic UDS ECU behind a
> Classic Diagnostic Adapter (CDA) — the client uses the identical SOVD REST API.
> Protocol translation (SOVD↔UDS/DoIP) is handled server-side and fully transparent.

## Features

- **100% SOVD**: Communicates exclusively via SOVD REST APIs — no UDS, DoIP, or CAN
- **CDA-transparent**: Classic UDS ECUs and native SOVD HPCs accessed through the same API
- **Capability-driven**: Discovers and adapts to server capabilities dynamically
- **Plugin architecture**: Open core with extensible plugin system for OEM differentiation
- **Flashing as a Job**: Full lifecycle management with pre-check, deployment, monitoring, verification, and reporting
- **Automation-ready**: CLI with JSON output, environment variable configuration, exit codes
- **Observable**: Structured logging, event recording, audit-ready reports
- **No embedded secrets**: Security handled by plugins or external token providers
- **Eclipse OpenSOVD integration**: Builds on CDA, Fault Library, ODX Converter

## Eclipse OpenSOVD Ecosystem

| Component | Repository | Role |
|---|---|---|
| **This Client** | [OpenSOVD-flash-client](https://github.com/rettde/OpenSOVD-flash-client) | SOVD-native diagnostic & flash client (CLI + GUI) |
| **CDA** | [classic-diagnostic-adapter](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) | SOVD→UDS/DoIP for classic ECUs (Rust) |
| **Fault Library** | [fault-lib](https://github.com/eclipse-opensovd/fault-lib) | Framework-agnostic fault reporting (Rust) |
| **ODX Converter** | [odx-converter](https://github.com/eclipse-opensovd/odx-converter) | ODX→MDD converter (Kotlin) |
| **UDS2SOVD Proxy** | [uds2sovd-proxy](https://github.com/eclipse-opensovd/uds2sovd-proxy) | UDS→SOVD for legacy testers |

## Quick Start

### Prerequisites

- Rust 1.75+ (install via [rustup](https://rustup.rs))
- A running SOVD server (with CDA for classic ECU access)

### Build

```bash
cargo build --release
```

The binary is at `target/release/sovd-flash`.

### Usage

```bash
# Check server health
sovd-flash --server http://localhost:8080 health

# Discover capabilities
sovd-flash --server http://localhost:8080 capabilities

# List ECU components
sovd-flash --server http://localhost:8080 components

# Read diagnostic data (DID)
sovd-flash diag read --component ECU_01 --data-id voltage

# Read DTCs
sovd-flash diag dtc --component ECU_01

# Clear DTCs
sovd-flash diag clear-dtc --component ECU_01

# Start a flash job
sovd-flash flash start --component ECU_01 --package MyUpdate --version 2.0.0

# Get flash status
sovd-flash flash status --job-id <UUID>

# JSON output (for automation)
sovd-flash --format json components
```

### Environment Variables

| Variable | Description | Default |
|---|---|---|
| `SOVD_SERVER_URL` | SOVD server base URL | `http://localhost:8080` |
| `SOVD_AUTH_TOKEN` | Bearer authentication token | — |
| `RUST_LOG` | Log level filter | `info` |

### Configuration File

Optional: `~/.config/sovd-flash/config.toml`

```toml
server_url = "http://sovd-server:8080"
plugin_dirs = ["/opt/sovd-plugins"]
output_format = "text"
timeout_seconds = 30
```

## Architecture

See [ARCHITECTURE.md](ARCHITECTURE.md) for the full C4-model architecture documentation.

### Crate Structure

| Crate | Purpose |
|---|---|
| `sovd-core` | Types, models, error handling |
| `sovd-client` | SOVD REST client & capability discovery |
| `sovd-plugin` | Plugin SPI, registry, dynamic loading |
| `sovd-workflow` | Workflow engine, job controller, state machine |
| `sovd-observe` | Logging, tracing, event recording, reports |
| `sovd-cli` | CLI binary (`sovd-flash`) |

### Plugin Development

Plugins implement traits from `sovd-plugin::spi`:

```rust
use sovd_plugin::spi::{Plugin, BackendPlugin, PluginManifest};

pub struct MyPlugin { /* ... */ }

#[async_trait]
impl Plugin for MyPlugin {
    fn manifest(&self) -> &PluginManifest { /* ... */ }
}

#[async_trait]
impl BackendPlugin for MyPlugin {
    async fn pre_flash(&self, job: &Job) -> SovdResult<FlashDecision> {
        // OEM-specific pre-flash logic
        Ok(FlashDecision::Proceed)
    }
    // ...
}
```

See `plugins/example-plugin/` for a complete example.

## Design Decisions

Architecture Decision Records are in [`docs/adr/`](docs/adr/):

- [ADR-0001: Open Core](docs/adr/0001-open-core.md)
- [ADR-0002: Capability-Driven Workflows](docs/adr/0002-capability-driven-workflows.md)
- [ADR-0003: Plugin Isolation](docs/adr/0003-plugin-isolation.md)
- [ADR-0004: SOVD-Only Communication](docs/adr/0004-sovd-only-communication.md)
- [ADR-0005: Mandatory Test Coverage](docs/adr/0005-mandatory-test-coverage.md)

## Contributing

Contributions are welcome! See [CONTRIBUTION.md](CONTRIBUTION.md) for guidelines.

## License

Licensed under the [Apache License, Version 2.0](LICENSE).

See [NOTICE](NOTICE) for third-party attribution.
