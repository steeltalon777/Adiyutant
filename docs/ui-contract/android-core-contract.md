# Android-Core UI Contract — v0.2.0

> Defines the binding between Android screens and `AdiyutantCoreService`.
> Every screen maps to exactly one primary DTO and a fixed call sequence.
> Android must not access `Store` or domain models directly.

---

## TZ

- `/home/makc/AI_sandbox/ADIYUTANT/docs/tz/phase-7.1-core-api-contract-hardening.md`

---

## Table of Contents

1. [Startup Screen](#1-startup-screen)
2. [Today Dashboard](#2-today-dashboard)
3. [Current Activity](#3-current-activity)
4. [Plan Screen](#4-plan-screen)
5. [Waiting Review](#5-waiting-review)
6. [Checklist Run](#6-checklist-run)
7. [Journal Feed](#7-journal-feed)
8. [State Glossary](#state-glossary)
9. [Error DTO and Recovery Flows](#error-dto-and-recovery-flows)
10. [DTO Reference](#dto-reference)

---

## 1. Startup Screen

**DTO:** `StartupViewDto`
**Primary method:** `get_startup_state() → StartupViewDto`
**Follow-up methods:** depends on `intent` value (see states below)

### Call sequence

```
1. service.get_startup_state()
2. Android reads intent → navigates accordingly
```

### States

| `intent` value | Trigger condition | UI behavior |
|---|---|---|
| `start_day_required` | No morning check-in exists for today | Show "Start your day" with morning check-in form |
| `offer_daily_schedule` | Morning check-in done, no plan items exist | Show "Create plan for today" with plan button |
| `show_today_dashboard` | Morning done + plan exists + hour ∈ [6, 20) | Navigate directly to Today Dashboard |
| `suggest_evening_shutdown` | Morning done + plan exists + hour ∉ [6, 20) + no evening check-in | Show "Evening shutdown" prompt |
| `no_action` | Fallback (all done, evening check-in exists) | Show Today Dashboard |

### DTO example

```
{
  "intent": "start_day_required",
  "reason": "No morning check-in yet. Start your day by checking in."
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `STORAGE_ERROR` | Show retry banner; `suggested_action: "Retry or check disk"` |
| `INTERNAL_ERROR` | Show error screen; `suggested_action: "Contact support"` |

---

## 2. Today Dashboard

**DTO:** `TodayViewDto`
**Primary method:** `get_today() → TodayViewDto`
**Supporting methods:** `get_suggestions() → Vec<String>`

### Call sequence

```
1. service.get_today()
2. service.get_suggestions()   (optional, for suggestion badge count)
```

### States

| Condition | UI behavior |
|---|---|
| `has_morning_check_in: false` | Show morning check-in prompt overlay |
| `has_morning_check_in: true`, `has_plan: false` | Show "Create plan" CTA |
| `has_plan: true`, `plan_item_count > 0` | Render plan items list with status chips |
| `habit_events_done < habit_count` | Show habit progress bar (partial) |
| `habit_events_done == habit_count` | Show habit progress bar (complete) |
| `suggestion_count > 0` | Show suggestion badge |

### DTO example

```
{
  "date": "2026-06-09",
  "has_daily_log": true,
  "mode": "none",
  "sleep_score": "7",
  "energy": "6",
  "mood": "8",
  "check_in_count": 1,
  "has_morning_check_in": true,
  "has_evening_check_in": false,
  "habit_count": 3,
  "habit_events_done": 1,
  "has_plan": true,
  "plan_title": "Plan for 2026-06-09",
  "plan_item_count": 2,
  "plan_items": [
    {
      "description": "Write API docs",
      "status": "Planned",
      "estimated_minutes": "",
      "notes": "Phase 7.1"
    },
    {
      "description": "Review PR",
      "status": "Started",
      "estimated_minutes": "",
      "notes": ""
    }
  ],
  "suggestion_count": 2
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `STORAGE_ERROR` | Show cached last-known state + retry button |
| `INTERNAL_ERROR` | Show error screen; `suggested_action: "Contact support"` |

---

## 3. Current Activity

**DTO:** `CurrentActivityDto`
**Primary method:** `get_current_activity() → CurrentActivityDto`

### Call sequence

```
1. service.get_current_activity()
2. Android renders prompt + activity indicator
```

### States

| `activity_kind` value | Trigger condition | UI behavior |
|---|---|---|
| `start_day` | No morning check-in | Show "Start your day" prompt with morning check-in button |
| `no_plan` | Morning done, no plan exists | Show "Create a plan" CTA |
| `scheduled_task` | Plan has a Started item, or pending items exist | Show active task title + `recommended_prompt`; if pending, show "Ready to start" |
| `recovery` | Daily log `mode == "recovery"` + no plan | Show "Rest and recharge" message |
| `shutdown` | Hour ∉ [6, 20) + no evening check-in + all tasks done | Show "Evening shutdown" prompt |
| `free_time` | All tasks done, not late, or evening check-in exists | Show "No active task" with optional suggestions |
| `sleep_window` | Reserved for future deep-night logic | (currently unreachable; treat as `shutdown`) |
| `break` | Reserved for future break scheduling | (currently unreachable; treat as `free_time`) |
| `meal` | Reserved for future meal reminders | (currently unreachable; treat as `free_time`) |

### DTO example

```
{
  "date": "2026-06-09",
  "activity_kind": "scheduled_task",
  "active_task": "Write API docs",
  "active_block": "",
  "next_checkpoint_at": "",
  "recommended_prompt": "Working on: Write API docs"
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `STORAGE_ERROR` | Show last-known activity + retry |
| `INTERNAL_ERROR` | Show error screen |

---

## 4. Plan Screen

**DTO:** `DayPlanDto` (list), `PlanItemDto` (item)
**Primary methods:**
- `list_plan_items() → DayPlanDto`
- `add_plan_item(title, quadrant?, priority?) → PlanItemDto`
- `start_plan_item(item_id) → PlanItemDto`
- `done_plan_item(item_id) → PlanItemDto`
- `move_to_waiting(item_id, review_days?) → PlanItemDto`

### Call sequence

```
1. service.list_plan_items()                  → render list
2. service.add_plan_item(title, q, p)         → on "Add" action
3. service.start_plan_item(item_id)           → on "Start" action
4. service.done_plan_item(item_id)            → on "Done" action
5. service.move_to_waiting(item_id, days)     → on "Waiting" action
```

### States (PlanItemDto.status)

| `status` value | UI chip | Available actions |
|---|---|---|
| `Planned` | Gray / blue | Start, Move to Waiting |
| `Started` | Green / active | Done, Move to Waiting |
| `Done` | Checkmark | (none, or undo) |
| `Waiting` | Yellow / clock | (managed in Waiting Review screen) |
| `Archived` | Dimmed | (read-only) |
| `Deleted` | (hidden) | (not shown) |

### DTO example — DayPlanDto

```
{
  "id": "",
  "date": "2026-06-09",
  "items": [
    {
      "id": "a1b2c3d4-...",
      "title": "Write API docs",
      "description": "Phase 7.1 contract",
      "quadrant": "important-urgent",
      "planned_start": "09:00",
      "planned_end": "11:00",
      "status": "Started",
      "priority": 1,
      "source": "Manual",
      "created_at": "2026-06-09T08:00:00Z",
      "updated_at": "2026-06-09T09:00:00Z"
    }
  ],
  "item_count": 1
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `INVALID_INPUT` | Show field-level error; `suggested_action: "Check input"` |
| `NOT_FOUND` | Refresh list; item may have been deleted; `suggested_action: "Check item exists"` |
| `STORAGE_ERROR` | Retry banner |

---

## 5. Waiting Review

**DTO:** `Vec<WaitingTaskDto>` (list), `PlanItemDto` (after decision)
**Primary methods:**
- `list_waiting_tasks() → Vec<WaitingTaskDto>`
- `review_waiting(item_id, decision) → PlanItemDto`

### Call sequence

```
1. service.list_waiting_tasks()               → render waiting list
2. service.review_waiting(item_id, decision)  → on user decision
   decision ∈ { "keep", "resume", "delete", "archive" }
3. service.list_waiting_tasks()               → refresh list after decision
```

### States

| Condition | UI behavior |
|---|---|
| Empty list | Show "No waiting tasks" empty state |
| `review_due` date ≤ today | Highlight row as "Due for review" |
| `review_due` date > today | Show countdown "Review in N days" |

### Decision effects

| `decision` | Effect on PlanItem |
|---|---|
| `keep` | Extends `waiting_review_at` by 7 days, stays `Waiting` |
| `resume` | Sets status back to `Planned`, clears `waiting_review_at` |
| `delete` | Removes item from store permanently |
| `archive` | Sets status to `Archived`, clears `waiting_review_at` |

### DTO example — WaitingTaskDto

```
{
  "id": "a1b2c3d4-...",
  "title": "Wait for design feedback",
  "waiting_since": "2026-06-01T10:00:00Z",
  "review_due": "2026-06-08"
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `INVALID_INPUT` | Show error for unknown decision value |
| `NOT_FOUND` | Refresh list; item gone |
| `STORAGE_ERROR` | Retry banner |

---

## 6. Checklist Run

**DTO:** `ChecklistTemplateDto` (template list), `ChecklistRunDto` (active run)
**Primary methods:**
- `list_checklist_templates(category?) → Vec<ChecklistTemplateDto>`
- `run_checklist(category) → ChecklistRunDto`

### Call sequence

```
1. service.list_checklist_templates()         → show available checklists
2. service.run_checklist("morning")           → start a run
3. (future P2: answer_checklist_item() × N)
4. (future P2: complete_checklist_run())
```

### States

| Condition | UI behavior |
|---|---|
| `ChecklistRunDto.completed_at` is empty | Run is in progress; show items to answer |
| `ChecklistRunDto.completed_at` is non-empty | Run is complete; show summary |
| `answer_count == 0` | No answers yet; show first question |
| `answer_count > 0` | Show progress indicator |

### Categories

| Category | Template title | Items |
|---|---|---|
| `morning` | Morning Check-in | 4 items (sleep, energy, mood, notes) |
| `evening` | Evening Review | 3 items (day review, went well, improve) |
| `shutdown` | Shutdown Routine | 4 items (review, plan tomorrow, alarms, wind down) |
| `day` | Mid-day Check | 3 items (energy, on track, adjust) |
| `recovery` | Recovery Check | 3 items (feeling, rest, ready) |

### DTO example — ChecklistRunDto

```
{
  "id": "run-uuid-...",
  "template_title": "Morning Check-in",
  "started_at": "2026-06-09T07:00:00Z",
  "completed_at": "",
  "answer_count": 0
}
```

### DTO example — ChecklistTemplateDto

```
{
  "id": "tmpl-uuid-...",
  "title": "Morning Check-in",
  "category": "morning",
  "item_count": 4,
  "is_active": true
}
```

### Error handling

| Error code | Recovery |
|---|---|
| `NOT_FOUND` | Unknown category; show available categories |
| `STORAGE_ERROR` | Retry banner |
| `INTERNAL_ERROR` | Error screen |

---

## 7. Journal Feed

**DTO:** `Vec<JournalEntryDto>`
**Primary methods:**
- `get_journal() → Vec<JournalEntryDto>`
- `add_journal_entry(entry_type, summary) → JournalEntryDto`

### Call sequence

```
1. service.get_journal()                      → render feed (newest first)
2. service.add_journal_entry("Note", "...")   → on manual note creation
```

### States

| Condition | UI behavior |
|---|---|
| Empty list | Show "No journal entries yet" empty state |
| Entries exist | Render chronological feed grouped by entry_type |

### Entry types

| `entry_type` value | Source | Auto-generated? |
|---|---|---|
| `CheckInCreated` | `create_checkin()` | Yes |
| `TaskStarted` | `start_plan_item()` | Yes (future P2) |
| `TaskDone` | `done_plan_item()` | Yes (future P2) |
| `TaskMoved` | `move_to_waiting()` | Yes (future P2) |
| `ChecklistCompleted` | `complete_checklist_run()` | Yes (future P2) |
| `NudgeAnswered` | (future P2: checkpoint answer) | Yes |
| `Note` | `add_journal_entry()` or manual | Manual |
| `Shutdown` | (future: shutdown flow) | Yes |

### DTO example

```
[
  {
    "id": "journal-uuid-1",
    "entry_type": "CheckInCreated",
    "summary": "morning check-in: Feeling good today",
    "timestamp": "2026-06-09T07:00:00Z"
  },
  {
    "id": "journal-uuid-2",
    "entry_type": "TaskStarted",
    "summary": "Started: Write API docs",
    "timestamp": "2026-06-09T09:00:00Z"
  },
  {
    "id": "journal-uuid-3",
    "entry_type": "Note",
    "summary": "Remember to call dentist",
    "timestamp": "2026-06-09T12:30:00Z"
  }
]
```

### Error handling

| Error code | Recovery |
|---|---|
| `INVALID_INPUT` | Unknown entry_type; show valid types |
| `STORAGE_ERROR` | Retry banner |
| `INTERNAL_ERROR` | Error screen |

---

## State Glossary

All 8 states that drive UI behavior, mapped to their source DTOs and fields.

### 1. `start_day_required`

- **Source:** `StartupViewDto.intent` or `CurrentActivityDto.activity_kind == "start_day"`
- **Meaning:** No morning check-in exists for today. The user must check in before anything else.
- **UI:** Full-screen morning check-in form.
- **Exit condition:** `create_checkin("morning", ...)` succeeds → state transitions to `offer_daily_schedule` or `show_today_dashboard`.

### 2. `offer_daily_schedule`

- **Source:** `StartupViewDto.intent == "offer_daily_schedule"`
- **Meaning:** Morning check-in is done, but no plan items exist for today.
- **UI:** "Create plan for today" prompt with CTA button.
- **Exit condition:** `add_plan_item(...)` creates first item → state transitions to `scheduled_task` on next activity check.

### 3. `scheduled_task`

- **Source:** `CurrentActivityDto.activity_kind == "scheduled_task"`
- **Meaning:** A plan item is either Started (in progress) or Planned (ready to start).
- **UI:** Active task card with title, `recommended_prompt`, and action buttons (Done, Waiting).
- **Exit condition:** `done_plan_item()` or `move_to_waiting()` → re-evaluates to next task or `free_time`.

### 4. `no_plan`

- **Source:** `CurrentActivityDto.activity_kind == "no_plan"`
- **Meaning:** Morning check-in done, no plan exists, not in recovery mode.
- **UI:** "No plan for today" message with "Create plan" CTA.
- **Exit condition:** `add_plan_item(...)` → transitions to `scheduled_task`.

### 5. `waiting`

- **Source:** `PlanItemDto.status == "Waiting"` (visible in Waiting Review screen)
- **Meaning:** A task is paused, waiting for external input or a review date.
- **UI:** Yellow chip in Plan Screen; full card in Waiting Review with `review_due` date.
- **Exit condition:** `review_waiting(item_id, decision)` with `resume` → back to `Planned`; `keep` → extends review; `delete`/`archive` → removed from waiting.

### 6. `shutdown`

- **Source:** `CurrentActivityDto.activity_kind == "shutdown"`
- **Meaning:** Late hour (∉ [6, 20)), no evening check-in, tasks are done.
- **UI:** "Evening shutdown" prompt with shutdown checklist CTA.
- **Exit condition:** `run_checklist("shutdown")` + `create_checkin("shutdown", ...)` → day concludes.

### 7. `recovery`

- **Source:** `CurrentActivityDto.activity_kind == "recovery"`
- **Meaning:** Daily log `mode == "recovery"`, no plan. Low-energy day.
- **UI:** "Rest and recharge" message, minimal UI, no task pressure.
- **Exit condition:** User changes daily log mode or creates a plan → transitions to `scheduled_task` or `no_plan`.

### 8. `free_time`

- **Source:** `CurrentActivityDto.activity_kind == "free_time"`
- **Meaning:** All tasks done, not late, or evening check-in exists. No active obligation.
- **UI:** "No active task" with optional suggestions from `get_suggestions()`.
- **Exit condition:** New `add_plan_item(...)` + `start_plan_item(...)` → transitions to `scheduled_task`.

---

## Error DTO and Recovery Flows

### CoreErrorDto structure

All errors from Core are mapped to `CoreErrorDto`:

```
{
  "code": "NOT_FOUND",
  "message": "not found: plan_item abc-123",
  "recoverable": true,
  "suggested_action": "Check item exists"
}
```

### Error codes

| `code` | `recoverable` | `suggested_action` | When |
|---|---|---|---|
| `INVALID_INPUT` | `true` | `"Check input"` | Bad UUID, unknown enum value, malformed time, unknown quadrant/decision/checkin-type |
| `NOT_FOUND` | `true` | `"Check item exists"` | Requested entity (plan_item, habit, checklist template) doesn't exist |
| `INTERNAL_ERROR` | `false` | `"Contact support"` | Unexpected internal state, logic bug |
| `STORAGE_ERROR` | `false` | `"Retry or check disk"` | SQLite I/O failure, disk full, migration error |

### Recovery flows by error code

#### INVALID_INPUT (recoverable)

1. Android shows field-level validation error.
2. User corrects input.
3. Retry the same call with corrected parameters.

Common triggers:
- Malformed UUID in `item_id` → validate UUID format before calling
- Unknown `quadrant` value → use only: `"important-urgent"`, `"important-not-urgent"`, `"not-important-urgent"`, `"not-important-not-urgent"`
- Unknown `decision` value → use only: `"keep"`, `"resume"`, `"delete"`, `"archive"`
- Unknown `checkin_type` → use only: `"morning"`, `"day"`, `"evening"`, `"shutdown"`
- Unknown `entry_type` → use only: `"CheckInCreated"`, `"ChecklistCompleted"`, `"TaskStarted"`, `"TaskDone"`, `"TaskMoved"`, `"NudgeAnswered"`, `"Note"`, `"Shutdown"`
- Malformed time in `add_alarm()` → use `HH:MM` format

#### NOT_FOUND (recoverable)

1. Android refreshes the list from Core (`list_plan_items()`, `list_waiting_tasks()`, etc.).
2. If item is gone, removes it from local UI state.
3. Shows "Item no longer available" toast.

Common triggers:
- `start_plan_item(id)` / `done_plan_item(id)` / `move_to_waiting(id)` with stale ID
- `review_waiting(id, ...)` with stale ID
- `mark_habit_done(id)` / `skip_habit(id)` with deleted habit ID

#### STORAGE_ERROR (non-recoverable)

1. Android shows persistent "Storage error" banner.
2. Offers "Retry" button that re-calls the failed method.
3. If retry fails 3 times, shows "Check disk space" guidance.
4. Does NOT auto-retry (could worsen disk issues).

#### INTERNAL_ERROR (non-recoverable)

1. Android shows full-screen error state.
2. Logs the `message` field for bug reports.
3. Offers "Contact support" action.
4. Does NOT retry (indicates logic bug).

### Global error handling rules

1. **Never crash on Core error.** Every Core call must be wrapped in error handling.
2. **Check `recoverable` field** to decide retry vs. escalate.
3. **Display `suggested_action`** to the user when showing error UI.
4. **Log `code` + `message`** for diagnostics.
5. **Preserve UI state** on error — do not clear forms or lists unless the error confirms the data is gone (`NOT_FOUND`).

---

## DTO Reference

### Summary table

| DTO | Used by screen | Key fields |
|---|---|---|
| `StartupViewDto` | Startup | `intent`, `reason` |
| `TodayViewDto` | Today Dashboard | `date`, `has_morning_check_in`, `has_plan`, `plan_items[]`, `habit_count`, `habit_events_done` |
| `CurrentActivityDto` | Current Activity | `activity_kind`, `active_task`, `recommended_prompt` |
| `DayPlanDto` | Plan Screen | `date`, `items[]`, `item_count` |
| `PlanItemDto` | Plan Screen, Waiting Review | `id`, `title`, `status`, `quadrant`, `priority` |
| `WaitingTaskDto` | Waiting Review | `id`, `title`, `waiting_since`, `review_due` |
| `ChecklistTemplateDto` | Checklist Run | `id`, `title`, `category`, `item_count`, `is_active` |
| `ChecklistRunDto` | Checklist Run | `id`, `template_title`, `started_at`, `completed_at`, `answer_count` |
| `JournalEntryDto` | Journal Feed | `id`, `entry_type`, `summary`, `timestamp` |
| `CoreErrorDto` | All screens | `code`, `message`, `recoverable`, `suggested_action` |
| `NotificationInstructionDto` | (future: notifications) | `source`, `title`, `body`, `action_id` |
| `PlanItemViewDto` | Today Dashboard (nested) | `description`, `status`, `estimated_minutes`, `notes` |

### Field type conventions

- All IDs: UUID strings (`"a1b2c3d4-..."`)
- All dates: ISO-8601 (`"2026-06-09"`)
- All timestamps: ISO-8601 with timezone (`"2026-06-09T07:00:00Z"`)
- All times: `HH:MM` format (`"09:00"`)
- Empty optional strings: `""` (not null)
- Numeric scores: string-encoded (`"7"`, not `7`)
- Boolean flags: native JSON booleans

---

## Stability Guarantees

1. **DTO field additions** — backward compatible (new fields at end, deserialization tolerant).
2. **DTO field renames** — breaking change (requires version bump).
3. **New methods** — backward compatible.
4. **Method signature changes** — breaking change (requires version bump).
5. **New `intent` / `activity_kind` values** — backward compatible (Android must handle unknown values gracefully with a fallback UI).
6. **New `entry_type` values** — backward compatible (Android must render unknown types as generic entries).
