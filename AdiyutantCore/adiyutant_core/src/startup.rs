use chrono::Timelike;

use crate::today_state::TodayState;

/// What the app should suggest to the user on launch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartupIntent {
    StartDayRequired,
    OfferDailySchedule,
    ShowTodayDashboard,
    SuggestEveningShutdown,
    NoAction,
}

impl StartupIntent {
    pub fn as_str(&self) -> &'static str {
        match self {
            StartupIntent::StartDayRequired => "start_day_required",
            StartupIntent::OfferDailySchedule => "offer_daily_schedule",
            StartupIntent::ShowTodayDashboard => "show_today_dashboard",
            StartupIntent::SuggestEveningShutdown => "suggest_evening_shutdown",
            StartupIntent::NoAction => "no_action",
        }
    }
}

/// Result of determining what to show on launch.
#[derive(Debug, Clone)]
pub struct StartupState {
    pub intent: StartupIntent,
    pub reason: String,
}

impl StartupState {
    /// Determine the startup intent based on today's state using current time.
    pub fn determine(today_state: &TodayState) -> Self {
        Self::determine_at_hour(today_state, chrono::Utc::now().hour())
    }

    /// Determine the startup intent at a given hour (0–23) for testability.
    ///
    /// Logic:
    /// 1. No morning check-in → `StartDayRequired`
    /// 2. Morning check-in exists, no plan → `OfferDailySchedule`
    /// 3. Evening / late hour (hour ∉ [6,20)), no shutdown → `SuggestEveningShutdown`
    /// 4. Fallback → `ShowTodayDashboard`
    pub fn determine_at_hour(today_state: &TodayState, hour: u32) -> Self {
        let has_morning = today_state.morning_check_in().is_some();
        let has_plan = today_state.has_plan();
        let has_evening = today_state.evening_check_in().is_some();

        // Rule 1: No morning check-in → start the day
        if !has_morning {
            return StartupState {
                intent: StartupIntent::StartDayRequired,
                reason: "No morning check-in yet. Start your day by checking in.".to_string(),
            };
        }

        // Rule 2: Morning done but no plan → offer schedule
        if !has_plan {
            return StartupState {
                intent: StartupIntent::OfferDailySchedule,
                reason: "Morning check-in done. Create a plan for today.".to_string(),
            };
        }

        // Rule 3: Evening/late hour without shutdown
        if !has_evening && !(6..20).contains(&hour) {
            return StartupState {
                intent: StartupIntent::SuggestEveningShutdown,
                reason: "Late hour detected. Consider starting your evening shutdown.".to_string(),
            };
        }

        // Rule 4: Fallback
        StartupState {
            intent: StartupIntent::ShowTodayDashboard,
            reason: "Showing today's dashboard.".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::check_in::{CheckIn, CheckInType};
    use crate::model::daily_log::DailyLog;
    use crate::model::plan::Plan;
    use crate::today_state::TodayStateBuilder;

    fn make_state() -> TodayStateBuilder {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        TodayStateBuilder::new().date(date)
    }

    #[test]
    fn no_morning_check_in_returns_start_day() {
        let state = make_state().build();
        let result = StartupState::determine_at_hour(&state, 10);
        assert_eq!(result.intent, StartupIntent::StartDayRequired);
    }

    #[test]
    fn morning_without_plan_offers_schedule() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let state = make_state().daily_log(log).check_in(ci).build();
        let result = StartupState::determine_at_hour(&state, 10);
        assert_eq!(result.intent, StartupIntent::OfferDailySchedule);
    }

    #[test]
    fn morning_and_plan_shows_dashboard() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Task 1".into());
        let state = make_state().daily_log(log).check_in(ci).plan(plan).build();
        // Use hour=10 (mid-morning) to avoid evening shutdown trigger
        let result = StartupState::determine_at_hour(&state, 10);
        assert_eq!(result.intent, StartupIntent::ShowTodayDashboard);
    }

    #[test]
    fn late_hour_without_evening_suggests_shutdown() {
        let date = chrono::NaiveDate::from_ymd_opt(2026, 6, 9).unwrap();
        let log = DailyLog::new(date);
        let ci = CheckIn::new(log.id, CheckInType::Morning, "gm".into());
        let mut plan = Plan::new(log.id, "Today".into());
        plan.add_item("Task 1".into());
        let state = make_state().daily_log(log).check_in(ci).plan(plan).build();
        let result = StartupState::determine_at_hour(&state, 22);
        assert_eq!(result.intent, StartupIntent::SuggestEveningShutdown);
    }

    #[test]
    fn startup_intent_as_str_returns_kebab() {
        assert_eq!(
            StartupIntent::StartDayRequired.as_str(),
            "start_day_required"
        );
        assert_eq!(
            StartupIntent::ShowTodayDashboard.as_str(),
            "show_today_dashboard"
        );
    }
}
