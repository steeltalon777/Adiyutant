# AI_CONTEXT

## Repository Status

This repository is in **MVP 0.1 state**. Core implemented, 103 tests, 9 CLI commands, SQLite storage.

## Implemented

- `AdiyutantCore/adiyutant_core` — 10 domain entities, 6 local rules, `TodayState` aggregator
- `AdiyutantCore/adiyutant_store` — `SqliteStore` with 10 tables
- `AdiyutantCore/adiyutant_cli` — 9 commands (`today`, `log`, `checkin`, `habit`, `timer`, `reminder`, `alarm`, `context`, `suggest`)

## Not Yet Implemented

- UI clients (Android, Web, Desktop)
- Agent Gateway (except `LocalRuleAgentGateway`)
- External LLM integration
- Backend / sync server
- MCP

## Core Rules for AI Agents

- Core is the source of domain logic.
- UI clients are shells around Core.
- Backend is not present yet.
- Agent Gateway is external to Core.
- Do not store LLM API keys in mobile clients.
- Do not add backend, sync, LLM or UI code unless a specific task asks for it.

## Architecture Principles

- **Local-first Core first** — Core works without backend and without external LLM.
- **Server later** — Backend/sync is added after Core MVP.
- **AI Gateway outside the Core** — Core must not call LLM providers directly.
- **UI clients are shells** — Platform clients consume Core, not duplicate it.

## Document Status Conventions

- `Current` — reflects implemented state
- `Planned` — intended future state, not yet implemented
- `Unknown` — decision not yet made
