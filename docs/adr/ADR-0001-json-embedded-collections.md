# ADR-0001: JSON storage for embedded child collections

**Status:** Accepted  
**Date:** 2026-06-09  
**Driver:** Phase 7 — Core Service Facade & Android Readiness

## Context

Checklist `Template → Item` and `Run → Answer` are one-to-many parent-child relationships. The child entities (`ChecklistItem`, `ChecklistAnswer`) have no independent lifecycle — they are always loaded, created, and saved as part of their parent.

The TZ originally specified separate SQLite tables for `checklist_items` and `checklist_answers`, but also modelled them as `Vec<ChecklistItem>` / `Vec<ChecklistAnswer>` embedded in their parent structs.

## Decision

Store `ChecklistItem`s and `ChecklistAnswer`s as JSON arrays within their parent table columns:

- `checklist_templates.items_json` → serialised `Vec<ChecklistItem>`
- `checklist_runs.answers_json` → serialised `Vec<ChecklistAnswer>`

## Rationale

1. **No independent queries.** Items and answers are never queried outside their parent context. Separate tables would add CRUD bloat without any query benefit.
2. **Transactional atomicity.** A template and its items are inserted/updated in a single row. No multi-table transactions needed.
3. **Pattern consistency.** The MVP already stores `plans.items_json` the same way.
4. **Simpler code.** No separate migration, no separate CRUD, no separate row-mapping.

## Consequences

- Cannot query "all answers across all runs" with a single SQL query (not needed in MVP).
- Schema change in `items_json` format requires a versioned migration (acceptable for MVP).
- If a future query pattern requires separate tables, we can migrate with a one-time script.

## Compliance

The ADR titled "TZ deviation note" is referenced from the TZ at line C6.
