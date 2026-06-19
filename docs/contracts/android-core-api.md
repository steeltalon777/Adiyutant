# Android-Core API Contract — Phase 8.2 Draft

## DTO Catalog

| DTO | Fields |
|-----|--------|
| TodayViewDto | date, has_daily_log, mode, sleep_score, energy, mood, check_in_count, has_morning_check_in, has_evening_check_in, habit_count, habit_events_done, has_plan, plan_title, plan_item_count, plan_items, suggestion_count |
| StartupViewDto | intent, reason |
| CurrentActivityDto | activity_kind, active_task, active_block, next_checkpoint_at, recommended_prompt, date |
| DayPlanDto | id, date, items, item_count |
| PlanItemDto | id, title, description, quadrant, planned_start, planned_end, status, priority, source, created_at, updated_at |
| WaitingTaskDto | id, title, waiting_since, review_due |
| ChecklistTemplateDto | id, title, category, item_count, is_active |
| ChecklistRunDto | id, template_title, started_at, completed_at, answer_count |
| JournalEntryDto | id, entry_type, summary, timestamp |
| NotificationInstructionDto | source, title, body, action_id |
| CoreErrorDto | code, message, recoverable, suggested_action |
| HabitDto | id, name, is_active |
| HabitEventDto | id, habit_id, habit_name, status, level, date_time |
| TimerDto | id, title, duration_seconds, mode |
| ReminderDto | id, title, schedule_rule |
| AlarmDto | id, title, time |
| ContextDocumentDto | id, doc_type, title, content |
| CheckInDto | id, checkin_type, text, created_at |
| SuggestionDto | proposal_type, reason, suggestion |

## Methods

| Method | Returns | Android Screen |
|--------|---------|---------------|
| add_habit(name) | HabitDto | Habits |
| list_habits() | Vec\<HabitDto\> | Habits |
| mark_habit_done(id, level?) | HabitEventDto | Habit detail |
| skip_habit(id) | HabitEventDto | Habit detail |
| add_timer(title, secs) | TimerDto | Timers |
| list_timers() | Vec\<TimerDto\> | Timers |
| add_reminder(title, rule) | ReminderDto | Reminders |
| list_reminders() | Vec\<ReminderDto\> | Reminders |
| add_alarm(title, time) | AlarmDto | Alarms |
| list_alarms() | Vec\<AlarmDto\> | Alarms |
| add_context_document(type, title, content) | ContextDocumentDto | Context |
| list_context_documents() | Vec\<ContextDocumentDto\> | Context |
| create_checkin(type, text, ...) | CheckInDto | Check-in |
| get_suggestions() | Vec\<SuggestionDto\> | Today dashboard |

### Startup / Today (existing)

| Method | Returns | Android Screen |
|--------|---------|---------------|
| get_startup_state() | StartupViewDto | Startup |
| get_today() | TodayViewDto | Today Dashboard |
| get_current_activity() | CurrentActivityDto | Current Activity |

### Planning (existing)

| Method | Returns | Android Screen |
|--------|---------|---------------|
| list_plan_items() | DayPlanDto | Plan |
| add_plan_item(title, quadrant?, priority?) | PlanItemDto | Plan |
| start_plan_item(id) | PlanItemDto | Plan |
| done_plan_item(id) | PlanItemDto | Plan |
| move_to_waiting(id, review_days?) | PlanItemDto | Plan |
| list_waiting_tasks() | Vec\<WaitingTaskDto\> | Waiting Review |
| review_waiting(id, decision) | PlanItemDto | Waiting Review |

### Checklist / Journal (existing)

| Method | Returns | Android Screen |
|--------|---------|---------------|
| list_checklist_templates(category?) | Vec\<ChecklistTemplateDto\> | Checklists |
| run_checklist(category) | ChecklistRunDto | Checklist Run |
| answer_checklist_item(run_id, item_id, value, comment?) | ChecklistRunDto | Checklist Run |
| complete_checklist_run(run_id) | ChecklistRunDto | Checklist Run |
| get_journal() | Vec\<JournalEntryDto\> | Journal |
| add_journal_entry(type, summary) | JournalEntryDto | Journal |

### Checkpoint (existing)

| Method | Returns | Android Screen |
|--------|---------|---------------|
| get_pending_checkpoint_notification() | Option\<NotificationInstructionDto\> | Notifications |
| answer_checkpoint(id, response) | PlanItemDto | Checkpoint modal |
| dismiss_checkpoint(id) | Result\<(), CoreError\> | Checkpoint modal |

## Contract Examples

Golden JSON examples for each screen are available in `docs/contracts/examples/`.
These represent stable DTO shapes guaranteed by the Core API.

| Example file | Screen |
|---|---|
| `startup-state.json` | Startup Screen |
| `today-dashboard.json` | Today Dashboard |
| `current-activity.json` | Current Activity |
| `plan-item-lifecycle.json` | Plan Screen (create → start → done) |
| `waiting-review.json` | Waiting Review |
| `checklist-run.json` | Checklist Run |
| `journal-feed.json` | Journal Feed |
