use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::model::check_in::CheckInType;
use crate::model::context_document::ContextDocumentType;
use crate::model::habit_event::HabitEventLevel;
use crate::model::timer::TimerMode;

// ──────────────────────────────────────────────
// Primary DTOs — flat, stable output models
// Rules: no Id<T>, no serde_json::Value, no chrono inner types.
// All dates/times are ISO-8601 strings.
// ──────────────────────────────────────────────

/// View model for the "today" dashboard.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TodayViewDto {
    pub date: String,
    pub has_daily_log: bool,
    pub mode: String,
    pub sleep_score: String,
    pub energy: String,
    pub mood: String,
    pub check_in_count: usize,
    pub has_morning_check_in: bool,
    pub has_evening_check_in: bool,
    pub habit_count: usize,
    pub habit_events_done: usize,
    pub has_plan: bool,
    pub plan_title: String,
    pub plan_item_count: usize,
    pub plan_items: Vec<PlanItemViewDto>,
    pub suggestion_count: usize,
}

/// Flat view of a single plan item (legacy Plan model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItemViewDto {
    pub description: String,
    pub status: String,
    pub estimated_minutes: String,
    pub notes: String,
}

/// View model for the startup screen decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupViewDto {
    pub intent: String,
    pub reason: String,
}

/// View model for the "current activity" central screen.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CurrentActivityDto {
    pub activity_kind: String,
    pub active_task: String,
    pub active_block: String,
    pub next_checkpoint_at: String,
    pub recommended_prompt: String,
    pub date: String,
}

// ─── Wave B: Planning DTOs ─────────────────────

/// View model for a day plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DayPlanDto {
    pub id: String,
    pub date: String,
    pub items: Vec<PlanItemDto>,
    pub item_count: usize,
}

/// View model for a single plan item (new PlanItem model).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanItemDto {
    pub id: String,
    pub title: String,
    pub description: String,
    pub quadrant: String,
    pub planned_start: String,
    pub planned_end: String,
    pub status: String,
    pub priority: u8,
    pub source: String,
    pub created_at: String,
    pub updated_at: String,
}

/// View model for a waiting task.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WaitingTaskDto {
    pub id: String,
    pub title: String,
    pub waiting_since: String,
    pub review_due: String,
}

/// Instruction for the UI layer to display a notification.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationInstructionDto {
    pub source: String,
    pub title: String,
    pub body: String,
    pub action_id: String,
}

/// Structured error for external consumers (Android, CLI, contract tests).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreErrorDto {
    /// Machine-readable error code: "NOT_FOUND", "INVALID_INPUT", "STORAGE_ERROR", "INTERNAL".
    pub code: String,
    /// Human-readable message for developer logs.
    pub message: String,
    /// Whether the client can recover (retry, change input, fix state).
    pub recoverable: bool,
    /// Suggested action for the client: "Retry", "Check input", "Contact support", etc.
    pub suggested_action: String,
}

// ─── Wave C: Checklist & Journal DTOs ─────────

/// View model for a checklist template.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistTemplateDto {
    pub id: String,
    pub title: String,
    pub category: String,
    pub item_count: usize,
    pub is_active: bool,
}

/// View model for a checklist run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistRunDto {
    pub id: String,
    pub template_title: String,
    pub started_at: String,
    pub completed_at: String,
    pub answer_count: usize,
}

/// View model for a journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntryDto {
    pub id: String,
    pub entry_type: String,
    pub summary: String,
    pub timestamp: String,
}

// ─── Wave D: Habit, Timer, Reminder, Alarm DTOs ──

/// View model for a habit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitDto {
    pub id: String,
    pub name: String,
    pub is_active: bool,
}

/// View model for a habit event (done/skip).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HabitEventDto {
    pub id: String,
    pub habit_id: String,
    pub habit_name: String,
    /// "Done" / "Skipped"
    pub status: String,
    /// "Min" / "Light" / "Base" / "Full" — for Done events; "Min" for Skipped
    pub level: String,
    /// ISO-8601 timestamp
    pub date_time: String,
}

/// View model for a timer definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimerDto {
    pub id: String,
    pub title: String,
    pub duration_seconds: u64,
    /// "Focus" / "Break" / "Custom"
    pub mode: String,
}

/// View model for a reminder definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReminderDto {
    pub id: String,
    pub title: String,
    pub schedule_rule: String,
}

/// View model for an alarm definition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlarmDto {
    pub id: String,
    pub title: String,
    /// "HH:MM" format
    pub time: String,
}

// ─── Wave E: Context, CheckIn, Suggestion DTOs ──

