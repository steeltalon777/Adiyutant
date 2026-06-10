use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

// ──────────────────────────────────────────────
// Primary DTOs — flat, stable output models
// Rules: no Id<T>, no serde_json::Value, no chrono inner types.
// All dates/times are ISO-8601 strings.
// All DTOs are Serialize + Deserialize for JSON bridge / UniFFI.
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

// ─── Structured Error DTO ─────────────────────

/// Structured error for external consumers (Android, CLI, contract tests).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CoreErrorDto {
    /// Machine-readable error code: "NOT_FOUND", "INVALID_INPUT", "STORAGE_ERROR", "INTERNAL_ERROR".
    pub code: String,
    /// Human-readable message for developer logs.
    pub message: String,
    /// Whether the client can recover (retry, change input, fix state).
    pub recoverable: bool,
    /// Suggested action for the client: "Retry", "Check input", "Contact support".
    pub suggested_action: String,
}

// ─── helpers ──────────────────────────────────

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
    use serde_json;

    fn make_today_view() -> TodayViewDto {
        TodayViewDto {
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
            has_plan: false,
            plan_title: String::new(),
            plan_item_count: 0,
            plan_items: vec![],
            suggestion_count: 2,
        }
    }

    #[test]
    fn today_view_dto_serde_round_trip() {
        let dto = make_today_view();
        let json = serde_json::to_string(&dto).unwrap();
        let back: TodayViewDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.date, back.date);
        assert_eq!(dto.mode, back.mode);
        assert_eq!(dto.suggestion_count, back.suggestion_count);
    }

    #[test]
    fn startup_view_dto_serde_round_trip() {
        let dto = StartupViewDto {
            intent: "start_day_required".into(),
            reason: "No morning check-in".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: StartupViewDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.intent, back.intent);
        assert_eq!(dto.reason, back.reason);
    }

    #[test]
    fn current_activity_dto_serde_round_trip() {
        let dto = CurrentActivityDto {
            date: "2026-06-09".into(),
            activity_kind: "scheduled_task".into(),
            active_task: "Android shell".into(),
            active_block: String::new(),
            next_checkpoint_at: "12:00".into(),
            recommended_prompt: "Ready to start: Android shell".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: CurrentActivityDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.activity_kind, back.activity_kind);
        assert_eq!(dto.active_task, back.active_task);
    }

    #[test]
    fn day_plan_dto_serde_round_trip() {
        let dto = DayPlanDto {
            id: "uuid-1".into(),
            date: "2026-06-09".into(),
            items: vec![PlanItemDto {
                id: "uuid-2".into(),
                title: "Task".into(),
                description: String::new(),
                quadrant: "important-urgent".into(),
                planned_start: String::new(),
                planned_end: String::new(),
                status: "Planned".into(),
                priority: 1,
                source: "Manual".into(),
                created_at: "2026-01-01T00:00:00Z".into(),
                updated_at: "2026-01-01T00:00:00Z".into(),
            }],
            item_count: 1,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: DayPlanDto = serde_json::from_str(&json).unwrap();
        assert_eq!(back.item_count, 1);
        assert_eq!(back.items[0].title, "Task");
    }

    #[test]
    fn waiting_task_dto_serde_round_trip() {
        let dto = WaitingTaskDto {
            id: "uuid-3".into(),
            title: "Waiting".into(),
            waiting_since: "2026-01-01".into(),
            review_due: "2026-01-08".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: WaitingTaskDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
    }

    #[test]
    fn checklist_template_dto_serde_round_trip() {
        let dto = ChecklistTemplateDto {
            id: "uuid-4".into(),
            title: "Morning".into(),
            category: "morning".into(),
            item_count: 4,
            is_active: true,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ChecklistTemplateDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.category, back.category);
    }

    #[test]
    fn checklist_run_dto_serde_round_trip() {
        let dto = ChecklistRunDto {
            id: "uuid-5".into(),
            template_title: "Morning".into(),
            started_at: "2026-01-01T00:00:00Z".into(),
            completed_at: String::new(),
            answer_count: 0,
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: ChecklistRunDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.answer_count, back.answer_count);
    }

    #[test]
    fn journal_entry_dto_serde_round_trip() {
        let dto = JournalEntryDto {
            id: "uuid-6".into(),
            entry_type: "Note".into(),
            summary: "test".into(),
            timestamp: "2026-01-01T00:00:00Z".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: JournalEntryDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.summary, back.summary);
    }

    #[test]
    fn core_error_dto_serde_round_trip() {
        let dto = CoreErrorDto {
            code: "NOT_FOUND".into(),
            message: "plan_item abc".into(),
            recoverable: true,
            suggested_action: "Check item exists".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: CoreErrorDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.code, back.code);
        assert_eq!(dto.recoverable, back.recoverable);
    }

    #[test]
    fn notification_instruction_dto_serde_round_trip() {
        let dto = NotificationInstructionDto {
            source: "Checkpoint".into(),
            title: "Progress check".into(),
            body: "How is it going?".into(),
            action_id: "checkpoint-123".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let back: NotificationInstructionDto = serde_json::from_str(&json).unwrap();
        assert_eq!(dto.title, back.title);
    }

    #[test]
    fn core_error_dto_json_fields() {
        let dto = CoreErrorDto {
            code: "STORAGE_ERROR".into(),
            message: "disk full".into(),
            recoverable: false,
            suggested_action: "Retry or check disk".into(),
        };
        let json = serde_json::to_string(&dto).unwrap();
        let v: serde_json::Value = serde_json::from_str(&json).unwrap();
        assert_eq!(v["code"], "STORAGE_ERROR");
        assert_eq!(v["recoverable"], false);
        assert_eq!(v["suggested_action"], "Retry or check disk");
    }
}
