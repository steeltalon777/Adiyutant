# SOLUTION_MAP

## Solution Overview

Adiyutant is structured as a multi-project solution with a local-first Rust core and multiple platform UI shells.

## Projects

| Project | Status | Responsibility |
|---------|--------|----------------|
| AdiyutantCore | **Implemented (MVP 0.1)** | Rust local-first domain core |

Core workspace: 3 crates, SQLite, CLI, 103 tests.

| Crate | Tests | Role |
|-------|-------|------|
| `adiyutant_core` | 66 | Domain models, `TodayState`, `LocalRuleAgentGateway` |
| `adiyutant_store` | 23 | `SqliteStore` with 10 tables |
| `adiyutant_cli` | 14 (11 unit + 3 integration) | 9 CLI commands via clap |

## Planned Projects

| Project | Status | Responsibility |
|---------|--------|----------------|
| AdiyutantAndroid | Planned | Android UI shell |
| AdiyutantWeb | Planned | Web/PWA client |
| AdiyutantDesktop | Planned | Desktop shell |
| Backend / Sync Server | Not present | accounts, devices, sync, cloud backup, AI relay |
| Agent Gateway | Not present (LocalRule only) | LLM routing, tool execution, API key management |

## Unknown / Not Decided

- Core integration method for Android (UniFFI, C ABI, or other)
- Desktop technology choice (Tauri, WPF, or other)
- Web integration method (backend API, WASM, or sync-based)
- First Agent Gateway implementation (OpenClaw, custom, or local relay)
- Sync protocol and conflict resolution