/// View model for a context document.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextDocumentDto {
    pub id: String,
    /// "core", "goals", "rules", "routine", "health", "work", "custom", etc.
    pub doc_type: String,
    pub title: String,
    /// Full markdown content
    pub content: String,
}

/// View model for a check-in.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckInDto {
    pub id: String,
    /// "morning" / "day" / "evening" / "shutdown"
    pub checkin_type: String,
    pub text: String,
    /// ISO-8601 timestamp
    pub created_at: String,
}

/// View model for a local rule suggestion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionDto {
    /// Rule type that generated the suggestion (e.g. "habit_reminder")
    pub proposal_type: String,
    /// Why this suggestion was generated
    pub reason: String,
    /// What the user should do
    pub suggestion: String,
}

// ─── helpers ──────────────────────────────────

// ─── enum-to-string helpers (stable contract) ──

pub(crate) fn habit_event_status_to_string(done: bool) -> &'static str {
    if done { "done" } else { "skipped" }
}

pub(crate) fn habit_level_to_string(level: &HabitEventLevel) -> &'static str {
    match level {
        HabitEventLevel::Min => "min",
        HabitEventLevel::Light => "light",
        HabitEventLevel::Base => "base",
        HabitEventLevel::Full => "full",
        HabitEventLevel::Custom => "custom",
    }
}

pub(crate) fn timer_mode_to_string(mode: &TimerMode) -> &'static str {
    match mode {
        TimerMode::Focus => "focus",
        TimerMode::Rest => "rest",
        TimerMode::Custom => "custom",
    }
}

pub(crate) fn context_doc_type_to_string(doc_type: &ContextDocumentType) -> &'static str {
    match doc_type {
        ContextDocumentType::Core => "core",
        ContextDocumentType::Goals => "goals",
        ContextDocumentType::Rules => "rules",
        ContextDocumentType::Routine => "routine",
        ContextDocumentType::Health => "health",
        ContextDocumentType::Work => "work",
        ContextDocumentType::Custom => "custom",
        ContextDocumentType::LifeCore => "life_core",
        ContextDocumentType::RecoveryProtocol => "recovery_protocol",
        ContextDocumentType::Tone => "tone",
        ContextDocumentType::PlanningPreferences => "planning_preferences",
    }
}

#[allow(dead_code)]
pub(crate) fn checkin_type_to_string(ct: &CheckInType) -> &'static str {
    match ct {
        CheckInType::Morning => "morning",
        CheckInType::Day => "day",
        CheckInType::Evening => "evening",
        CheckInType::Shutdown => "shutdown",
    }
}

pub(crate) fn naive_date_to_string(d: NaiveDate) -> String {
    d.format("%Y-%m-%d").to_string()
}

pub(crate) fn option_string(s: &Option<String>) -> String {
    s.as_deref().unwrap_or("").to_string()
}

