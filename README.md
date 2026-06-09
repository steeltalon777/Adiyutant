# Adiyutant

## Project Overview

Adiyutant is a local-first organizer with an embedded AI agent.

Planned features: alarm, timer, daily journal, calendar, habits, reminders, personal context, AI agent chat, tool-based agent operations, future cross-device sync.

## Current State

**Status: MVP 0.1** — local-first Rust Core implemented.

- **3 crates**: `adiyutant_core` (domain), `adiyutant_store` (SQLite), `adiyutant_cli` (CLI)
- **9 CLI commands**: today, log, checkin, habit, timer, reminder, alarm, context, suggest
- **10 SQLite tables**: DailyLog, CheckIn, Habit, HabitEvent, ReminderDefinition, AlarmDefinition, TimerDefinition, ContextDocument, Plan, ActionProposal
- **103 tests**: 66 core + 23 store + 11 CLI unit + 3 CLI integration
- **6 local rules** in `LocalRuleAgentGateway` — no LLM required
- **0 external dependencies**: no server, no sync, no AI keys needed

## Repository Type

Multi-project solution repository:

| Project | Status | Responsibility |
|---------|--------|----------------|
| AdiyutantCore | **Implemented (MVP 0.1)** | Domain logic, local storage, CLI |
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

## Development Progress

1. Phase 0 — Repository Foundation ✅
2. Phase 1 — Rust Core Workspace ✅
3. Phase 2 — Domain Model ✅
4. Phase 3 — Local Rules and Today State ✅
5. Phase 4 — SQLite Storage ✅
6. Phase 5 — CLI MVP ✅
7. Phase 6 — Stabilization ✅

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
| `docs/mvp-acceptance.md` | MVP 0.1 acceptance scenario |
| `pipeline-state.json` | Pipeline phase state |
