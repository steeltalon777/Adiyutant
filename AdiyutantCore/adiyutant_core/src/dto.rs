use chrono::NaiveDate;

// ──────────────────────────────────────────────
// Primary DTOs — flat, stable output models
// Rules: no Id<T>, no serde_json::Value, no chrono inner types.
// All dates/times are ISO-8601 strings.
// ──────────────────────────────────────────────

/// View model for the "today" dashboard.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct PlanItemViewDto {
    pub description: String,
    pub status: String,
    pub estimated_minutes: String,
    pub notes: String,
}

/// View model for the startup screen decision.
#[derive(Debug, Clone)]
pub struct StartupViewDto {
    pub intent: String,
    pub reason: String,
}

/// View model for the "current activity" central screen.
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct DayPlanDto {
    pub id: String,
    pub date: String,
    pub items: Vec<PlanItemDto>,
    pub item_count: usize,
}

/// View model for a single plan item (new PlanItem model).
#[derive(Debug, Clone)]
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
#[derive(Debug, Clone)]
pub struct WaitingTaskDto {
    pub id: String,
    pub title: String,
    pub waiting_since: String,
    pub review_due: String,
}

/// Instruction for the UI layer to display a notification.
#[derive(Debug, Clone)]
pub struct NotificationInstructionDto {
    pub source: String,
    pub title: String,
    pub body: String,
    pub action_id: String,
}

// ─── Wave C: Checklist & Journal DTOs ─────────

/// View model for a checklist template.
#[derive(Debug, Clone)]
pub struct ChecklistTemplateDto {
    pub id: String,
    pub title: String,
    pub category: String,
    pub item_count: usize,
    pub is_active: bool,
}

/// View model for a checklist run.
#[derive(Debug, Clone)]
pub struct ChecklistRunDto {
    pub id: String,
    pub template_title: String,
    pub started_at: String,
    pub completed_at: String,
    pub answer_count: usize,
}

/// View model for a journal entry.
#[derive(Debug, Clone)]
pub struct JournalEntryDto {
    pub id: String,
    pub entry_type: String,
    pub summary: String,
    pub timestamp: String,
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
