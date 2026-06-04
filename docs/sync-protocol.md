# Sync Protocol

## Purpose

Early notes on the future sync mechanism. No detailed protocol design at this stage.

## Current State

Sync is **planned later**. MVP 0.1 works entirely without sync.

## Key Principles

- MVP 0.1 works without sync.
- Local client is the primary source of truth for MVP.
- Future sync is optional, not required for basic operation.
- The local database is always the primary working state of the client.

## Future Scope

Future backend may manage:
- Account registration and authentication
- Device registration and management
- Change journal and sync cursor
- Cloud backup and restore
- Conflict detection and resolution
- AI relay

## Open Questions

- Conflict resolution strategy is unknown.
- Sync protocol design is not started.
- Data model for sync state is not defined.
- Encryption requirements are not specified.
