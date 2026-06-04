# AI_CONTEXT

## Repository Status

This repository is in **bootstrap state**. No application code has been implemented.

## Core Rules for AI Agents

- This repository is in bootstrap state.
- Do not assume implementation exists.
- Core is the planned source of domain logic.
- UI clients are planned shells.
- Backend is not present yet.
- Agent Gateway is external to Core.
- Do not store LLM API keys in mobile clients.
- Do not add backend, sync, LLM, MCP, OpenClaw or UI code unless a specific task asks for it.

## Architecture Principles

- **Local-first Core first** — Core must work without backend and without external LLM.
- **Server later** — Backend/sync is added after Core MVP.
- **AI Gateway outside the Core** — Core must not call LLM providers directly.
- **UI clients are shells** — Platform clients consume Core, not duplicate it.

## Document Status Conventions

- `Current` — reflects implemented state
- `Planned` — intended future state, not yet implemented
- `Unknown` — decision not yet made