pub(crate) fn option_u8_string(v: &Option<u8>) -> String {
    v.map(|n| n.to_string()).unwrap_or_else(|| "-".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn today_view_dto_serde_round_trip() {
        let dto = TodayViewDto {
            date: "2026-06-09".into(),
            has_daily_log: true,
            mode: "none".into(),
            sleep_score: "7".into(),
            energy: "6".into(),
            mood: "8".into(),
            check_in_count: 1,
            has_morning_check_in: true,
            has_evening_check_in: false,
            habit_count: 3,
            habit_events_done: 1,
            has_plan: true,
            plan_title: "Plan".into(),
            plan_item_count: 2,
            plan_items: vec![PlanItemViewDto {
                description: "Task".into(),
                status: "Planned".into(),
                estimated_minutes: "".into(),
                notes: "".into(),
            }],
            suggestion_count: 0,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: TodayViewDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.date, back.date);
        assert_eq!(dto.has_morning_check_in, back.has_morning_check_in);
        assert_eq!(dto.plan_items.len(), back.plan_items.len());
    }

    #[test]
    fn startup_view_dto_serde_round_trip() {
        let dto = StartupViewDto {
            intent: "start_day_required".into(),
            reason: "No morning check-in.".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: StartupViewDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.intent, back.intent);
    }

    #[test]
    fn current_activity_dto_serde_round_trip() {
        let dto = CurrentActivityDto {
            activity_kind: "scheduled_task".into(),
            active_task: "Work".into(),
            active_block: "".into(),
            next_checkpoint_at: "".into(),
            recommended_prompt: "Working on: Work".into(),
            date: "2026-06-09".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: CurrentActivityDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.activity_kind, back.activity_kind);
    }

    #[test]
    fn plan_item_dto_serde_round_trip() {
        let dto = PlanItemDto {
            id: "a1b2c3d4-1234-5678-9abc-def012345678".into(),
            title: "Write docs".into(),
            description: "Phase 7.1".into(),
            quadrant: "important-urgent".into(),
            planned_start: "09:00".into(),
            planned_end: "11:00".into(),
            status: "Planned".into(),
            priority: 1,
            source: "Manual".into(),
            created_at: "2026-06-09T08:00:00Z".into(),
            updated_at: "2026-06-09T09:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: PlanItemDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
        assert_eq!(dto.status, back.status);
    }

    #[test]
    fn waiting_task_dto_serde_round_trip() {
        let dto = WaitingTaskDto {
            id: "uuid".into(),
            title: "Wait".into(),
            waiting_since: "2026-06-01T10:00:00Z".into(),
            review_due: "2026-06-08".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: WaitingTaskDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
    }

    #[test]
    fn day_plan_dto_serde_round_trip() {
        let dto = DayPlanDto {
            id: "".into(),
            date: "2026-06-09".into(),
            items: vec![],
            item_count: 0,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: DayPlanDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.date, back.date);
    }

    #[test]
    fn checklist_template_dto_serde_round_trip() {
        let dto = ChecklistTemplateDto {
            id: "uuid".into(),
            title: "Morning".into(),
            category: "morning".into(),
            item_count: 4,
            is_active: true,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ChecklistTemplateDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
    }

    #[test]
    fn checklist_run_dto_serde_round_trip() {
        let dto = ChecklistRunDto {
            id: "uuid".into(),
            template_title: "Morning".into(),
            started_at: "2026-06-09T07:00:00Z".into(),
            completed_at: "".into(),
            answer_count: 0,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ChecklistRunDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.template_title, back.template_title);
    }

    #[test]
    fn journal_entry_dto_serde_round_trip() {
        let dto = JournalEntryDto {
            id: "uuid".into(),
            entry_type: "Note".into(),
            summary: "Test entry".into(),
            timestamp: "2026-06-09T12:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: JournalEntryDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.entry_type, back.entry_type);
    }

    #[test]
    fn notification_instruction_dto_serde_round_trip() {
        let dto = NotificationInstructionDto {
            source: "Checkpoint".into(),
            title: "Time to check in".into(),
            body: "How is your task going?".into(),
            action_id: "checkpoint_123".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: NotificationInstructionDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
    }

    #[test]
    fn core_error_dto_serde_round_trip() {
        let dto = CoreErrorDto {
            code: "NOT_FOUND".into(),
            message: "not found: item".into(),
            recoverable: true,
            suggested_action: "Check item exists".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: CoreErrorDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.code, back.code);
        assert_eq!(dto.recoverable, back.recoverable);
    }

    #[test]
    fn habit_dto_serde_round_trip() {
        let dto = HabitDto {
            id: "uuid".into(),
            name: "Exercise".into(),
            is_active: true,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: HabitDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.name, back.name);
        assert_eq!(dto.is_active, back.is_active);
    }

    #[test]
    fn habit_event_dto_serde_round_trip() {
        let dto = HabitEventDto {
            id: "evt-uuid".into(),
            habit_id: "hab-uuid".into(),
            habit_name: "Exercise".into(),
            status: "done".into(),
            level: "base".into(),
            date_time: "2026-06-19T10:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: HabitEventDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.habit_id, back.habit_id);
        assert_eq!(dto.habit_name, back.habit_name);
        assert_eq!(dto.status, back.status);
        assert_eq!(dto.level, back.level);
        assert_eq!(dto.date_time, back.date_time);
    }

    #[test]
    fn timer_dto_serde_round_trip() {
        let dto = TimerDto {
            id: "timer-uuid".into(),
            title: "Focus".into(),
            duration_seconds: 1500,
            mode: "focus".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: TimerDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.title, back.title);
        assert_eq!(dto.duration_seconds, back.duration_seconds);
        assert_eq!(dto.mode, back.mode);
    }

    #[test]
    fn reminder_dto_serde_round_trip() {
        let dto = ReminderDto {
            id: "rem-uuid".into(),
            title: "Drink water".into(),
            schedule_rule: "every 2h".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ReminderDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.title, back.title);
        assert_eq!(dto.schedule_rule, back.schedule_rule);
    }

    #[test]
    fn alarm_dto_serde_round_trip() {
        let dto = AlarmDto {
            id: "alarm-uuid".into(),
            title: "Wake up".into(),
            time: "07:00".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: AlarmDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.title, back.title);
        assert_eq!(dto.time, back.time);
    }

    #[test]
    fn context_document_dto_serde_round_trip() {
        let dto = ContextDocumentDto {
            id: "doc-uuid".into(),
            doc_type: "core".into(),
            title: "My Core".into(),
            content: "# Values\n\nBe kind.".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ContextDocumentDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.doc_type, back.doc_type);
        assert_eq!(dto.title, back.title);
        assert_eq!(dto.content, back.content);
    }

    #[test]
    fn check_in_dto_serde_round_trip() {
        let dto = CheckInDto {
            id: "ci-uuid".into(),
            checkin_type: "morning".into(),
            text: "feeling great".into(),
            created_at: "2026-06-19T07:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: CheckInDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.id, back.id);
        assert_eq!(dto.checkin_type, back.checkin_type);
        assert_eq!(dto.text, back.text);
        assert_eq!(dto.created_at, back.created_at);
    }

    #[test]
    fn suggestion_dto_serde_round_trip() {
        let dto = SuggestionDto {
            proposal_type: "habit_reminder".into(),
            reason: "Habit not done today".into(),
            suggestion: "Do your morning exercise".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: SuggestionDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.proposal_type, back.proposal_type);
        assert_eq!(dto.reason, back.reason);
        assert_eq!(dto.suggestion, back.suggestion);
    }

    // ── Golden JSON contract validation ──

    #[test]
    fn contract_example_startup_state_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/startup-state.json");
        let dto: StartupViewDto = serde_json::from_str(json)
            .expect("startup-state.json must deserialize into StartupViewDto");
        assert!(!dto.intent.is_empty());
        assert!(!dto.reason.is_empty());
    }

    #[test]
    fn contract_example_today_dashboard_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/today-dashboard.json");
        let dto: TodayViewDto = serde_json::from_str(json)
            .expect("today-dashboard.json must deserialize into TodayViewDto");
        assert!(!dto.date.is_empty());
    }

    #[test]
    fn contract_example_current_activity_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/current-activity.json");
        let dto: CurrentActivityDto = serde_json::from_str(json)
            .expect("current-activity.json must deserialize into CurrentActivityDto");
        assert!(!dto.activity_kind.is_empty());
    }

    #[derive(Debug, Deserialize)]
    struct PlanItemLifecycleWrapper {
        lifecycle: Vec<LifecycleStage>,
    }
    #[derive(Debug, Deserialize)]
    struct LifecycleStage {
        stage: String,
        plan_item: PlanItemDto,
    }

    #[test]
    fn contract_example_plan_item_lifecycle_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/plan-item-lifecycle.json");
        let wrapper: PlanItemLifecycleWrapper =
            serde_json::from_str(json).expect("plan-item-lifecycle.json must deserialize");
        assert_eq!(wrapper.lifecycle.len(), 3);
        assert_eq!(wrapper.lifecycle[0].stage, "created");
        assert_eq!(wrapper.lifecycle[1].plan_item.status, "Started");
        assert_eq!(wrapper.lifecycle[2].plan_item.status, "Done");
    }

    #[derive(Debug, Deserialize)]
    struct WaitingReviewWrapper {
        waiting_tasks: Vec<WaitingTaskDto>,
    }

    #[test]
    fn contract_example_waiting_review_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/waiting-review.json");
        let wrapper: WaitingReviewWrapper =
            serde_json::from_str(json).expect("waiting-review.json must deserialize");
        assert!(!wrapper.waiting_tasks.is_empty());
    }

    #[derive(Debug, Deserialize)]
    struct ChecklistRunWrapper {
        template: ChecklistTemplateDto,
        run: ChecklistRunDto,
    }

    #[test]
    fn contract_example_checklist_run_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/checklist-run.json");
        let wrapper: ChecklistRunWrapper =
            serde_json::from_str(json).expect("checklist-run.json must deserialize");
        assert_eq!(wrapper.template.category, "morning");
        assert_eq!(wrapper.run.answer_count, 4);
    }

    #[derive(Debug, Deserialize)]
    struct JournalFeedWrapper {
        entries: Vec<JournalEntryDto>,
    }

    #[test]
    fn contract_example_journal_feed_deserializable() {
        let json = include_str!("../../../docs/contracts/examples/journal-feed.json");
        let wrapper: JournalFeedWrapper =
            serde_json::from_str(json).expect("journal-feed.json must deserialize");
        assert_eq!(wrapper.entries.len(), 3);
        assert_eq!(wrapper.entries[0].entry_type, "CheckInCreated");
    }
}
