# SOLUTION_MAP

## Solution Overview

Adiyutant is structured as a multi-project solution with a local-first Rust core and multiple platform UI shells.

## Projects

| Project | Status | Responsibility |
|---------|--------|----------------|
| AdiyutantCore | Planned | Rust local-first domain core |
| AdiyutantAndroid | Planned | Android UI shell |
| AdiyutantWeb | Planned | Web/PWA client |
| AdiyutantDesktop | Planned | Desktop shell |

## Current Responsibility Map

No project has implemented code yet. All projects are in planned/bootstrap state.

## Planned Responsibility Map

- **AdiyutantCore** — domain models, local storage, local rules, CLI, sync state preparation
- **AdiyutantAndroid** — Android UI, platform alarms/notifications, permissions
- **AdiyutantWeb** — Web/PWA dashboard, future cloud interface
- **AdiyutantDesktop** — Desktop UI, system tray, local notifications
- **Backend / Sync Server** — accounts, devices, sync, cloud backup, AI relay (not present yet)
- **Agent Gateway** — LLM routing, tool execution, API key management (not present yet)

## Unknown / Not Decided

- Core integration method for Android (UniFFI, C ABI, or other)
- Desktop technology choice (Tauri, WPF, or other)
- Web integration method (backend API, WASM, or sync-based)
- First Agent Gateway implementation (OpenClaw, custom, or local relay)
- Sync protocol and conflict resolution
