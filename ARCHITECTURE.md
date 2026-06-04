# ARCHITECTURE

## System Overview

Adiyutant is a local-first organizer. The architecture follows a core-shell pattern with a planned external AI gateway.

```
AdiyutantCore (Rust, local-first)
  |
  +-- AdiyutantAndroid (planned)
  +-- AdiyutantWeb (planned)
  +-- AdiyutantDesktop (planned)
  |
  +-- Agent Gateway (planned, external to Core)
  +-- Backend / Sync Server (planned later)
```

## Current State

**Current state: bootstrap repository only. No application code is implemented yet.**

## Planned High-Level Architecture

1. **AdiyutantCore** — portable Rust library containing domain models, local storage, business logic, and CLI
2. **Platform Shells** — UI clients that consume Core and add platform-specific behavior (alarms, notifications, permissions)
3. **Agent Gateway** — external layer that handles LLM routing, tool execution, and API key management
4. **Backend** — optional server for sync, accounts, cloud backup, and AI relay (added later)

## Core Boundary

Core owns:
- Domain models (DailyLog, Habit, Reminder, etc.)
- Local SQLite storage
- Business logic and validation
- TodayState aggregation
- Local rules (no LLM)
- Sync state preparation

Core does not own:
- Platform alarms and notifications
- UI rendering
- LLM provider calls
- API key storage
- Account management
- Sync transport

## Platform Boundary

Platform shells own:
- UI rendering
- OS-specific alarms, notifications, permissions
- Platform-specific storage for settings

## Agent Gateway Boundary

Agent Gateway owns:
- LLM provider routing
- API key management
- Tool execution
- Agent response generation

Agent Gateway is external to Core. Core must not call LLM providers directly.

## Backend Boundary

Backend owns (planned, not implemented):
- Account management
- Device management
- Sync protocol
- Cloud backup
- AI relay

## Known Unknowns

- Android-Core integration mechanism
- Desktop technology choice
- Web integration method
- Agent Gateway implementation choice
- Sync protocol design
- Conflict resolution strategy
