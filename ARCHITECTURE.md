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

**MVP 0.1 completed, v0.2.0 — Core Service Facade & Android Readiness.** 148 tests passing. Core implemented with 3 Rust crates, SQLite persistence, CLI with 15 commands, service facade layer for Android consumption.

### Implemented

```
AdiyutantCore/                   ← Cargo workspace
├── adiyutant_core/               ← 92 tests
│   ├── model/                    ← 16 domain entities (10 legacy + DayPlan,
│   │                               PlanItem, TaskCheckpoint, NudgeSource,
│   │                               ChecklistTemplate, ChecklistRun, JournalEntry)
│   ├── store.rs                  ← Store trait (definition moved from store crate)
│   ├── service.rs                ← AdiyutantCoreService facade (use-case layer)
│   ├── startup.rs                ← StartupState, StartupIntent
│   ├── current_activity.rs       ← CurrentActivity, ExpectedActivityKind
│   ├── dto.rs                    ← Flat DTO layer (no domain types in public API)
│   ├── today_state.rs            ← Day state aggregator + plan_items
│   └── local_rule_gateway.rs     ← 6 local rules (no LLM)
├── adiyutant_store/              ← 46 tests
│       └── Store impl + SqliteStore  ← 15 tables (legacy + plan_items + task_checkpoints
│                                    + checklist_templates + checklist_runs + journal_entries)
└── adiyutant_cli/                ← 10 tests (7 unit + 3 integration)
    └── 15 commands via clap       ← today, log, checkin, startup, current,
                                     checklist, plan, habit, timer, reminder,
                                     alarm, context, suggest, journal, waiting
```

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
