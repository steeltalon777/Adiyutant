# AI_CONTEXT

## Repository Status

v0.4.0-ready. Core hardened: service facade split into 10 modules, DTO contract complete (20+ DTOs), 203 tests, golden JSON contract examples.

## Implemented

- `AdiyutantCore/adiyutant_core` — 16 domain entities, 6 local rules, `TodayState` aggregator, `AdiyutantCoreService` facade split into 10 modules (`today.rs`, `planning.rs`, `checkpoints.rs`, `checklists.rs`, `journal.rs`, `checkin.rs`, `basic_entities.rs`, `suggestions.rs`, `helpers.rs` + `mod.rs`)
- `AdiyutantCore/adiyutant_store` — `SqliteStore` with 15 tables, versioned migrations, 7 composite transactions
- `AdiyutantCore/adiyutant_cli` — 15 commands (`today`, `log`, `checkin`, `startup`, `current`, `checklist`, `plan`, `habit`, `timer`, `reminder`, `alarm`, `context`, `suggest`, `journal`, `waiting`)
- `docs/contracts/examples/` — 7 golden JSON contract examples

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
