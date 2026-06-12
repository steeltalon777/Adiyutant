# Core Service API — Public Contract

> Public API surface of `AdiyutantCoreService`. This is the sole boundary between
> domain logic and consumers (CLI, Android). All methods return stable DTOs.
> Consumers must never access `Store` or domain internals directly.

---

## Overview

`AdiyutantCoreService` is the application-layer facade. Every `pub fn` below is
a use-case method that:

- Accepts simple types (`&str`, `Option<&str>`, `Option<u8>`, `Option<u64>`)
- Returns a DTO (defined in `adiyutant_core::dto`)
- Returns `CoreError` on failure (mapped to `CoreErrorDto` for consumers)
- Is **stable** unless marked otherwise

---

## Constructor

### `new(store)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `store` | `Box<dyn Store<Error = CoreError>>` | Storage backend |

- **Returns:** `AdiyutantCoreService`
- **Errors:** never
- **Stable:** yes

---

### `with_time(store, time)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `store` | `Box<dyn Store<Error = CoreError>>` | Storage backend |
| `time` | `Box<dyn TimeProvider>` | Time source for deterministic testing |

- **Returns:** `AdiyutantCoreService`
- **Errors:** never
- **Stable:** yes

---

## Core Read Methods

### `get_today()`

- **Returns:** `TodayViewDto`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Builds a snapshot of today's state: daily log, check-ins, habits, plan items,
timers, reminders, alarms, context documents, and suggestions.

### `get_startup_state()`

- **Returns:** `StartupViewDto`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Determines what to show on launch: `start_day_required`, `offer_daily_schedule`,
`show_today_dashboard`, `suggest_evening_shutdown`, or `no_action`.

### `get_current_activity()`

- **Returns:** `CurrentActivityDto`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Determines what the user is (or should be) doing right now. Returns activity
kind, active task, and recommended prompt.

### `get_suggestions()`

- **Returns:** `Vec<String>`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Evaluates local rules and returns actionable suggestions.

---

## Planning Domain

### `add_plan_item(title, quadrant?, priority?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | `&str` | Item title |
| `quadrant` | `Option<&str>` | Eisenhower quadrant (`important-urgent`, `important-not-urgent`, `not-important-urgent`, `not-important-not-urgent`) |
| `priority` | `Option<u8>` | Priority (1–255, lower = higher) |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad quadrant), `STORAGE_ERROR`
- **Stable:** yes

Creates a plan item for today and auto-creates a `StartCheck` checkpoint.

### `list_plan_items()`

- **Returns:** `DayPlanDto`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

Lists today's plan items ordered by priority ascending.

### `start_plan_item(item_id)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `item_id` | `&str` | UUID of the plan item |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad UUID), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Sets item status to `Started` and creates a `ProgressCheck` checkpoint.
Auto-creates a `TaskStarted` journal entry.

### `done_plan_item(item_id)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `item_id` | `&str` | UUID of the plan item |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad UUID), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Sets item status to `Done`. Auto-creates a `TaskDone` journal entry.

### `move_to_waiting(item_id, review_days?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `item_id` | `&str` | UUID of the plan item |
| `review_days` | `Option<u64>` | Days until next review (default: 7) |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad UUID), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Sets item status to `Waiting` with a review date. Auto-creates a `TaskMoved`
journal entry.

### `list_waiting_tasks()`

- **Returns:** `Vec<WaitingTaskDto>`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

Lists all plan items in `Waiting` status that are due for review.

### `review_waiting(item_id, decision)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `item_id` | `&str` | UUID of the plan item |
| `decision` | `&str` | One of: `keep`, `resume`, `delete`, `archive` |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad UUID or unknown decision), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Applies the review decision to a waiting plan item.

---

## Checkpoint Domain

### `answer_checkpoint(checkpoint_id, response)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `checkpoint_id` | `&str` | UUID of the checkpoint |
| `response` | `&str` | User response text |

- **Returns:** `PlanItemDto`
- **Errors:** `INVALID_INPUT` (bad UUID, already answered), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Answers a pending checkpoint and transitions its status through the state
machine: `Pending → Shown → Answered`.

### `get_pending_checkpoint_notification()`

- **Returns:** `Option<NotificationInstructionDto>`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Finds the first pending checkpoint across all active plan items and returns a
notification instruction for the UI.

---

## Checklist Domain

### `list_checklist_templates(category?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `category` | `Option<&str>` | Filter by category (`morning`, `evening`, `shutdown`, `day`, `recovery`) |

- **Returns:** `Vec<ChecklistTemplateDto>`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

Lists available checklist templates, optionally filtered by category.

### `run_checklist(category)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `category` | `&str` | Checklist category |

- **Returns:** `ChecklistRunDto`
- **Errors:** `STORAGE_ERROR`, `INTERNAL_ERROR`
- **Stable:** yes

Starts a checklist run. Auto-seeds a default template if none exists for the
category.

### `answer_checklist_item(run_id, item_id, value, comment?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `run_id` | `&str` | UUID of the checklist run |
| `item_id` | `&str` | UUID of the checklist item |
| `value` | `&str` | Answer value |
| `comment` | `Option<&str>` | Optional comment |

- **Returns:** `ChecklistRunDto`
- **Errors:** `INVALID_INPUT`, `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Records an answer for a checklist item in an active run.

### `complete_checklist_run(run_id)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `run_id` | `&str` | UUID of the checklist run |

