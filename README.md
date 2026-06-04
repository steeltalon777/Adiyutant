# Adiyutant

## Project Overview

Adiyutant is a local-first organizer with an embedded AI agent.

Planned features: alarm, timer, daily journal, calendar, habits, reminders, personal context, AI agent chat, tool-based agent operations, future cross-device sync.

## Current State

**Status: bootstrap**

This repository contains documentation and directory structure only. No application code has been implemented yet.

## Repository Type

Multi-project solution repository. Planned projects:

- `AdiyutantCore` — Rust local-first core
- `AdiyutantAndroid` — Android UI shell
- `AdiyutantWeb` — Web/PWA client
- `AdiyutantDesktop` — Desktop UI shell

## Planned Solution Projects

| Project | Status | Responsibility |
|---------|--------|----------------|
| AdiyutantCore | Planned | Domain logic, local storage, CLI |
| AdiyutantAndroid | Planned | Android mobile UI |
| AdiyutantWeb | Planned | Web/PWA dashboard |
| AdiyutantDesktop | Planned | Desktop UI shell |
| Backend / Sync Server | Not present | Future sync, accounts, AI relay |
| Agent Gateway | Not present | Future AI/LLM integration |

## Architecture Direction

```
Local-first Core first.
Server later.
AI Gateway outside the Core.
```

Key principles:

- Core works without a server.
- Core works without an external LLM.
- UI clients are shells around the Core.
- Backend/server is added later for sync, accounts, cloud backup, and AI relay.
- LLM API keys must not be stored in mobile clients.

## MVP Focus

The first implementation target is **AdiyutantCore** — a portable Rust domain core with SQLite storage and CLI interface.

MVP 0.1 scope: local-only organizer core. No server, no sync, no external AI.

## What Is Not Implemented Yet

- Rust workspace / Cargo.toml
- Domain models
- SQLite storage
- CLI
- Android / Web / Desktop UI
- Backend / sync server
- LLM / AI integration
- Agent Gateway (OpenClaw or custom)

## Documentation Map

| Document | Purpose |
|----------|---------|
| `SPECIFICATION.md` | Product specification |
| `ROADMAP.md` | Global development roadmap |
| `TASKS.md` | Current and planned tasks |
| `SOLUTION_MAP.md` | Solution project map |
| `ARCHITECTURE.md` | Architecture overview |
| `INDEX.md` | Repository navigation |
| `AI_CONTEXT.md` | AI agent context rules |
| `AI_ENTRY_POINTS.md` | Agent reading guide |
| `MEMORY.md` | Stable project facts |
| `AGENTS.md` | Agent working rules |
| `SECURITY_NOTES.md` | Security and privacy notes |
| `GLOSSARY.md` | Domain terminology |
| `docs/core-boundary.md` | Core boundary definition |
| `docs/agent-gateway.md` | Agent Gateway definition |
| `docs/sync-protocol.md` | Future sync protocol notes |

## Development Priority

1. Phase 0 — Repository Foundation (current)
2. Phase 1 — Rust Core Workspace
3. Phase 2 — Domain Model
4. Phase 3 — Local Rules and Today State
5. Phase 4 — SQLite Storage
6. Phase 5 — CLI MVP
7. Phase 6 — Stabilization
