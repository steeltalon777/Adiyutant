# MEMORY

Stable project facts. This is not a changelog.

## Project Identity

- Adiyutant is an organizer with an embedded AI agent.
- MVP 0.1 is local-first.
- First implementation target is `AdiyutantCore`.
- Core must work without backend and without external LLM.
- API keys must not be stored in mobile clients.
- Backend is planned later for sync, account management, cloud backup and AI relay.
- OpenClaw may become one possible Agent Gateway, but is not required for MVP 0.1.

## Architecture Decisions

- **Local-first Core first** — domain logic before UI, backend, or sync.
- **Core owns data and rules** — the Core is the single source of truth for domain state.
- **Platform owns UI and OS integration** — shells add platform-specific behavior.
- **Agent Gateway is external** — Core does not call LLM providers.
- **No API keys in mobile clients** — provider keys live on the server or gateway.

## Current Phase

Phase 0 — Repository Foundation (TASK-0000). Bootstrap only.