- **Returns:** `ChecklistRunDto`
- **Errors:** `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

Completes a checklist run: sets `completed_at`, serialises answers.
Auto-creates a `ChecklistCompleted` journal entry.

---

## Journal Domain

### `get_journal()`

- **Returns:** `Vec<JournalEntryDto>`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

Returns today's journal entries in chronological order.

### `add_journal_entry(entry_type, summary)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `entry_type` | `&str` | One of: `CheckInCreated`, `ChecklistCompleted`, `TaskStarted`, `TaskDone`, `TaskMoved`, `NudgeAnswered`, `Note`, `Shutdown` |
| `summary` | `&str` | Entry content |

- **Returns:** `JournalEntryDto`
- **Errors:** `INVALID_INPUT` (unknown entry type), `STORAGE_ERROR`
- **Stable:** yes

Creates a manual journal entry.

---

## Check-In

### `create_checkin(checkin_type, text, sleep_score?, energy?, mood?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `checkin_type` | `&str` | `morning`, `day`, `evening`, or `shutdown` |
| `text` | `&str` | Free-form check-in text |
| `sleep_score` | `Option<u8>` | Sleep quality (1–10) |
| `energy` | `Option<u8>` | Energy level (1–10) |
| `mood` | `Option<u8>` | Mood (1–10) |

- **Returns:** `String` (check-in ID)
- **Errors:** `INVALID_INPUT` (unknown type), `STORAGE_ERROR`
- **Stable:** yes

Creates a check-in, optionally updates daily log metrics, and auto-creates a
`CheckInCreated` journal entry.

---

## Habits

### `add_habit(name)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `name` | `&str` | Habit name |

- **Returns:** `String` (habit ID)
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

### `list_habits()`

- **Returns:** `Vec<(String, String, bool)>` — `(id, name, is_active)`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

### `mark_habit_done(habit_id, level?)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `habit_id` | `&str` | UUID of the habit |
| `level` | `Option<&str>` | `min`, `light`, `base` (default), `full` |

- **Returns:** `String` (event ID)
- **Errors:** `INVALID_INPUT` (bad UUID), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

### `skip_habit(habit_id)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `habit_id` | `&str` | UUID of the habit |

- **Returns:** `String` (event ID)
- **Errors:** `INVALID_INPUT` (bad UUID), `NOT_FOUND`, `STORAGE_ERROR`
- **Stable:** yes

---

## Timers

### `add_timer(title, duration_seconds)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | `&str` | Timer name |
| `duration_seconds` | `u64` | Duration in seconds |

- **Returns:** `String` (timer ID)
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

### `list_timers()`

- **Returns:** `Vec<(String, String, u64)>` — `(id, title, duration_seconds)`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

---

## Reminders

### `add_reminder(title, schedule_rule)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | `&str` | Reminder name |
| `schedule_rule` | `&str` | Rule string (e.g. `"every 2h"`) |

- **Returns:** `String` (reminder ID)
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

### `list_reminders()`

- **Returns:** `Vec<(String, String, String)>` — `(id, title, schedule_rule)`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

---

## Alarms

### `add_alarm(title, time)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `title` | `&str` | Alarm name |
| `time` | `&str` | Time in `HH:MM` format |

- **Returns:** `String` (alarm ID)
- **Errors:** `INVALID_INPUT` (bad time format), `STORAGE_ERROR`
- **Stable:** yes

### `list_alarms()`

- **Returns:** `Vec<(String, String, String)>` — `(id, title, time)`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

---

## Context Documents

### `add_context_document(doc_type, title, content)`

| Parameter | Type | Description |
|-----------|------|-------------|
| `doc_type` | `&str` | Document type |
| `title` | `&str` | Document title |
| `content` | `&str` | Markdown content |

- **Returns:** `String` (document ID)
- **Errors:** `INVALID_INPUT` (unknown doc_type), `STORAGE_ERROR`
- **Stable:** yes

### `list_context_documents()`

- **Returns:** `Vec<(String, String, String)>` — `(id, doc_type, title)`
- **Errors:** `STORAGE_ERROR`
- **Stable:** yes

---

## Error Contract

All methods above can return errors. Errors are mapped to `CoreErrorDto` via
`CoreError::to_dto()`:

| `code` | `recoverable` | `suggested_action` | When |
|---|---|---|---|
| `INVALID_INPUT` | `true` | `"Check input"` | Bad UUID, unknown enum value, malformed time |
| `NOT_FOUND` | `true` | `"Check item exists"` | Requested entity doesn't exist |
| `INTERNAL_ERROR` | `false` | `"Contact support"` | Unexpected internal state |
| `STORAGE_ERROR` | `false` | `"Retry or check disk"` | SQLite I/O failure |

---

## Stability Guarantees

1. **Method additions** — backward compatible.
2. **DTO field additions** — backward compatible (new fields at end of struct).
3. **DTO field renames or removals** — breaking change (version bump).
4. **Method signature changes** — breaking change (version bump).
5. **New error codes** — backward compatible (consumers must handle unknown codes).

---

## Version History

| Version | Date | Changes |
|---------|------|---------|
| v0.3.0 | 2026-06 | Initial API contract document. Added checkpoint, checklist, journal events. |
