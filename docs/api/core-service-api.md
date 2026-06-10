# Core Service API — Public Contract v0.2.0

> Stable public API of `AdiyutantCoreService`. Android shell and all external consumers depend only on the methods and DTOs documented here.

---

## AdiyutantCoreService

**Constructor:** `AdiyutantCoreService::new(store: Box<dyn Store<Error = CoreError>>)`

### Startup & Dashboard

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `get_startup_state()` | `StartupViewDto` | `INTERNAL_ERROR`, `STORAGE_ERROR` | ✅ |
| `get_today()` | `TodayViewDto` | `INTERNAL_ERROR`, `STORAGE_ERROR` | ✅ |
| `get_current_activity()` | `CurrentActivityDto` | `INTERNAL_ERROR`, `STORAGE_ERROR` | ✅ |

### Check-In

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `create_checkin(type, text, sleep, energy, mood)` | `String` (check-in ID) | `INVALID_INPUT`, `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |

### Planning

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `add_plan_item(title, quadrant?, priority?)` | `PlanItemDto` | `INVALID_INPUT`, `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |
| `list_plan_items()` | `DayPlanDto` | `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |
| `start_plan_item(item_id)` | `PlanItemDto` | `NOT_FOUND`, `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |
| `done_plan_item(item_id)` | `PlanItemDto` | `NOT_FOUND`, `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |
| `move_to_waiting(item_id, review_days?)` | `PlanItemDto` | `NOT_FOUND`, `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |

### Waiting Review

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `list_waiting_tasks()` | `Vec<WaitingTaskDto>` | `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |
| `review_waiting(item_id, decision)` | `PlanItemDto` | `NOT_FOUND`, `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |

### Habits

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `add_habit(name)` | `String` (habit ID) | `STORAGE_ERROR` | ✅ |
| `list_habits()` | `Vec<(id, name, active)>` | `STORAGE_ERROR` | ✅ |
| `mark_habit_done(habit_id, level?)` | `String` (event ID) | `NOT_FOUND`, `STORAGE_ERROR` | ✅ |
| `skip_habit(habit_id)` | `String` (event ID) | `NOT_FOUND`, `STORAGE_ERROR` | ✅ |

### Timers, Reminders, Alarms

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `add_timer(title, duration_seconds)` | `String` (timer ID) | `STORAGE_ERROR` | ✅ |
| `list_timers()` | `Vec<(id, title, seconds)>` | `STORAGE_ERROR` | ✅ |
| `add_reminder(title, schedule_rule)` | `String` (reminder ID) | `STORAGE_ERROR` | ✅ |
| `list_reminders()` | `Vec<(id, title, rule)>` | `STORAGE_ERROR` | ✅ |
| `add_alarm(title, time)` | `String` (alarm ID) | `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |
| `list_alarms()` | `Vec<(id, title, time)>` | `STORAGE_ERROR` | ✅ |

### Context Documents

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `add_context_document(doc_type, title, content)` | `String` (doc ID) | `INVALID_INPUT`, `STORAGE_ERROR` | ✅ |
| `list_context_documents()` | `Vec<(id, type, title)>` | `STORAGE_ERROR` | ✅ |

### Checklists

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `list_checklist_templates(category?)` | `Vec<ChecklistTemplateDto>` | `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |
| `run_checklist(category)` | `ChecklistRunDto` | `NOT_FOUND`, `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |

### Journal

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `get_journal()` | `Vec<JournalEntryDto>` | `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |
| `add_journal_entry(entry_type, summary)` | `JournalEntryDto` | `INVALID_INPUT`, `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |

### Suggestions

| Method | Returns | Errors | Stable |
|--------|---------|--------|--------|
| `get_suggestions()` | `Vec<String>` | `STORAGE_ERROR`, `INTERNAL_ERROR` | ✅ |

---

## Error Codes

All errors return `CoreError` in Rust. External consumers receive `CoreErrorDto`:

| code | recoverable | suggested_action | When |
|------|-------------|------------------|------|
| `INVALID_INPUT` | ✅ true | `Check input` | Bad UUID, unknown enum value, malformed time |
| `NOT_FOUND` | ✅ true | `Check item exists` | Requested entity doesn't exist |
| `INTERNAL_ERROR` | ❌ false | `Contact support` | Unexpected internal state |
| `STORAGE_ERROR` | ❌ false | `Retry or check disk` | SQLite I/O failure, migration error |

---

## DTO Reference

All DTOs are flat, stable structures with `Serialize`/`Deserialize` support. No `Id<T>`, no `serde_json::Value`, no chrono inner types. Dates/times are ISO-8601 strings.

### StartupViewDto

```json
{
  "intent": "start_day_required",
  "reason": "No morning check-in yet."
}
```

**intent values:** `start_day_required`, `offer_daily_schedule`, `show_today_dashboard`, `suggest_evening_shutdown`, `no_action`

### CurrentActivityDto

```json
{
  "date": "2026-06-09",
  "activity_kind": "scheduled_task",
  "active_task": "Android shell",
  "active_block": "",
  "next_checkpoint_at": "12:00",
  "recommended_prompt": "Ready to start: Android shell"
}
```

**activity_kind values:** `start_day`, `no_plan`, `scheduled_task`, `free_time`, `recovery`, `shutdown`, `sleep_window`

### TodayViewDto

```json
{
  "date": "2026-06-09",
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
      "description": "Task A",
      "status": "Planned",
      "estimated_minutes": "",
      "notes": ""
    }
  ],
  "suggestion_count": 2
}
```

### CoreErrorDto

```json
{
  "code": "NOT_FOUND",
  "message": "not found: plan_item abc-123",
  "recoverable": true,
  "suggested_action": "Check item exists"
}
```

---

## Stability Guarantees

1. **DTO field additions** — backward compatible (new fields at end, deserialization tolerant)
2. **DTO field renames** — breaking change (requires version bump)
3. **New methods** — backward compatible
4. **Method signature changes** — breaking change (requires version bump)
5. **Error code additions** — backward compatible

All `Stable: ✅` methods have passed Phase 7 acceptance testing and are safe for Android consumption.
