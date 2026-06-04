# Core Boundary

## Purpose

Define what AdiyutantCore owns and what it does not own. This boundary must not be violated.

## Core Owns

- Domain models: DailyLog, CheckIn, Habit, HabitEvent, ReminderDefinition, AlarmDefinition, TimerDefinition, CalendarEvent, ContextDocument, Plan, ActionProposal
- Local SQLite storage
- Business logic and validation
- TodayState aggregation
- Local rules (no LLM)
- Sync state preparation (data change tracking)
- CLI for local interaction

## Core Does Not Own

- Platform alarms and notifications
- UI rendering
- LLM provider calls
- API key storage
- Account management
- Sync transport
- Push notifications
- OAuth / authentication / billing

## Platform Owns

- Android AlarmManager
- Desktop tray / notifications
- Browser notifications
- Platform permissions
- UI rendering

## Backend Owns Later

- Account management
- Device management
- Sync transport
- Cloud backup
- AI relay

## Agent Gateway Owns Later

- LLM provider routing
- API key management
- Tool execution
- Agent response generation

## Key Formula

```
Core stores meaning.
Platforms execute OS-specific behavior.
Agent Gateway handles AI/provider/tool execution.
Backend handles sync/account/cloud concerns later.
```

## Open Questions

- How will Android call Rust Core? (UniFFI, C ABI, or other)
- What mechanism for Desktop integration? (Tauri, direct FFI, or other)
- How will Web integrate with Core? (backend API, WASM, or sync-based)
