# OpenSOVD Flash Client

The [OpenSOVD Flash Client](https://github.com/rettde/OpenSOVD-flash-client) is an open-source
SOVD-native diagnostic and flashing client, part of the
[Eclipse OpenSOVD](https://github.com/eclipse-opensovd) ecosystem.

The source code is hosted at [GitHub](https://github.com/rettde/OpenSOVD-flash-client).

Please note that the [Eclipse Foundation's Terms of Use](https://www.eclipse.org/legal/terms-of-use/) apply.
In addition, you need to sign the [ECA](https://www.eclipse.org/legal/ECA.php) and the
[DCO](https://www.eclipse.org/legal/dco/) to contribute to the project.

## Contributing

### Getting the Source Code & Building the Project

Please refer to the [README.md](README.md) for build instructions.

#### Prerequisites

- **Rust 1.75+** — install via [rustup](https://rustup.rs)
- **Node.js 18+** — for GUI development (`sovd-gui/`)

#### Build & Test

```bash
# Build all crates
cargo build --workspace

# Run all tests
cargo test --workspace

# Check formatting
cargo fmt --all -- --check

# Run clippy
cargo clippy --workspace -- -D warnings
```

#### GUI Development

```bash
cd sovd-gui
npm install
npm run tauri dev
```

### Getting Involved

#### Bug Fixes and Improvements

In case you want to fix a bug or contribute an improvement, please perform the following steps:

1. Create a PR by using the corresponding template
   ([Bugfix PR template](.github/PULL_REQUEST_TEMPLATE/bug_fix.md) or
   [Improvement PR template](.github/PULL_REQUEST_TEMPLATE/improvement.md)).
   Please mark your PR as **Draft** until it is ready for review by the Committers
   (see the [Eclipse Foundation Project Handbook](https://www.eclipse.org/projects/handbook/#contributing-committers)
   for more information on role definitions).
2. Initiate content review by opening a corresponding issue for the PR when it is ready
   for review. Use the [Bugfix Issue template](.github/ISSUE_TEMPLATE/bug_fix.md) or
   [Improvement Issue template](.github/ISSUE_TEMPLATE/improvement.md).

### Code Style

- Follow standard Rust idioms and `rustfmt` defaults
- Add `#[cfg(test)]` modules to every source file with public items (ADR-0005)
- Use `tracing` for logging (not `println!` or `eprintln!`)
- Error types use `thiserror`; application errors use `anyhow`
- Keep dependencies minimal — justify new additions

### Commit Messages

All Git commit messages must adhere to the rules described in the
[Eclipse Foundation Project Handbook](https://www.eclipse.org/projects/handbook/#resources-commit).

Use conventional commit style:

```
feat(sovd-client): add retry backoff configuration
fix(sovd-core): correct DTC severity Display impl
docs: update ARCHITECTURE.md with plugin SPI details
test(sovd-workflow): add state machine edge case tests
```

### Key Architectural Principles

1. **SOVD-Only** — The client speaks SOVD REST exclusively (ADR-0004)
2. **Capability-Driven** — Features adapt to server capabilities (ADR-0002)
3. **Open Core** — Differentiation via plugins only (ADR-0001)
4. **Mandatory Tests** — Every source file with public items must have tests (ADR-0005)

### Project Structure

| Directory | Purpose |
|---|---|
| `crates/sovd-core/` | Core types, models, error handling |
| `crates/sovd-client/` | SOVD REST client & capability discovery |
| `crates/sovd-plugin/` | Plugin SPI, registry, dynamic loading |
| `crates/sovd-workflow/` | Workflow engine, job controller, state machine |
| `crates/sovd-observe/` | Logging, tracing, event recording, reports |
| `crates/sovd-cli/` | CLI binary (`sovd-flash`) |
| `sovd-gui/` | GUI application (Tauri 2.0 + React/TypeScript) |
| `plugins/example-plugin/` | Example dynamic plugin |
| `docs/adr/` | Architecture Decision Records |

## License

By contributing, you agree that your contributions will be licensed under the
[Apache License 2.0](LICENSE).
