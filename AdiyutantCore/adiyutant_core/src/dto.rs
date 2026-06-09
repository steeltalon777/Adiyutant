use chrono::NaiveDate;

// ──────────────────────────────────────────────
// Primary DTOs — flat, stable output models
// Rules: no Id<T>, no serde_json::Value, no chrono inner types.
// All dates/times are ISO-8601 strings.
// ──────────────────────────────────────────────

/// View model for the "today" dashboard.
#[derive(Debug, Clone)]
pub struct TodayViewDto {
    pub date: String, // ISO-8601 date
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

/// Flat view of a single plan item.
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
