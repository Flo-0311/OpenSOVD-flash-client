# ARCHITECTURE.md

**OpenSOVD Flash Client**

---

## Purpose

This document describes the architecture of the **OpenSOVD Flash Client** using the **C4 model**.

The goal is to provide a **clear, stable and open architectural reference** that:

- supports **open-source development** within the Eclipse OpenSOVD ecosystem
- cleanly separates **generic functionality** from **OEM-specific differentiation**
- enables **diagnostics and flashing exclusively via SOVD (ISO 17978)**
- serves as a **functional replacement for legacy monolithic tools** (e.g. DTS Monaco)

---

## Eclipse OpenSOVD Ecosystem

This client is part of the **Eclipse OpenSOVD** project and builds on existing assets:

| Component | Repository | Language | Role |
|---|---|---|---|
| **OpenSOVD (Main)** | [eclipse-opensovd/opensovd](https://github.com/eclipse-opensovd/opensovd) | — | Design, architecture, coordination |
| **Classic Diagnostic Adapter** | [eclipse-opensovd/classic-diagnostic-adapter](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) | Rust | SOVD→UDS/DoIP translation for classic ECUs |
| **ODX Converter** | [eclipse-opensovd/odx-converter](https://github.com/eclipse-opensovd/odx-converter) | Kotlin | ODX→MDD diagnostic description converter |
| **Fault Library** | [eclipse-opensovd/fault-lib](https://github.com/eclipse-opensovd/fault-lib) | Rust | Framework-agnostic fault reporting |
| **UDS2SOVD Proxy** | [eclipse-opensovd/uds2sovd-proxy](https://github.com/eclipse-opensovd/uds2sovd-proxy) | — | UDS→SOVD mapping for legacy testers |
| **Flash Client** | [rettde/OpenSOVD-flash-client](https://github.com/rettde/OpenSOVD-flash-client) | Rust | SOVD-native diagnostic & flash client |

---

## Fundamental Communication Principle

> **The client speaks SOVD — always, exclusively, to every ECU.**
>
> Whether the target is a native SOVD High Performance Computer (HPC)
> or a classic UDS ECU behind a Classic Diagnostic Adapter (CDA) —
> the client uses the **identical SOVD REST API**.
> Protocol translation is **transparent** and handled server-side.

```
                          ┌─────────────────────────────────────────────────────┐
                          │              SOVD Infrastructure                    │
                          │                                                     │
┌──────────────┐   SOVD  │  ┌──────────┐     ┌──────────────┐                  │
│  OpenSOVD    │   REST   │  │  SOVD    │────▶│ SOVD Gateway │                  │
│  Flash       │─────────▶│  │  Server  │     └──────┬───────┘                  │
│  Client      │   HTTP   │  └──────────┘            │                          │
└──────────────┘          │                  ┌───────┴────────┐                 │
                          │                  │                │                 │
                          │           ┌──────▼──────┐  ┌─────▼──────┐          │
                          │           │   Classic    │  │   Native   │          │
                          │           │   Diagnostic │  │   SOVD     │          │
                          │           │   Adapter    │  │   (direct) │          │
                          │           │   (CDA)      │  │            │          │
                          │           └──────┬───────┘  └─────┬──────┘          │
                          │                  │                │                 │
                          └──────────────────┼────────────────┼─────────────────┘
                                             │                │
                                      UDS/DoIP          SOVD native
                                             │                │
                                      ┌──────▼──────┐  ┌─────▼──────┐
                                      │  Classic    │  │   HPC      │
                                      │  UDS ECU    │  │  (AUTOSAR  │
                                      │  (legacy)   │  │   Adaptive │
                                      │             │  │   / S-CORE)│
                                      └─────────────┘  └────────────┘
```

### What this means for the client

1. The client **never** implements UDS, DoIP, CAN, or any lower-level protocol.
2. The client **never** needs to know whether an ECU is classic or native.
3. `ComponentType` (Classic UDS / Native SOVD) is **informational metadata only** —
   it does not change the API calls the client makes.
4. The CDA handles: SOVD→UDS translation, session management, diagnostic descriptions (MDD/ODX).
5. The SOVD Gateway handles: routing, multi-ECU dispatch, service orchestration.

---

## Architectural Principles

1. **SOVD-Only Communication (ISO 17978)**
   The client communicates exclusively via SOVD REST APIs. No UDS, DoIP, CAN,
   or any other protocol. CDA-to-UDS translation is server-side and transparent.

2. **API-First, Tool-Second**
   The client is a consumer of SOVD APIs, not a protocol implementation.

3. **Capability-Driven**
   All features are discovered dynamically via SOVD capabilities.

4. **CDA-Transparent**
   Classic ECUs (UDS) and native HPCs (SOVD) are accessed through the same API.
   The CDA is an infrastructure concern, invisible to the client.

5. **Open Core, Closed Extensions**
   The core is open source; differentiation is implemented via plugins.

6. **No Embedded Security Logic**
   Security is enforced server-side or via plugins.

7. **Automation-Ready**
   The architecture supports interactive and automated use cases equally.

---

## C4 Level 1 — System Context

### Description

The **OpenSOVD Flash Client** is a standalone system used by engineers, automation
systems, and factory environments to perform diagnostics and flashing through an
**SOVD Server / Gateway**.

It does **not** communicate directly with ECUs. All ECU communication — whether
to native SOVD HPCs or classic UDS ECUs via CDA — is mediated by the SOVD
infrastructure.

### Actors

- **Engineer / Developer** — Interactive diagnostics, flash operations
- **Automation System / CI** — Automated flash pipelines, test benches
- **Factory Station** — Production flashing, end-of-line testing
- **Cloud Service** — Fleet diagnostics, remote software deployment

### Context Diagram

```
┌─────────────┐         ┌──────────────────────┐          ┌─────────────────────────┐
│  Engineer /  │────────▶│  OpenSOVD Flash      │──SOVD──▶│ SOVD Server / Gateway   │
│  Automation  │         │  Client              │  REST    │                         │
└─────────────┘         └──────────────────────┘          │  ┌─────┐    ┌────────┐  │
                                                          │  │ CDA │    │ Native │  │
                                                          │  │     │    │ SOVD   │  │
                                                          │  └──┬──┘    └───┬────┘  │
                                                          └─────┼───────────┼───────┘
                                                           UDS/DoIP    SOVD native
                                                                │           │
                                                          ┌─────▼───┐ ┌────▼─────┐
                                                          │Classic  │ │  HPC     │
                                                          │UDS ECU  │ │ (native) │
                                                          └─────────┘ └──────────┘
```

---

## C4 Level 2 — Container Diagram

### Description

The OpenSOVD Flash Client is composed of **logical containers** with clear responsibilities.

It intentionally avoids embedding backend, protocol, or security logic.

### Containers

- **GUI** (`sovd-gui`) — *New*
  Graphical desktop application for interactive diagnostics and flashing.
  Built with **Tauri 2.0** (Rust) + **React / TypeScript** frontend.

- **CLI** (`sovd-cli`)
  Headless command-line interface for automation, CI pipelines, and scripting.

- **Workflow Engine** (`sovd-workflow`)
  Orchestrates diagnostics and flashing jobs.

- **SOVD Client** (`sovd-client`)
  Handles REST communication and capability discovery.

- **Plugin Runtime** (`sovd-plugin`)
  Loads and manages extensions.

- **Observability** (`sovd-observe`)
  Logging, tracing, reporting.

```
┌────────────────────────────────────────────────────────────────┐
│                    OpenSOVD Flash Client                       │
│                                                                │
│  ┌────────────────────┐  ┌───────────┐                        │
│  │   GUI (Tauri 2.0)  │  │    CLI    │                        │
│  │  ┌──────────────┐  │  │ (sovd-cli)│                        │
│  │  │ React / TS   │  │  └─────┬─────┘                        │
│  │  │ Frontend     │  │        │                               │
│  │  └──────┬───────┘  │        │                               │
│  │     IPC │(Tauri)   │        │                               │
│  │  ┌──────┴───────┐  │        │                               │
│  │  │ Rust Backend │  │        │                               │
│  │  └──────┬───────┘  │        │                               │
│  └─────────┼──────────┘        │                               │
│            │                   │                               │
│            └────────┬──────────┘                               │
│                     │ shared Rust crates                       │
│         ┌───────────┴──────────────┐  ┌──────────┐            │
│         │    Workflow Engine       │──│ Observe  │            │
│         └───────────┬──────────────┘  └──────────┘            │
│                     │                                          │
│         ┌───────────┴──────────────┐                          │
│         │                          │                          │
│    ┌────┴─────┐          ┌─────────┴────────┐                 │
│    │  SOVD    │          │  Plugin Runtime  │                 │
│    │  Client  │          │                  │                 │
│    └────┬─────┘          └────────┬─────────┘                 │
│         │                         │                           │
└─────────┼─────────────────────────┼───────────────────────────┘
          │ SOVD REST only          │
          ▼                         ▼
┌─────────────────────┐    ┌─────────────┐
│  SOVD Server /      │    │  Plugins    │
│  Gateway            │    │ (Security,  │
│  ┌─────┐  ┌───────┐│    │  Backend,   │
│  │ CDA │  │ Diag  ││    │  Workflow)  │
│  │     │  │ Fault  ││    └─────────────┘
│  │     │  │ Mgr    ││
│  └─────┘  └───────┘│
│                     │
└─────────────────────┘
```

---

## C4 Level 3 — Component Diagram (Core)

### Description

This view focuses on the **core internals** of the OpenSOVD Flash Client.

Only **generic, reusable components** are included.

### Core Components

- **Job Controller** (`sovd-workflow::controller`)
  Manages job lifecycle across all phases.

- **Capability Resolver** (`sovd-client::discovery`)
  Interprets available SOVD capabilities. Determines what the server supports
  independently of whether ECUs are classic or native.

- **State Machine** (`sovd-workflow::state_machine`)
  Governs valid job state transitions, handles progress and error states.

- **Plugin Interfaces** (`sovd-plugin::spi`)
  Stable extension points for security, backends, workflows, and reporting.

- **Report Generator** (`sovd-observe::report`)
  Produces audit-ready output for completed jobs.

- **Event Recorder** (`sovd-observe::events`)
  Records all workflow events for tracing and observability.

---

## SOVD Communication Model

### How ECUs are reached

The SOVD standard (ISO 17978) defines a service-oriented architecture where
**all diagnostic communication uses the SOVD REST API**. The infrastructure
handles protocol adaptation transparently:

| ECU Type | Server-side Path | Client Impact |
|---|---|---|
| **Native SOVD HPC** (AUTOSAR Adaptive, S-CORE) | SOVD Gateway → direct SOVD | None — identical API |
| **Classic UDS ECU** (legacy) | SOVD Gateway → CDA → UDS/DoIP | None — identical API |

### Classic Diagnostic Adapter (CDA)

The [CDA](https://github.com/eclipse-opensovd/classic-diagnostic-adapter) is a
server-side component (written in Rust) that:

1. Receives SOVD REST calls from the Gateway
2. Translates them to UDS requests (ISO 14229) using diagnostic descriptions (MDD)
3. Sends UDS requests via DoIP (or other transports) to the classic ECU
4. Translates UDS responses back to SOVD REST responses

The MDD files are generated from ODX using the
[ODX Converter](https://github.com/eclipse-opensovd/odx-converter).

**The Flash Client never interacts with the CDA directly.**

### Fault Management

The [Fault Library](https://github.com/eclipse-opensovd/fault-lib) (Rust) provides
framework-agnostic fault reporting on the ECU side. The Diagnostic Fault Manager
aggregates faults and exposes them via the SOVD Server. The Flash Client reads
faults through the same SOVD API as any other data.

---

## GUI Architecture

### Technology Decision: Tauri 2.0 + React / TypeScript

The GUI uses **Tauri 2.0** as the application shell and **React with TypeScript**
as the frontend framework.

#### Why Tauri

| Criterion | Tauri 2.0 | Electron | egui (pure Rust) | Qt (C++) |
|---|---|---|---|---|
| **Rust Backend Integration** | Native — same crates | Node.js bridge needed | Native | FFI layer needed |
| **Binary Size** | ~5–10 MB | ~150 MB | ~5 MB | ~30 MB |
| **Webview** | OS-native (WebView2, WebKit) | Bundled Chromium | No webview | Custom rendering |
| **UI Ecosystem** | React/Vue/Svelte + npm | React/Vue/Svelte + npm | Limited | Custom widget system |
| **Security** | No Node.js runtime, CSP | Node.js + Chromium | Good | Good |
| **License** | MIT | MIT | MIT/Apache | GPL/Commercial |
| **Cross-Platform** | Windows, macOS, Linux | Windows, macOS, Linux | Windows, macOS, Linux | Windows, macOS, Linux |
| **Automotive Suitability** | Lightweight, factory-ready | Too heavy for factory | UX-limited | License issues (GPL) |

**Decision**: Tauri 2.0 offers the best balance of:
- **Seamless Rust integration** — existing crates (`sovd-client`, `sovd-workflow`, etc.) are exposed directly as Tauri commands
- **Lightweight footprint** — critical for factory stations and embedded testers
- **Modern UI stack** — React + TypeScript + TailwindCSS for professional UX
- **Open-source compatibility** — MIT license, fits the Eclipse ecosystem

#### Why React + TypeScript

- **TypeScript** — Static typing, aligned with Rust’s philosophy
- **React** — Component-based, massive ecosystem (shadcn/ui, Recharts, TanStack Table)
- **TailwindCSS** — Utility-first styling, consistent design system
- **Vite** — Fast dev tooling, HMR for productive development

### GUI Stack Overview

```
┌─────────────────────────────────────────────────────────┐
│                   Tauri 2.0 Application                  │
│                                                         │
│  ┌───────────────────────────────────────────────────┐  │
│  │               React / TypeScript Frontend          │  │
│  │                                                   │  │
│  │  ┌───────────┐ ┌──────────┐ ┌──────────────────┐ │  │
│  │  │ Dashboard │ │ Flash    │ │ Component        │ │  │
│  │  │           │ │ Wizard   │ │ Explorer         │ │  │
│  │  └───────────┘ └──────────┘ └──────────────────┘ │  │
│  │  ┌───────────┐ ┌──────────┐ ┌──────────────────┐ │  │
│  │  │ DTC       │ │ Job      │ │ Live             │ │  │
│  │  │ Viewer    │ │ Monitor  │ │ Monitoring       │ │  │
│  │  └───────────┘ └──────────┘ └──────────────────┘ │  │
│  │  ┌───────────┐ ┌──────────┐ ┌──────────────────┐ │  │
│  │  │ Bulk      │ │ Reports  │ │ Diagnostics      │ │  │
│  │  │ Flash     │ │          │ │                  │ │  │
│  │  └───────────┘ └──────────┘ └──────────────────┘ │  │
│  │  ┌───────────┐ ┌──────────┐ ┌──────────────────┐ │  │
│  │  │ Log       │ │ Plugin   │ │ Settings         │ │  │
│  │  │ Viewer    │ │ Manager  │ │                  │ │  │
│  │  └───────────┘ └──────────┘ └──────────────────┘ │  │
│  │                                                   │  │
│  │  UI Libraries: TailwindCSS · Lucide React         │  │
│  │  State: Zustand                                   │  │
│  └──────────────────────┬────────────────────────────┘  │
│                         │ Tauri IPC (invoke / events)    │
│  ┌──────────────────────┴────────────────────────────┐  │
│  │               Rust Backend (Tauri Commands)        │  │
│  │                                                   │  │
│  │  sovd-client  · sovd-workflow · sovd-plugin        │  │
│  │  sovd-core    · sovd-observe                       │  │
│  └───────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

### GUI Requirements

#### Functional Requirements

| ID | Requirement | Priority | Description |
|---|---|---|---|
| **GUI-F01** | Server Connection | High | Connect to SOVD server, health check, capability discovery |
| **GUI-F02** | Component Explorer | High | Tabular view of all ECUs with status, type (Native/Classic), SW version |
| **GUI-F03** | Flash Wizard | High | Guided assistant: select component → configure package → start flash → progress → report |
| **GUI-F04** | Flash Progress | High | Real-time progress display with phase indicator (PreCheck → Deployment → Monitoring → Verification → Reporting) |
| **GUI-F05** | Job Overview | High | List of all running/completed jobs with status, duration, result |
| **GUI-F06** | DTC Viewer | Medium | Read/clear fault memory per component, filter by severity/status |
| **GUI-F07** | Diagnostic Data | Medium | Read/write DID values per component |
| **GUI-F08** | Configuration Editor | Medium | Read/write ECU coding |
| **GUI-F09** | Live Monitoring | Medium | Real-time monitoring parameters with auto-refresh and sparkline charts |
| **GUI-F10** | Log Viewer | Medium | Structured log display with filtering and search |
| **GUI-F11** | Plugin Manager | Low | Display loaded plugins, plugin status, dynamic loading |
| **GUI-F12** | Report Export | Medium | Export flash reports as JSON/HTML/PDF |
| **GUI-F13** | Bulk Flash | Medium | Flash multiple components sequentially with individual progress |
| **GUI-F14** | Dark/Light Theme | Low | Switchable color scheme, respects system preference |

#### Non-Functional Requirements

| ID | Requirement | Description |
|---|---|---|
| **GUI-NF01** | Responsive Layout | Min. 1280×720, optimized for 1920×1080 |
| **GUI-NF02** | Startup Time | < 2s to interactive UI |
| **GUI-NF03** | Memory Usage | < 150 MB RAM in normal operation |
| **GUI-NF04** | Accessibility | WCAG 2.1 AA — keyboard navigation, screen reader support, contrast |
| **GUI-NF05** | Language | English UI (plain strings, no i18n framework) |
| **GUI-NF06** | Offline Capability | UI starts without server connection, shows connection status |
| **GUI-NF07** | Error Tolerance | Network errors displayed in UI, no crashes |
| **GUI-NF08** | Cross-Platform | Windows 10+, macOS 12+, Ubuntu 22.04+ |

### Tauri IPC — Rust ↔ TypeScript Communication

Communication between frontend and backend uses the **Tauri Command system**:

```
TypeScript (Frontend)                Rust (Backend)
──────────────────────               ──────────────────────
invoke('connect',                    #[tauri::command]
  { url, token })    ──IPC──▶        fn connect(url, token)
                                         → sovd_client::connect()

invoke('list_components')            #[tauri::command]
                     ──IPC──▶        fn list_components()
                                         → sovd_client::list_components()

invoke('start_flash',                #[tauri::command]
  { component, pkg })──IPC──▶        fn start_flash(component, pkg)
                                         → sovd_workflow::execute_flash()

listen('flash_progress')             app.emit('flash_progress',
                     ◀──Event──          { phase, percent })
```

**Advantages of this model:**
- Type-safe serialization (serde ↔ TypeScript interfaces)
- No HTTP overhead between frontend and backend
- Backend logic stays in Rust — no duplication
- Events for real-time updates (flash progress, logs)

### GUI Component Structure (React)

```
sovd-gui/
├── src-tauri/              # Tauri Rust backend
│   ├── src/
│   │   ├── main.rs         # Tauri app setup
│   │   ├── commands/       # Tauri Commands (→ sovd-* Crates)
│   │   │   ├── connection.rs
│   │   │   ├── components.rs
│   │   │   ├── flash.rs
│   │   │   ├── diagnostics.rs
│   │   │   └── plugins.rs
│   │   └── state.rs        # Shared application state
│   ├── Cargo.toml          # Depends on: sovd-client, sovd-workflow, etc.
│   └── tauri.conf.json
├── src/                    # React TypeScript Frontend
│   ├── App.tsx
│   ├── components/         # Reusable UI components
│   │   ├── ConnectionBar.tsx
│   │   ├── PhaseIndicator.tsx
│   │   └── Toast.tsx
│   ├── lib/
│   │   └── tauri.ts        # Tauri command wrappers
│   ├── pages/              # Main views
│   │   ├── Dashboard.tsx
│   │   ├── ComponentExplorer.tsx
│   │   ├── FlashWizard.tsx
│   │   ├── BulkFlash.tsx
│   │   ├── JobMonitor.tsx
│   │   ├── DtcViewer.tsx
│   │   ├── Diagnostics.tsx
│   │   ├── LiveMonitoring.tsx
│   │   ├── LogViewer.tsx
│   │   ├── PluginManager.tsx
│   │   ├── Reports.tsx
│   │   └── Settings.tsx
│   ├── stores/             # Zustand State Management
│   │   ├── connectionStore.ts
│   │   ├── jobStore.ts
│   │   └── settingsStore.ts
│   └── types/
│       └── index.ts        # TypeScript interfaces (mirror Rust types)
├── package.json
├── tsconfig.json
├── tailwind.config.ts
└── vite.config.ts
```

### GUI Page Overview

```
┌──────────────────────────────────────────────────────────────┐
│  ┌─ Sidebar ─────┐  ┌─ Main Content ───────────────────────┐│
│  │                │  │                                      ││
│  │  Dashboard  1  │  │  ┌──────────────────────────────┐   ││
│  │  Components 2  │  │  │  Server: connected            │   ││
│  │  Flash      3  │  │  │  Components: 12    Jobs: 3    │   ││
│  │  Jobs       4  │  │  └──────────────────────────────┘   ││
│  │  DTCs       5  │  │                                      ││
│  │  Diagnostics6  │  │  ┌─────────┐ ┌─────────┐ ┌───────┐ ││
│  │  Monitoring 7  │  │  │ Active  │ │ Pending │ │ Done  │ ││
│  │  Bulk Flash 8  │  │  │ Jobs: 1 │ │ Jobs: 2 │ │ J: 15 │ ││
│  │  Reports    9  │  │  └─────────┘ └─────────┘ └───────┘ ││
│  │  Logs       0  │  │                                      ││
│  │  Plugins       │  │  [Recent Flash Jobs]                 ││
│  │ ─────────────  │  │  ECU_01  v2.1.3→v2.2.0  Done  100%  ││
│  │  Settings      │  │  ECU_04  v1.0.0→v1.1.0  Running 67% ││
│  └────────────────┘  │  ECU_07  v3.0.1→v3.1.0  Pending     ││
│                      └──────────────────────────────────────┘│
│  ┌─ Status Bar ──────────────────────────────────────────┐  │
│  │ SOVD v1.0 │ 12 Components │ 42 Capabilities │ Online │  │
│  └───────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────┘
```

---

## Plugin Architecture

### Rationale

Plugins isolate **non-generic functionality** and prevent vendor lock-in.

The core never depends on a specific plugin implementation.

### Plugin Types

#### Security Plugins
- Authentication (token providers, certificate handling)
- Authorization (access control decisions)
- Signature verification

#### Backend Integration Plugins
- OEM flashing backends
- CI / test bench integration
- Legacy systems (e.g. DevFlash)
- Software package resolution

#### UI / Workflow Plugins
- Approval flows
- OEM-specific UX
- Compliance checks

#### Reporting Plugins
- Custom report formats (JSON, HTML, PDF, XML)
- Audit trail generation

### Plugin SPI

Plugins implement Rust traits defined in `sovd-plugin::spi`:

- `Plugin` — Base trait (manifest, lifecycle)
- `SecurityPlugin` — Auth & crypto
- `BackendPlugin` — OEM backend integration
- `WorkflowPlugin` — Phase gates & approval
- `ReportingPlugin` — Custom report generation

Plugins can be:
- **Built-in** (statically linked)
- **Dynamic** (loaded at runtime from `.so`/`.dylib`/`.dll`)

---

## Flashing as a Job

Flashing is modeled as a **job**, not a command.

The same job model applies regardless of whether the target ECU is a native
SOVD HPC or a classic UDS ECU behind a CDA — the SOVD API is identical.

### Generic Flash Job Phases

1. **Pre-Check** — Verify component availability and preconditions via SOVD
2. **Deployment** — Transfer software package to ECU via SOVD flash endpoint
3. **Monitoring** — Poll flash progress via SOVD job status endpoint
4. **Verification** — Confirm software version post-flash via SOVD data read
5. **Reporting** — Generate audit-ready output

All enforcement and decision logic resides **outside the core**.

### Sequence: Flash Job (simplified)

```
Client              SOVD Server          Gateway              CDA / HPC
  │                      │                  │                      │
  │──GET /components────▶│                  │                      │
  │◀─────component list──│                  │                      │
  │                      │                  │                      │
  │──POST /flash────────▶│──route──────────▶│                      │
  │                      │                  │──SOVD or UDS/DoIP──▶│
  │                      │                  │◀─────────────────────│
  │◀─────job_id──────────│◀─────────────────│                      │
  │                      │                  │                      │
  │──GET /flash/{id}────▶│──route──────────▶│                      │
  │                      │                  │──status query───────▶│
  │◀─────progress────────│◀─────────────────│◀─────────────────────│
  │                      │                  │                      │
  │──GET /data/version──▶│──route──────────▶│                      │
  │◀─────sw_version──────│◀─────────────────│◀─────────────────────│
```

The client sees **only** the left column. Everything to the right is transparent.

---

## Security Model

- The core client contains **no secrets**
- No cryptographic material is stored in the client
- Authentication uses HTTPS + token-based auth (as per SOVD/ISO 17978)
- Authorization decisions are external (SOVD Server / Authentication Manager)
- Plugins may enforce OEM-specific policies
- The CDA does not add additional auth — it inherits the SOVD session

This ensures the client remains **open-source safe**.

---

## Deployment View

The OpenSOVD Flash Client supports multiple deployment modes:

| Deployment | Interface | Use Case |
|---|---|---|
| **Desktop (GUI)** | Tauri App (`.msi` / `.dmg` / `.AppImage`) | Developer workstation, interactive diagnostics & flash |
| **CLI** | Single binary (`sovd-flash`) | CI/CD pipeline, scripting, automation |
| **Factory Station** | GUI or CLI | End-of-line programming, EOL testing |
| **Docker / Cloud** | CLI in container | Fleet tooling, remote deployment |

### GUI Distribution

Tauri generates native installers per platform:

| Platform | Installer | Webview |
|---|---|---|
| Windows 10+ | `.msi` / `.exe` (NSIS) | WebView2 (Edge-based, pre-installed on Win 11) |
| macOS 12+ | `.dmg` / `.app` | WebKit (system-native) |
| Linux (Ubuntu 22.04+) | `.deb` / `.AppImage` | WebKitGTK |

The GUI binary is ~10 MB (vs. ~150 MB with Electron).

The SOVD infrastructure (Server, Gateway, CDA) is always remote — the client
connects over HTTP/HTTPS.

---

## Crate Structure

```
OpenSOVD-flash-client/
├── crates/
│   ├── sovd-core/       # Types, models (Component, Capability, Job, DTC, etc.)
│   ├── sovd-client/     # SOVD REST client & capability discovery
│   ├── sovd-plugin/     # Plugin SPI, registry, dynamic loading
│   ├── sovd-workflow/   # Workflow engine, job controller, state machine
│   ├── sovd-observe/    # Logging, tracing, event recording, reports
│   └── sovd-cli/        # CLI binary (sovd-flash)
├── sovd-gui/            # GUI application (Tauri 2.0 + React/TypeScript)
│   ├── src-tauri/       #   Rust backend (Tauri commands, app state)
│   ├── src/             #   React frontend (pages, components, hooks)
│   ├── package.json     #   Node.js dependencies
│   └── vite.config.ts   #   Build configuration
├── plugins/
│   └── example-plugin/  # Example dynamic plugin
├── docs/
│   └── adr/             # Architecture Decision Records
└── ARCHITECTURE.md
```

### GUI Technology Stack

| Layer | Technology | Version | Purpose |
|---|---|---|---|
| **App Shell** | Tauri | 2.x | Native desktop app, Rust backend, IPC |
| **Frontend Framework** | React | 18.x | Component-based UI |
| **Language (Frontend)** | TypeScript | 5.x | Type safety, IDE support |
| **Styling** | TailwindCSS | 3.x | Utility-first CSS, dark/light mode |
| **UI Components** | shadcn/ui | latest | Accessible, customizable primitives |
| **State Management** | Zustand | 5.x | Lightweight global state |
| **Icons** | Lucide React | latest | Consistent icon set |
| **Build** | Vite | 6.x | Fast dev server, HMR |
| **Backend (Rust)** | sovd-* crates | workspace | Shared with CLI |

### Relationship to Eclipse OpenSOVD Crates

| This client crate | Interacts with OpenSOVD component |
|---|---|
| `sovd-client` | SOVD Server (REST API consumer) |
| `sovd-core::Component` | Components exposed by Gateway (CDA + native) |
| `sovd-core::DiagnosticTroubleCode` | Fault Manager / Fault Library data |
| `sovd-workflow` | Flash Service App (via SOVD API) |

---

## Architectural Constraints

- **SOVD is the only supported external API** — no UDS, DoIP, CAN, or other protocols
- **No direct ECU communication** — all routes through SOVD Server/Gateway
- **CDA is transparent** — the client does not know or care about CDA internals
- **No OEM IP in the core** — differentiation only via plugins
- **Backward compatibility via plugins only**

---

## Architecture Decision Records (ADR)

Key decisions are tracked in `/docs/adr`.

- [ADR-0001: Open Core](docs/adr/0001-open-core.md)
- [ADR-0002: Capability-Driven Workflows](docs/adr/0002-capability-driven-workflows.md)
- [ADR-0003: Plugin Isolation](docs/adr/0003-plugin-isolation.md)
- [ADR-0004: SOVD-Only Communication](docs/adr/0004-sovd-only-communication.md)
- [ADR-0005: Mandatory Test Coverage](docs/adr/0005-mandatory-test-coverage.md)
- [ADR-0006: GUI Technology — Tauri + React/TypeScript](docs/adr/0006-gui-technology.md)

---

## Summary

The OpenSOVD Flash Client architecture:

- speaks **100% SOVD** — to every ECU, whether classic UDS (via CDA) or native HPC
- cleanly separates **standard vs. differentiation**
- provides a **modern GUI** (Tauri 2.0 + React/TypeScript) **and** a headless **CLI**
- shares the **same Rust crates** between GUI and CLI — zero code duplication
- integrates with the **Eclipse OpenSOVD ecosystem** (CDA, Fault Library, ODX Converter)
- enables **open collaboration**
- replaces legacy tooling **without reproducing its flaws**
- scales from **interactive desktop** to **fully automated CI pipelines**

It is intentionally **simple, explicit and extensible**.
