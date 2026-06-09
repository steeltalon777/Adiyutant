use chrono::Timelike;

use crate::today_state::TodayState;

/// What kind of activity is expected right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedActivityKind {
    NoPlan,
    StartDay,
    ScheduledTask,
    Break,
    FreeTime,
    Meal,
    Recovery,
    Shutdown,
    SleepWindow,
}

impl ExpectedActivityKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ExpectedActivityKind::NoPlan => "no_plan",
            ExpectedActivityKind::StartDay => "start_day",
            ExpectedActivityKind::ScheduledTask => "scheduled_task",
            ExpectedActivityKind::Break => "break",
            ExpectedActivityKind::FreeTime => "free_time",
            ExpectedActivityKind::Meal => "meal",
            ExpectedActivityKind::Recovery => "recovery",
            ExpectedActivityKind::Shutdown => "shutdown",
            ExpectedActivityKind::SleepWindow => "sleep_window",
        }
    }
}

/// What the user is (or should be) doing right now.
#[derive(Debug, Clone)]
pub struct CurrentActivity {
    pub kind: ExpectedActivityKind,
    pub active_task: Option<String>,
    pub active_block: Option<String>,
    pub next_checkpoint: Option<chrono::NaiveTime>,
    pub recommended_prompt: String,
}

impl CurrentActivity {
    /// Determine the current activity based on today's state using current time.
    pub fn determine(today_state: &TodayState) -> Self {
        Self::determine_at_hour(today_state, chrono::Utc::now().hour())
    }

    /// Determine the current activity at a given hour (0–23) for testability.
    ///
    /// Logic (simple MVP heuristic):
    /// 1. No morning check-in → StartDay
    /// 2. Recovery mode (no plan) → Recovery
    /// 3. No plan → NoPlan
    /// 4. Plan with in-progress items → ScheduledTask
    /// 5. Evening (hour ∉ [6,20)) without shutdown → Shutdown
    /// 6. Fallback → FreeTime
    pub fn determine_at_hour(today_state: &TodayState, hour: u32) -> Self {
        let has_morning = today_state.morning_check_in().is_some();
        let has_evening = today_state.evening_check_in().is_some();
        let is_late = !(6..20).contains(&hour);

        // 1. No morning check-in
        if !has_morning {
            return CurrentActivity {
                kind: ExpectedActivityKind::StartDay,
                active_task: None,
                active_block: None,
                next_checkpoint: None,
                recommended_prompt: "Ready to start your day? Do a morning check-in.".to_string(),
            };
        }

        // 2–3. No plan (with recovery check)
        if !today_state.has_plan() {
            let is_recovery = today_state
                .daily_log
                .as_ref()
                .and_then(|l| l.mode.as_deref())
                == Some("recovery");

            if is_recovery {
                return CurrentActivity {
                    kind: ExpectedActivityKind::Recovery,
                    active_task: None,
                    active_block: None,
                    next_checkpoint: None,
                    recommended_prompt: "Recovery mode active. Rest and recharge.".to_string(),
                };
            }

            return CurrentActivity {
                kind: ExpectedActivityKind::NoPlan,
                active_task: None,
                active_block: None,
                next_checkpoint: None,
                recommended_prompt: "No plan for today. Create one to stay on track.".to_string(),
            };
        }

        // 4. Check for in-progress task
        if let Some(plan) = &today_state.plan {
            let started_item = plan
                .items
                .iter()
                .find(|i| matches!(i.status, crate::model::plan::PlanItemStatus::InProgress));
            if let Some(item) = started_item {
                return CurrentActivity {
                    kind: ExpectedActivityKind::ScheduledTask,
                    active_task: Some(item.description.clone()),
                    active_block: None,
                    next_checkpoint: None,
                    recommended_prompt: format!("Working on: {}", item.description),
                };
            }

            // No started items — suggest first pending
            let first_pending = plan
                .items
                .iter()
                .find(|i| matches!(i.status, crate::model::plan::PlanItemStatus::Pending));
            if let Some(item) = first_pending {
                return CurrentActivity {
                    kind: ExpectedActivityKind::ScheduledTask,
                    active_task: Some(item.description.clone()),
                    active_block: None,
                    next_checkpoint: None,
                    recommended_prompt: format!("Ready to start: {}", item.description),
                };
            }
        }

        // 5. Evening without shutdown
        if is_late && !has_evening {
            return CurrentActivity {
                kind: ExpectedActivityKind::Shutdown,
                active_task: None,
                active_block: None,
                next_checkpoint: None,
                recommended_prompt: "It's getting late. Consider starting your evening shutdown."
                    .to_string(),
            };
        }

        // 6. Fallback
        CurrentActivity {
            kind: ExpectedActivityKind::FreeTime,
            active_task: None,
            active_block: None,
            next_checkpoint: None,
            recommended_prompt: "No active task. What would you like to do?".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::check_in::{CheckIn, CheckInType};
    use crate::model::daily_log::DailyLog;
    use crate::model::plan::{Plan, PlanItemStatus};
    use crate::today_state::TodayStateBuilder;

    fn make_state() -> TodayStateBuilder {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        TodayStateBuilder::new().date(date)
    }

    #[test]
    fn no_morning_returns_start_day() {
        let state = make_state().build();
        let result = CurrentActivity::determine_at_hour(&state, 10);
        assert_eq!(result.kind, ExpectedActivityKind::StartDay);
    }

    #[test]
    fn morning_no_plan_returns_no_plan() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let state = make_state().daily_log(log).check_in(ci).build();
        let result = CurrentActivity::determine_at_hour(&state, 10);
        assert_eq!(result.kind, ExpectedActivityKind::NoPlan);
    }

    #[test]
    fn morning_plan_with_in_progress_returns_scheduled_task() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Active task".into());
        plan.items[0].status = PlanItemStatus::InProgress;
        let state = make_state().daily_log(log).check_in(ci).plan(plan).build();
        let result = CurrentActivity::determine_at_hour(&state, 10);
        assert_eq!(result.kind, ExpectedActivityKind::ScheduledTask);
        assert_eq!(result.active_task.as_deref(), Some("Active task"));
    }

    #[test]
    fn recovery_mode_detected() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let mut log = DailyLog::new(date);
        log.mode = Some("recovery".into());
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let state = make_state().daily_log(log).check_in(ci).build();
        let result = CurrentActivity::determine_at_hour(&state, 10);
        assert_eq!(result.kind, ExpectedActivityKind::Recovery);
    }

    #[test]
    fn late_hour_without_evening_suggests_shutdown() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Task 1".into());
        plan.items[0].status = PlanItemStatus::Done;
        let state = make_state().daily_log(log).check_in(ci).plan(plan).build();
        let result = CurrentActivity::determine_at_hour(&state, 22);
        assert_eq!(result.kind, ExpectedActivityKind::Shutdown);
    }

    #[test]
    fn activity_kind_as_str() {
        assert_eq!(ExpectedActivityKind::NoPlan.as_str(), "no_plan");
        assert_eq!(
            ExpectedActivityKind::ScheduledTask.as_str(),
            "scheduled_task"
        );
    }
}
